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
    date_time_printer_parser::DateTimePrinterParser, sign_style::SignStyle,
    temporal_field::TemporalField,
};

pub(crate) const EXCEED_POINTS: [i64; 11] = [
    0,
    10,
    100,
    1000,
    10000,
    100000,
    1000000,
    10000000,
    100000000,
    1000000000,
    10000000000,
];

pub(crate) fn string_size(x: i64) -> i32 {
    // Taken from DecimalDigits
    let mut x = x;
    let mut d = 1;
    if x >= 0 {
        d = 0;
        x = -x;
    }
    let mut p = -10;
    for i in 1..19 {
        if x > p {
            return i + d;
        }
        p *= 10;
    }
    19 + d
}

pub trait NumberPrinterParser: DateTimePrinterParser {
    fn get_field(&self) -> TemporalField;
    fn get_min_width(&self) -> isize;
    fn get_max_width(&self) -> isize;
    fn get_sign_style(&self) -> SignStyle;
    fn get_subsequent_width(&self) -> isize;
    fn set_subsequent_width(&mut self, subsequent_width: isize);
}

/// In Java, NumberPrinterParser implementes DateTimePrinterParser and then other classes
/// extend NumberPrinterParser, overriding some of its methods as necessary.
/// That same hierarchy isn't really possible in Rust, but this emulates is closely enough.
/// 1. Anything that in Java extends NumberPrinterParser implements that trait here as well.
/// 2. Anything that in Java uses NumberPrinterParser's implementation of format/parse implements
///    this trait in addition.
pub trait DefaultNumberPrinterParser: NumberPrinterParser {
    fn get_value(&self, context: &DateTimePrintContext, value: i64) -> i64;
    fn set_value(
        &self,
        context: &mut DateTimeParseContext,
        value: i64,
        error_pos: isize,
        success_pos: isize,
    ) -> Result<isize, String>;

    fn is_fixed_width(&self, _context: &DateTimeParseContext) -> bool {
        self.get_subsequent_width() == -1
            || self.get_subsequent_width() > 0
                && self.get_min_width() == self.get_max_width()
                && self.get_sign_style() == SignStyle::NotNegative
    }
}

impl<T: DefaultNumberPrinterParser> DateTimePrinterParser for T {
    fn format(&self, context: &mut DateTimePrintContext, buf: &mut String) -> Result<bool, String> {
        let value = match context.get_value(self.get_field()) {
            Some(value) => self.get_value(context, value),
            None => return Ok(false),
        };
        let decimal_style = context.get_formatter().get_decimal_style();

        // Calculate size of the digits
        let mut size = string_size(value);
        if value < 0 {
            size -= 1; // Adjust for the negative sign, which is handled separately
        }

        if size > self.get_max_width() as i32 {
            return Err(format!(
                "Field {:?} cannot be printed as the value {} exceeds the maximum print width of {}",
                self.get_field(),
                value,
                self.get_max_width()
            ));
        }

        // Handle sign
        if value >= 0 {
            match self.get_sign_style() {
                SignStyle::ExceedsPad => {
                    if self.get_min_width() < 19 && size > self.get_min_width() as i32 {
                        buf.push(decimal_style.get_positive_sign());
                    }
                }
                SignStyle::Always => {
                    buf.push(decimal_style.get_positive_sign());
                }
                _ => {} // No sign for positive numbers in other cases
            }
        } else {
            match self.get_sign_style() {
                SignStyle::Normal | SignStyle::ExceedsPad | SignStyle::Always => {
                    buf.push(decimal_style.get_negative_sign());
                }
                SignStyle::NotNegative => {
                    return Err(format!(
                        "Field {:?} cannot be printed as the value {} cannot be negative according to the SignStyle",
                        self.get_field(),
                        value
                    ));
                }
                _ => {} // Handle other cases as needed
            }
        }

        // Add leading zeros if necessary
        let zero_digit = decimal_style.get_zero_digit();
        let zeros = self.get_min_width() as i32 - size;
        if zeros > 0 {
            for _ in 0..zeros {
                buf.push(zero_digit);
            }
        }

        // Add the number itself
        if zero_digit == '0' && value != i64::MIN {
            // For standard ASCII digits, we can use Rust's built-in formatting
            buf.push_str(&format!("{}", value.abs()));
        } else {
            // For non-standard digits, we need to convert each digit
            let abs_value = if value == i64::MIN {
                "9223372036854775808".to_string() // i64::MIN without the negative sign
            } else {
                value.abs().to_string()
            };

            buf.push_str(&decimal_style.convert_number_to_i18n(&abs_value)?);
        }

        Ok(true)
    }

    fn parse(
        &self,
        context: &mut DateTimeParseContext,
        text: &str,
        position: isize,
    ) -> Result<isize, String> {
        let length = text.len() as isize;
        if position == length {
            return Ok(!position);
        }

        let text_chars: Vec<char> = text.chars().collect();
        if position < 0 || position >= text_chars.len() as isize {
            return Err("Index out of bounds".to_owned());
        }

        let sign = text_chars[position as usize]; // Will panic if position is invalid
        let mut negative = false;
        let mut positive = false;

        let mut position = position;

        if sign == context.get_decimal_style().get_positive_sign() {
            if !self.get_sign_style().parse(
                true,
                context.is_strict(),
                self.get_min_width() == self.get_max_width(),
            ) {
                return Ok(!position);
            }
            positive = true;
            position += 1;
        } else if sign == context.get_decimal_style().get_negative_sign() {
            if !self.get_sign_style().parse(
                false,
                context.is_strict(),
                self.get_min_width() == self.get_max_width(),
            ) {
                return Ok(!position);
            }
            negative = true;
            position += 1;
        } else if self.get_sign_style() == SignStyle::Always && context.is_strict() {
            return Ok(!position);
        }

        // Determine effective min/max width
        let eff_min_width = if context.is_strict() || self.is_fixed_width(context) {
            self.get_min_width()
        } else {
            1
        };
        let min_end_pos = position + eff_min_width;

        if min_end_pos > length {
            return Ok(!position);
        }

        let mut eff_max_width = if context.is_strict() || self.is_fixed_width(context) {
            self.get_max_width()
        } else {
            9
        } + std::cmp::max(self.get_subsequent_width(), 0);

        let mut total: i64 = 0;
        let mut total_big: Option<num_bigint::BigInt> = None;
        let mut pos = position;

        // Main parsing loop
        for pass in 0..2 {
            let max_end_pos = std::cmp::min(pos + eff_max_width, length);

            while pos < max_end_pos {
                if pos >= text_chars.len() as isize {
                    break;
                }

                let ch = text_chars[pos as usize];
                pos += 1;

                let digit = context.get_decimal_style().convert_to_digit(ch);

                if digit < 0 {
                    pos -= 1;
                    if pos < min_end_pos {
                        return Ok(!position); // Need at least min width digits
                    }
                    break;
                }

                if (pos - position) > 18 {
                    // Switch to big integer for large numbers
                    if total_big.is_none() {
                        total_big = Some(num_bigint::BigInt::from(total));
                    }

                    if let Some(ref mut big) = total_big {
                        *big = big.clone() * 10 + digit;
                    }
                } else {
                    total = total * 10 + digit;
                }
            }

            if self.get_subsequent_width() > 0 && pass == 0 {
                // Re-parse now we know the correct width
                let parse_len = pos - position;
                eff_max_width =
                    std::cmp::max(eff_min_width, parse_len - self.get_subsequent_width());
                pos = position;
                total = 0;
                total_big = None;
            } else {
                break;
            }
        }

        // Handle negative numbers and special cases
        if negative {
            if let Some(ref mut big) = total_big {
                if *big == num_bigint::BigInt::from(0) && context.is_strict() {
                    return Ok(!(position - 1)); // minus zero not allowed in strict mode
                }
                *big = -big.clone();
            } else {
                if total == 0 && context.is_strict() {
                    return Ok(!(position - 1)); // minus zero not allowed in strict mode
                }
                total = -total;
            }
        } else if self.get_sign_style() == SignStyle::ExceedsPad && context.is_strict() {
            let parse_len = pos - position;
            if positive {
                if parse_len <= self.get_min_width() {
                    return Ok(!(position - 1)); // '+' only parsed if minWidth exceeded
                }
            } else if parse_len > self.get_min_width() {
                return Ok(!position); // '+' must be parsed if minWidth exceeded
            }
        }

        // Check for overflow and set the final value
        if let Some(total_big) = total_big {
            let mut total_big = total_big;
            // Check if the number can fit in an i64
            if total_big.bits() > 63 {
                // Overflow, parse 1 less digit
                total_big /= 10;
                pos -= 1;
            }
            let total_big = match i64::try_from(total_big) {
                Ok(v) => v,
                Err(e) => return Err(format!("Could not fix in i64, {e}")), // Can't fit in i64
            };
            return self.set_value(context, total_big, position, pos);
        }

        self.set_value(context, total, position, pos)
    }

    fn as_number_printer_parser(&mut self) -> Option<&mut dyn NumberPrinterParser> {
        Some(self)
    }
}

/// This impl corresponds to the "NumberPrinterParser" class in Java when it is initialized directly.
#[derive(Debug)]
pub struct BaseNumberPrinterParser {
    field: TemporalField,
    min_width: isize,
    max_width: isize,
    sign_style: SignStyle,
    subsequent_width: isize,
}
impl BaseNumberPrinterParser {
    pub fn new(
        field: TemporalField,
        min_width: isize,
        max_width: isize,
        sign_style: SignStyle,
    ) -> Self {
        BaseNumberPrinterParser {
            field,
            min_width,
            max_width,
            sign_style,
            subsequent_width: 0,
        }
    }
}

impl NumberPrinterParser for BaseNumberPrinterParser {
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

impl DefaultNumberPrinterParser for BaseNumberPrinterParser {
    fn get_value(&self, _context: &DateTimePrintContext, value: i64) -> i64 {
        value
    }

    fn set_value(
        &self,
        context: &mut DateTimeParseContext,
        value: i64,
        error_pos: isize,
        success_pos: isize,
    ) -> Result<isize, String> {
        context.set_parsed_field(self.field, value, error_pos, success_pos)
    }
}
