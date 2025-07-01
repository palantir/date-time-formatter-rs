use crate::{
    date_time_print_context::DateTimePrintContext, date_time_printer_parser::DateTimePrinterParser,
    temporal_field::TemporalField, text_style::TextStyle,
};

#[derive(Debug)]
pub struct LocalizedOffsetIdPrinterParser {
    style: TextStyle,
}
impl LocalizedOffsetIdPrinterParser {
    pub fn new(style: TextStyle) -> Self {
        LocalizedOffsetIdPrinterParser { style }
    }

    fn append_hms(buf: &mut String, t: i32) {
        buf.push(((t / 10) as u8 + b'0') as char);
        buf.push(((t % 10) as u8 + b'0') as char);
    }

    fn get_digit(text: &str, position: usize) -> isize {
        if position >= text.len() {
            return -1;
        }
        let c = text.chars().nth(position).unwrap();
        if !c.is_ascii_digit() {
            return -1;
        }
        (c as u32 - '0' as u32) as isize
    }
}

impl DateTimePrinterParser for LocalizedOffsetIdPrinterParser {
    fn format(&self, context: &mut DateTimePrintContext, buf: &mut String) -> Result<bool, String> {
        let offset_secs = match context.get_value(TemporalField::OffsetSeconds) {
            Some(value) => value,
            None => return Ok(false),
        };

        // Default to "GMT" if no localized resource is available
        // We don't support Locales so this should be fine
        let gmt_text = "GMT";
        buf.push_str(gmt_text);

        let total_secs = offset_secs as i32;
        if total_secs != 0 {
            let abs_hours = (total_secs / 3600).abs() % 100; // anything larger than 99 silently dropped
            let abs_minutes = (total_secs / 60).abs() % 60;
            let abs_seconds = total_secs.abs() % 60;

            if total_secs < 0 {
                buf.push('-');
            } else {
                buf.push('+');
            }

            if self.style == TextStyle::Full {
                Self::append_hms(buf, abs_hours);
                buf.push(':');
                Self::append_hms(buf, abs_minutes);
                if abs_seconds != 0 {
                    buf.push(':');
                    Self::append_hms(buf, abs_seconds);
                }
            } else {
                if abs_hours >= 10 {
                    buf.push(((abs_hours / 10) as u8 + b'0') as char);
                }
                buf.push(((abs_hours % 10) as u8 + b'0') as char);
                if abs_minutes != 0 || abs_seconds != 0 {
                    buf.push(':');
                    Self::append_hms(buf, abs_minutes);
                    if abs_seconds != 0 {
                        buf.push(':');
                        Self::append_hms(buf, abs_seconds);
                    }
                }
            }
        }
        Ok(true)
    }

    fn parse(
        &self,
        context: &mut crate::date_time_parse_context::DateTimeParseContext,
        text: &str,
        position: isize,
    ) -> Result<isize, String> {
        let mut pos = position as usize;
        let end = text.len();

        // Default to "GMT" as we don't support localized resources
        let gmt_text = "GMT";

        if !context.sub_sequence_equals(text, pos, gmt_text, 0, gmt_text.len()) {
            return Ok(!position);
        }

        pos += gmt_text.len();

        // parse normal plus/minus offset
        let negative;
        if pos == end {
            return context.set_parsed_field(
                TemporalField::OffsetSeconds,
                0,
                position,
                pos as isize,
            );
        }

        // Get the sign character
        let sign = text.chars().nth(pos).unwrap();
        if sign == '+' {
            negative = 1;
        } else if sign == '-' {
            negative = -1;
        } else {
            return context.set_parsed_field(
                TemporalField::OffsetSeconds,
                0,
                position,
                pos as isize,
            );
        }
        pos += 1;

        let mut h;
        let mut m = 0;
        let mut s = 0;

        if self.style == TextStyle::Full {
            let h1 = Self::get_digit(text, pos);
            pos += 1;
            let h2 = Self::get_digit(text, pos);
            pos += 1;

            // Check if we have valid hours and the expected colon
            if h1 < 0 || h2 < 0 || pos >= end || text.chars().nth(pos).unwrap() != ':' {
                return Ok(!position);
            }
            pos += 1;

            h = h1 * 10 + h2;

            let m1 = Self::get_digit(text, pos);
            pos += 1;
            let m2 = Self::get_digit(text, pos);
            pos += 1;

            if m1 < 0 || m2 < 0 {
                return Ok(!position);
            }

            m = m1 * 10 + m2;

            // Check for seconds
            if pos + 2 < end && text.chars().nth(pos).unwrap() == ':' {
                let s1 = Self::get_digit(text, pos + 1);
                let s2 = Self::get_digit(text, pos + 2);

                if s1 >= 0 && s2 >= 0 {
                    s = s1 * 10 + s2;
                    pos += 3;
                }
            }
        } else {
            let h1 = Self::get_digit(text, pos);
            pos += 1;

            if h1 < 0 {
                return Ok(!position);
            }

            h = h1;

            if pos < end {
                let h2 = Self::get_digit(text, pos);
                if h2 >= 0 {
                    h = h * 10 + h2;
                    pos += 1;
                }

                // Check for minutes
                if pos + 2 < end && text.chars().nth(pos).unwrap() == ':' {
                    let m1 = Self::get_digit(text, pos + 1);
                    let m2 = Self::get_digit(text, pos + 2);

                    if m1 >= 0 && m2 >= 0 {
                        m = m1 * 10 + m2;
                        pos += 3;

                        // Check for seconds
                        if pos + 2 < end && text.chars().nth(pos).unwrap() == ':' {
                            let s1 = Self::get_digit(text, pos + 1);
                            let s2 = Self::get_digit(text, pos + 2);

                            if s1 >= 0 && s2 >= 0 {
                                s = s1 * 10 + s2;
                                pos += 3;
                            }
                        }
                    }
                }
            }
        }

        let offset_secs = negative * (h * 3600 + m * 60 + s) as i64;
        context.set_parsed_field(
            TemporalField::OffsetSeconds,
            offset_secs,
            position,
            pos as isize,
        )
    }
}
