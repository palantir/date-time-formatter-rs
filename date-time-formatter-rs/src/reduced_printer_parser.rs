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

use crate::{
    date_time_parse_context::DateTimeParseContext, date_time_print_context::DateTimePrintContext,
    number_printer_parser, number_printer_parser::DefaultNumberPrinterParser,
    number_printer_parser::NumberPrinterParser, sign_style::SignStyle,
    temporal_field::TemporalField,
};

#[derive(Debug)]
pub struct ReducedPrinterParser {
    field: TemporalField,
    min_width: isize,
    max_width: isize,
    sign_style: SignStyle,
    subsequent_width: isize,

    // Not from NumberPrinterParser
    base_value: i64,
}
impl ReducedPrinterParser {
    pub const BASE_VALUE: i64 = 2000;

    pub fn new(field: TemporalField, min_width: isize, max_width: isize, base_value: i64) -> Self {
        ReducedPrinterParser {
            field,
            min_width,
            max_width,
            sign_style: SignStyle::NotNegative,
            subsequent_width: 0,
            base_value,
        }
    }
}

impl NumberPrinterParser for ReducedPrinterParser {
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

impl DefaultNumberPrinterParser for ReducedPrinterParser {
    fn get_value(&self, _context: &DateTimePrintContext, value: i64) -> i64 {
        let abs_value = value.abs();
        // NOTE: We are ignoring the baseDate branch from the Java code
        // because we do not handle different Chronologies
        if value >= self.base_value
            && value
                < self.base_value + number_printer_parser::EXCEED_POINTS[self.min_width as usize]
        {
            return abs_value % number_printer_parser::EXCEED_POINTS[self.min_width as usize];
        }
        abs_value % number_printer_parser::EXCEED_POINTS[self.max_width as usize]
    }

    fn set_value(
        &self,
        context: &mut DateTimeParseContext,
        value: i64,
        error_pos: isize,
        success_pos: isize,
    ) -> Result<isize, String> {
        let mut value = value;
        let parse_len = success_pos - error_pos;
        if parse_len == self.min_width && value >= 0 {
            let range = number_printer_parser::EXCEED_POINTS[self.min_width as usize];
            let last_part = self.base_value % range;
            let base_part = self.base_value - last_part;
            if self.base_value > 0 {
                value += base_part;
            } else {
                value = base_part - value;
            }
            if value < self.base_value {
                value += range
            }
        }
        context.set_parsed_field(self.field, value, error_pos, success_pos)
    }
}
