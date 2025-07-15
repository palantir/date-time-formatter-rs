// Copyright (C) 2025 Palantir
// This program is free software; you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation; either version 2 of the License, or (at your option)
// any later version.
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for
// more details.
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc., 59
// Temple Place, Suite 330, Boston, MA 02111-1307 USA
// sbhatia@palantir.com
// "CLASSPATH" EXCEPTION TO THE GPL
// Certain source files distributed by Oracle America and/or its affiliates are
// subject to the following clarification and special exception to the GPL, but
// only where Oracle has expressly included in the particular source file's header
// the words "Oracle designates this particular file as subject to the "Classpath"
// exception as provided by Oracle in the LICENSE file that accompanied this code."
// Linking this library statically or dynamically with other modules is making
// a combined work based on this library.  Thus, the terms and conditions of
// the GNU General Public License cover the whole combination.
// As a special exception, the copyright holders of this library give you
// permission to link this library with independent modules to produce an
// executable, regardless of the license terms of these independent modules,
// and to copy and distribute the resulting executable under terms of your
// choice, provided that you also meet, for each linked independent module,
// the terms and conditions of the license of that module.  An independent
// module is a module which is not derived from or based on this library.  If
// you modify this library, you may extend this exception to your version of
// the library, but you are not obligated to do so.  If you do not wish to do
// so, delete this exception statement from your version. "

use crate::number_printer_parser::BaseNumberPrinterParser;
use crate::{
    date_time_parse_context::DateTimeParseContext, date_time_print_context::DateTimePrintContext,
    date_time_printer_parser::DateTimePrinterParser, number_printer_parser::NumberPrinterParser,
    reduced_printer_parser::ReducedPrinterParser, sign_style::SignStyle,
    temporal_field::TemporalField,
};

#[derive(Debug)]
pub struct WeekBasedFieldPrinterParser {
    field: TemporalField,
    min_width: isize,
    max_width: isize,
    sign_style: SignStyle,
    subsequent_width: isize,

    // Fields not from NumberPrinterParser
    chr: char,
    count: isize,
}
impl WeekBasedFieldPrinterParser {
    pub fn new(chr: char, count: isize, min_width: isize, max_width: isize) -> Self {
        WeekBasedFieldPrinterParser {
            field: TemporalField::DayOfWeek,
            min_width,
            max_width,
            sign_style: SignStyle::NotNegative,
            subsequent_width: 0,
            chr,
            count,
        }
    }

    fn printer_parser(&self) -> Result<Box<dyn DateTimePrinterParser>, String> {
        match self.chr {
            'Y' => {
                let field = TemporalField::WeekBasedYear;
                if self.count == 2 {
                    let mut pp =
                        ReducedPrinterParser::new(field, 2, 2, ReducedPrinterParser::BASE_VALUE);
                    pp.set_subsequent_width(self.subsequent_width);
                    Ok(Box::new(pp))
                } else {
                    let mut pp = BaseNumberPrinterParser::new(
                        field,
                        self.count,
                        19,
                        if self.count < 4 {
                            SignStyle::Normal
                        } else {
                            SignStyle::ExceedsPad
                        },
                    );
                    pp.set_subsequent_width(self.subsequent_width);
                    Ok(Box::new(pp))
                }
            }
            'e' | 'c' => {
                let mut pp = BaseNumberPrinterParser::new(
                    // NOTE: This will always intercept these patterns,
                    // so the map entry in `TemporalField` which points
                    // to DayOfWeek instead is somewhat misleading.
                    TemporalField::LocaleDayOfWeek,
                    self.min_width,
                    self.max_width,
                    SignStyle::NotNegative,
                );
                pp.set_subsequent_width(self.subsequent_width);
                Ok(Box::new(pp))
            }
            'w' => {
                let mut pp = BaseNumberPrinterParser::new(
                    TemporalField::WeekOfWeekBasedYear,
                    self.min_width,
                    self.max_width,
                    SignStyle::NotNegative,
                );
                pp.set_subsequent_width(self.subsequent_width);
                Ok(Box::new(pp))
            }
            'W' => {
                let mut pp = BaseNumberPrinterParser::new(
                    TemporalField::WeekOfMonth,
                    self.min_width,
                    self.max_width,
                    SignStyle::NotNegative,
                );
                pp.set_subsequent_width(self.subsequent_width);
                Ok(Box::new(pp))
            }
            _ => Err("unreachable".to_owned()),
        }
    }
}
impl DateTimePrinterParser for WeekBasedFieldPrinterParser {
    fn format(&self, context: &mut DateTimePrintContext, buf: &mut String) -> Result<bool, String> {
        self.printer_parser()?.format(context, buf)
    }
    fn parse(
        &self,
        context: &mut DateTimeParseContext,
        text: &str,
        position: isize,
    ) -> Result<isize, String> {
        self.printer_parser()?.parse(context, text, position)
    }
    fn as_number_printer_parser(&mut self) -> Option<&mut dyn NumberPrinterParser> {
        Some(self)
    }
}
impl NumberPrinterParser for WeekBasedFieldPrinterParser {
    fn get_field(&self) -> TemporalField {
        self.field
    }

    fn get_min_width(&self) -> isize {
        self.min_width
    }

    fn get_max_width(&self) -> isize {
        self.max_width
    }

    fn get_sign_style(&self) -> SignStyle {
        self.sign_style
    }

    fn get_subsequent_width(&self) -> isize {
        self.subsequent_width
    }

    fn set_subsequent_width(&mut self, subsequent_width: isize) {
        self.subsequent_width = subsequent_width;
    }
}
