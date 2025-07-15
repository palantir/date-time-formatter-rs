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
    date_time_printer_parser::DateTimePrinterParser, temporal_field::TemporalField,
};

#[derive(Debug)]
pub struct OffsetIdPrinterParser {
    no_offset_text: &'static str,
    type_index: isize,
    style: isize,
}
impl OffsetIdPrinterParser {
    pub const PATTERNS: [&'static str; 22] = [
        "+HH",
        "+HHmm",
        "+HH:mm",
        "+HHMM",
        "+HH:MM",
        "+HHMMss",
        "+HH:MM:ss",
        "+HHMMSS",
        "+HH:MM:SS",
        "+HHmmss",
        "+HH:mm:ss",
        "+H",
        "+Hmm",
        "+H:mm",
        "+HMM",
        "+H:MM",
        "+HMMss",
        "+H:MM:ss",
        "+HMMSS",
        "+H:MM:SS",
        "+Hmmss",
        "+H:mm:ss",
    ];

    pub fn new(pattern: &str, no_offset_text: &'static str) -> Result<Self, String> {
        let type_index = OffsetIdPrinterParser::check_pattern(pattern)?;
        Ok(OffsetIdPrinterParser {
            no_offset_text,
            type_index,
            style: type_index % 11,
        })
    }

    pub fn check_pattern(pattern: &str) -> Result<isize, String> {
        for (i, &stored_pattern) in OffsetIdPrinterParser::PATTERNS.iter().enumerate() {
            if pattern == stored_pattern {
                return Ok(i as isize);
            }
        }
        Err(format!("Invalid zone offset pattern {pattern}"))
    }

    fn is_padded_hour(&self) -> bool {
        self.type_index < 11
    }

    fn is_colon(&self) -> bool {
        self.style > 0 && (self.style % 2) == 0
    }

    fn format_zero_pad(&self, colon: bool, value: i32, buf: &mut String) {
        if colon {
            buf.push(':');
        }
        buf.push(char::from_u32((value / 10) as u32 + '0' as u32).unwrap());
        buf.push(char::from_u32((value % 10) as u32 + '0' as u32).unwrap());
    }

    fn parse_hour(&self, parse_text: &str, padded_hour: bool, array: &mut [isize; 4]) {
        if padded_hour {
            // parse two digits
            if !self.parse_digits(parse_text, false, 1, array) {
                array[0] = !array[0];
            }
        } else {
            // parse one or two digits
            self.parse_variable_width_digits(parse_text, 1, 2, array);
        }
    }

    fn parse_minute(
        &self,
        parse_text: &str,
        is_colon: bool,
        mandatory: bool,
        array: &mut [isize; 4],
    ) {
        if !self.parse_digits(parse_text, is_colon, 2, array) && mandatory {
            array[0] = !array[0];
        }
    }

    fn parse_second(
        &self,
        parse_text: &str,
        is_colon: bool,
        mandatory: bool,
        array: &mut [isize; 4],
    ) {
        if !self.parse_digits(parse_text, is_colon, 3, array) && mandatory {
            array[0] = !array[0];
        }
    }

    fn parse_optional_minute_second(
        &self,
        parse_text: &str,
        is_colon: bool,
        array: &mut [isize; 4],
    ) {
        if self.parse_digits(parse_text, is_colon, 2, array) {
            self.parse_digits(parse_text, is_colon, 3, array);
        }
    }

    fn parse_digits(
        &self,
        parse_text: &str,
        is_colon: bool,
        array_index: isize,
        array: &mut [isize; 4],
    ) -> bool {
        let mut pos = array[0];
        if pos < 0 {
            return true;
        }

        if is_colon && array_index != 1 {
            // ':' will precede only in case of minute/second
            if pos + 1 > parse_text.len() as isize
                || parse_text.chars().nth(pos as usize).unwrap() != ':'
            {
                return false;
            }
            pos += 1;
        }

        if pos + 2 > parse_text.len() as isize {
            return false;
        }

        let chars: Vec<char> = parse_text.chars().collect();
        let ch1 = chars[pos as usize];
        let ch2 = chars[(pos + 1) as usize];
        pos += 2;

        if !ch1.is_ascii_digit() || !ch2.is_ascii_digit() {
            return false;
        }

        let value = (ch1 as isize - 48) * 10 + (ch2 as isize - 48);
        if !(0..=59).contains(&value) {
            return false;
        }

        array[array_index as usize] = value;
        array[0] = pos;
        true
    }

    fn parse_variable_width_digits(
        &self,
        parse_text: &str,
        min_digits: isize,
        max_digits: isize,
        array: &mut [isize; 4],
    ) {
        let mut pos = array[0];
        let mut available = 0;
        let mut chars = vec!['\0'; max_digits as usize];
        let text_chars: Vec<char> = parse_text.chars().collect();

        for i in 0..max_digits {
            if pos + 1 > parse_text.len() as isize {
                break;
            }
            let ch = text_chars[pos as usize];
            if !ch.is_ascii_digit() {
                break;
            }
            chars[i as usize] = ch;
            pos += 1;
            available += 1;
        }

        if available < min_digits {
            array[0] = !array[0];
            return;
        }

        match available {
            1 => {
                array[1] = chars[0] as isize - 48;
            }
            2 => {
                array[1] = (chars[0] as isize - 48) * 10 + (chars[1] as isize - 48);
            }
            3 => {
                array[1] = chars[0] as isize - 48;
                array[2] = (chars[1] as isize - 48) * 10 + (chars[2] as isize - 48);
            }
            4 => {
                array[1] = (chars[0] as isize - 48) * 10 + (chars[1] as isize - 48);
                array[2] = (chars[2] as isize - 48) * 10 + (chars[3] as isize - 48);
            }
            5 => {
                array[1] = chars[0] as isize - 48;
                array[2] = (chars[1] as isize - 48) * 10 + (chars[2] as isize - 48);
                array[3] = (chars[3] as isize - 48) * 10 + (chars[4] as isize - 48);
            }
            6 => {
                array[1] = (chars[0] as isize - 48) * 10 + (chars[1] as isize - 48);
                array[2] = (chars[2] as isize - 48) * 10 + (chars[3] as isize - 48);
                array[3] = (chars[4] as isize - 48) * 10 + (chars[5] as isize - 48);
            }
            _ => {}
        }
        array[0] = pos;
    }
}
impl DateTimePrinterParser for OffsetIdPrinterParser {
    fn format(&self, context: &mut DateTimePrintContext, buf: &mut String) -> Result<bool, String> {
        let offset_secs = match context.get_value(TemporalField::OffsetSeconds) {
            Some(value) => value,
            None => return Ok(false),
        };

        let total_secs = offset_secs as i32;
        if total_secs == 0 {
            buf.push_str(self.no_offset_text);
        } else {
            let abs_hours = (total_secs / 3600).abs() % 100; // anything larger than 99 silently dropped
            let abs_minutes = (total_secs / 60).abs() % 60;
            let abs_seconds = total_secs.abs() % 60;
            let buf_pos = buf.len();
            let mut output = abs_hours;

            if total_secs < 0 {
                buf.push('-');
            } else {
                buf.push('+');
            }

            if self.is_padded_hour() || abs_hours >= 10 {
                self.format_zero_pad(false, abs_hours, buf);
            } else {
                buf.push(char::from_u32(abs_hours as u32 + '0' as u32).unwrap());
            }

            if (self.style >= 3 && self.style <= 8)
                || (self.style >= 9 && abs_seconds > 0)
                || (self.style >= 1 && abs_minutes > 0)
            {
                self.format_zero_pad(self.is_colon(), abs_minutes, buf);
                output += abs_minutes;
                if self.style == 7 || self.style == 8 || (self.style >= 5 && abs_seconds > 0) {
                    self.format_zero_pad(self.is_colon(), abs_seconds, buf);
                    output += abs_seconds;
                }
            }

            if output == 0 {
                buf.truncate(buf_pos);
                buf.push_str(self.no_offset_text);
            }
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
        let no_offset_len = self.no_offset_text.len() as isize;

        if no_offset_len == 0 {
            if position == length {
                return context.set_parsed_field(
                    TemporalField::OffsetSeconds,
                    0,
                    position,
                    position,
                );
            }
        } else {
            if position == length {
                return Ok(-position - 1);
            }

            if context.sub_sequence_equals(
                text,
                position as usize,
                self.no_offset_text,
                0,
                no_offset_len as usize,
            ) {
                return context.set_parsed_field(
                    TemporalField::OffsetSeconds,
                    0,
                    position,
                    position + no_offset_len,
                );
            }
        }

        // parse normal plus/minus offset
        let chars: Vec<char> = text.chars().collect();
        let sign = chars.get(position as usize).unwrap();

        if *sign == '+' || *sign == '-' {
            // starts
            let negative = if *sign == '-' { -1 } else { 1 };
            let is_colon = self.is_colon();
            let padded_hour = self.is_padded_hour();
            let mut array = [position + 1, 0, 0, 0];
            let mut parse_type = self.type_index;

            // select parse type when lenient
            if !context.is_strict() {
                if padded_hour {
                    if is_colon
                        || (parse_type == 0
                            && length > position + 3
                            && chars[(position + 3) as usize] == ':')
                    {
                        parse_type = 10;
                    } else {
                        parse_type = 9;
                    }
                } else if is_colon
                    || (parse_type == 11
                        && length > position + 3
                        && (chars[(position + 2) as usize] == ':'
                            || chars[(position + 3) as usize] == ':'))
                {
                    parse_type = 21;
                } else {
                    parse_type = 20;
                }
            }

            // parse according to the selected pattern
            match parse_type {
                0 | 11 => {
                    // +HH or +H
                    self.parse_hour(text, padded_hour, &mut array);
                }
                1 | 2 | 13 => {
                    // +HHmm or +HH:mm or +H:mm
                    self.parse_hour(text, padded_hour, &mut array);
                    self.parse_minute(text, is_colon, false, &mut array);
                }
                3 | 4 | 15 => {
                    // +HHMM or +HH:MM or +H:MM
                    self.parse_hour(text, padded_hour, &mut array);
                    self.parse_minute(text, is_colon, true, &mut array);
                }
                5 | 6 | 17 => {
                    // +HHMMss or +HH:MM:ss or +H:MM:ss
                    self.parse_hour(text, padded_hour, &mut array);
                    self.parse_minute(text, is_colon, true, &mut array);
                    self.parse_second(text, is_colon, false, &mut array);
                }
                7 | 8 | 19 => {
                    // +HHMMSS or +HH:MM:SS or +H:MM:SS
                    self.parse_hour(text, padded_hour, &mut array);
                    self.parse_minute(text, is_colon, true, &mut array);
                    self.parse_second(text, is_colon, true, &mut array);
                }
                9 | 10 | 21 => {
                    // +HHmmss or +HH:mm:ss or +H:mm:ss
                    self.parse_hour(text, padded_hour, &mut array);
                    self.parse_optional_minute_second(text, is_colon, &mut array);
                }
                12 => {
                    // +Hmm
                    self.parse_variable_width_digits(text, 1, 4, &mut array);
                }
                14 => {
                    // +HMM
                    self.parse_variable_width_digits(text, 3, 4, &mut array);
                }
                16 => {
                    // +HMMss
                    self.parse_variable_width_digits(text, 3, 6, &mut array);
                }
                18 => {
                    // +HMMSS
                    self.parse_variable_width_digits(text, 5, 6, &mut array);
                }
                20 => {
                    // +Hmmss
                    self.parse_variable_width_digits(text, 1, 6, &mut array);
                }
                _ => {}
            }

            if array[0] > 0 {
                if array[1] > 23 || array[2] > 59 || array[3] > 59 {
                    return Err(
                        "Value out of range: Hour[0-23], Minute[0-59], Second[0-59]".to_owned()
                    );
                }
                let offset_secs =
                    negative * (array[1] as i64 * 3600 + array[2] as i64 * 60 + array[3] as i64);
                return context.set_parsed_field(
                    TemporalField::OffsetSeconds,
                    offset_secs,
                    position,
                    array[0],
                );
            }
        }

        // handle special case of empty no offset text
        if no_offset_len == 0 {
            return context.set_parsed_field(TemporalField::OffsetSeconds, 0, position, position);
        }

        Ok(-position - 1)
    }
}
