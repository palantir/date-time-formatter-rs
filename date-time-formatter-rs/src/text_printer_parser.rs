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
    date_time_printer_parser::DateTimePrinterParser, date_time_text_provider::DateTimeTextProvider,
    number_printer_parser::BaseNumberPrinterParser, sign_style::SignStyle,
    temporal_field::TemporalField, text_style::TextStyle,
};

#[derive(Debug)]
pub struct TextPrinterParser {
    field: TemporalField,
    text_style: TextStyle,
    provider: DateTimeTextProvider,
    number_printer_parser: BaseNumberPrinterParser,
}
impl TextPrinterParser {
    pub fn new(field: TemporalField, text_style: TextStyle) -> Self {
        TextPrinterParser {
            field,
            text_style,
            provider: DateTimeTextProvider::new(),
            number_printer_parser: BaseNumberPrinterParser::new(field, 1, 19, SignStyle::Normal),
        }
    }
}
impl DateTimePrinterParser for TextPrinterParser {
    fn format(&self, context: &mut DateTimePrintContext, buf: &mut String) -> Result<bool, String> {
        let value = match context.get_value(self.field) {
            Some(value) => value,
            None => return Ok(false),
        };
        match self.provider.get_text(self.field, value, self.text_style) {
            Some(text) => {
                buf.push_str(text);
                Ok(true)
            }
            None => self.number_printer_parser.format(context, buf),
        }
    }

    fn parse(
        &self,
        context: &mut DateTimeParseContext,
        text: &str,
        position: isize,
    ) -> Result<isize, String> {
        let length = text.len();
        if position < 0 || position > length as isize {
            return Err("Index out of bounds".to_owned());
        }
        let style = if context.is_strict() {
            Some(self.text_style)
        } else {
            None
        };
        if let Some(it) = self.provider.get_text_iterator(self.field, style) {
            for (it_text, it_value) in it {
                if context.sub_sequence_equals(it_text, 0, text, position as usize, it_text.len()) {
                    return context.set_parsed_field(
                        self.field,
                        it_value,
                        position,
                        position + it_text.len() as isize,
                    );
                }
            }
            if self.field == TemporalField::Era && !context.is_strict() {
                // Taken from IsoEras, can probably add more here without causing too many problems
                let eras = ["BCE", "CE"];
                for (era_value, era) in eras.iter().enumerate() {
                    if context.sub_sequence_equals(era, 0, text, position as usize, era.len()) {
                        return context.set_parsed_field(
                            self.field,
                            era_value as i64,
                            position,
                            position + era.len() as isize,
                        );
                    }
                }
            }
            if context.is_strict() {
                return Ok(!position);
            }
        }
        self.number_printer_parser.parse(context, text, position)
    }
}
