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
    date_time_printer_parser::DateTimePrinterParser, number_printer_parser,
    number_printer_parser::NumberPrinterParser, sign_style::SignStyle,
    temporal_field::TemporalField,
};

#[derive(Debug)]
pub struct NanosPrinterParser {
    field: TemporalField,
    min_width: isize,
    max_width: isize,
    sign_style: SignStyle,
    subsequent_width: isize,

    // Fields not from NumberPrinterParser
    decimal_point: bool,
}
impl NanosPrinterParser {
    pub fn new(min_width: isize, max_width: isize, decimal_point: bool) -> Result<Self, String> {
        if !(0..=9).contains(&min_width) {
            return Err(format!(
                "Minimum width must be from 0 to 9 inclusive but was {min_width}"
            ));
        }
        if !(1..=9).contains(&max_width) {
            return Err(format!(
                "Maximum width must be from 1 to 9 inclusive but was {max_width}"
            ));
        }
        if max_width < min_width {
            return Err(format!(
                "Maximum width must exceed or equal the minimum width but {max_width} < {min_width}"
            ));
        }
        Ok(NanosPrinterParser {
            field: TemporalField::NanoOfSecond,
            min_width,
            max_width,
            sign_style: SignStyle::NotNegative,
            subsequent_width: 0,
            decimal_point,
        })
    }

    fn is_fixed_width(&self, context: &DateTimeParseContext) -> bool {
        context.is_strict() && self.min_width == self.max_width && !self.decimal_point
    }

    const TENS: [i32; 9] = [
        1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000,
    ];
}
impl DateTimePrinterParser for NanosPrinterParser {
    fn format(&self, context: &mut DateTimePrintContext, buf: &mut String) -> Result<bool, String> {
        let value = match context.get_value(self.field) {
            Some(value) => value,
            None => return Ok(false),
        };

        let val = self.field.range().check_valid_int_value(value)?;
        let decimal_style = context.get_formatter().get_decimal_style();
        let string_size = number_printer_parser::string_size(val as i64);
        let zero = decimal_style.get_zero_digit();

        if val == 0 || string_size < 10 - self.max_width as i32 {
            // 0 or would round down to 0
            // to get output identical to FractionPrinterParser use minWidth if the
            // value is zero, maxWidth otherwise
            let width = if val == 0 {
                self.min_width
            } else {
                self.max_width
            };
            if width > 0 {
                if self.decimal_point {
                    buf.push(decimal_style.get_decimal_separator());
                }
                for _ in 0..width {
                    buf.push(zero);
                }
            }
        } else {
            if self.decimal_point {
                buf.push(decimal_style.get_decimal_separator());
            }
            // add leading zeros
            let zeros = 9 - string_size;
            if zeros > 0 {
                for _ in 0..zeros {
                    buf.push(zero);
                }
            }

            // truncate unwanted digits
            let mut val = val as i64;
            if self.max_width < 9 {
                val /= Self::TENS[9 - self.max_width as usize] as i64;
            }

            // truncate zeros
            for _ in (self.min_width + 1..=self.max_width).rev() {
                if (val % 10) != 0 {
                    break;
                }
                val /= 10;
            }

            buf.push_str(&decimal_style.convert_number_to_i18n(&val.to_string())?);
        }
        Ok(true)
    }

    fn parse(
        &self,
        context: &mut DateTimeParseContext,
        text: &str,
        position: isize,
    ) -> Result<isize, String> {
        let effective_min = if context.is_strict() || self.is_fixed_width(context) {
            self.min_width
        } else {
            0
        };

        let effective_max = if context.is_strict() || self.is_fixed_width(context) {
            self.max_width
        } else {
            9
        };

        let length = text.len() as isize;
        if position == length {
            // valid if whole field is optional, invalid if minimum width
            return if effective_min > 0 {
                Ok(!position)
            } else {
                Ok(position)
            };
        }

        let text_chars: Vec<char> = text.chars().collect();
        if position < 0 || position >= text_chars.len() as isize {
            return Err("Index out of bounds".to_owned());
        }

        let mut position = position;
        if self.decimal_point {
            if text_chars[position as usize] != context.get_decimal_style().get_decimal_separator()
            {
                // valid if whole field is optional, invalid if minimum width
                return if effective_min > 0 {
                    Ok(!position)
                } else {
                    Ok(position)
                };
            }
            position += 1;
        }

        let min_end_pos = position + effective_min;
        if min_end_pos > length {
            return Ok(!position); // need at least min width digits
        }

        let max_end_pos = std::cmp::min(position + effective_max, length);
        let mut total = 0; // can use int because we are only parsing up to 9 digits
        let mut pos = position;

        while pos < max_end_pos {
            let ch = text_chars[pos as usize];
            let digit = context.get_decimal_style().convert_to_digit(ch);
            if digit < 0 {
                if pos < min_end_pos {
                    return Ok(!position); // need at least min width digits
                }
                break;
            }
            pos += 1;
            total = total * 10 + digit;
        }

        // Multiply by 10^9-n where n is the number of digits parsed
        for _ in 0..(9 - (pos - position)) {
            total *= 10;
        }

        context.set_parsed_field(self.field, total, position, pos)
    }

    fn as_number_printer_parser(&mut self) -> Option<&mut dyn NumberPrinterParser> {
        Some(self)
    }
}
impl NumberPrinterParser for NanosPrinterParser {
    fn get_field(&self) -> TemporalField {
        TemporalField::NanoOfSecond
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
        self.subsequent_width = subsequent_width
    }
}
