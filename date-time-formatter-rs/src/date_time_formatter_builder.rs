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

use std::{cmp::Ordering, collections::HashSet, mem};

use crate::{
    char_literal_printer_parser::CharLiteralPrinterParser,
    composite_printer_parser::CompositePrinterParser,
    date_time_formatter::DateTimeFormatter,
    date_time_printer_parser::DateTimePrinterParser,
    day_period_printer_parser::DayPeriodPrinterParser,
    decimal_style::DecimalStyle,
    default_value_parser::DefaultValueParser,
    localized_offset_id_printer_parser::LocalizedOffsetIdPrinterParser,
    nanos_printer_parser::NanosPrinterParser,
    number_printer_parser::{BaseNumberPrinterParser, NumberPrinterParser},
    offset_id_printer_parser::OffsetIdPrinterParser,
    pad_printer_parser_decorator::PadPrinterParserDecorator,
    reduced_printer_parser::ReducedPrinterParser,
    resolver_style::ResolverStyle,
    sign_style::SignStyle,
    string_literal_printer_parser::StringLiteralPrinterParser,
    temporal_field::{get_temporal_field, TemporalField},
    text_printer_parser::TextPrinterParser,
    text_style::TextStyle,
    week_based_field_printer_parser::WeekBasedFieldPrinterParser,
    zone_id_printer_parser::ZoneIdPrinterParser,
    zone_text_printer_parser::ZoneTextPrinterParser,
};

pub struct DateTimeFormatterBuilder {
    active: Option<*mut DateTimeFormatterBuilder>,
    parent: Option<*mut DateTimeFormatterBuilder>,

    printer_parsers: Vec<Box<dyn DateTimePrinterParser>>,
    optional: bool,
    pad_next_width: isize,
    pad_next_char: Option<char>,
    value_parser_index: isize,
}

impl DateTimeFormatterBuilder {
    pub fn new() -> Self {
        let mut builder = DateTimeFormatterBuilder {
            active: None,
            parent: None,
            printer_parsers: vec![],
            optional: false,
            pad_next_width: 0,
            pad_next_char: None,
            value_parser_index: -1,
        };

        builder.active = Some(&mut builder);

        builder
    }

    fn get_active_mut(&mut self) -> &mut DateTimeFormatterBuilder {
        // TODO: Fix this when supporting optional sections
        self
    }

    pub fn create_formatter(mut self) -> DateTimeFormatter {
        // Create and return a DateTimeFormatter based on the builder state
        let composite_printer_parser =
            CompositePrinterParser::new(std::mem::take(&mut self.printer_parsers), false);
        DateTimeFormatter::new(
            composite_printer_parser,
            DecimalStyle::STANDARD,
            ResolverStyle::Smart,
            HashSet::new(), // TODO: Not sure if this should actually be empty
        )
    }

    pub fn parse_defaulting(&mut self, field: TemporalField, value: i64) -> Result<(), String> {
        self.append_internal(Box::new(DefaultValueParser::new(field, value)));
        Ok(())
    }

    pub fn parse_pattern(&mut self, pattern: &str) -> Result<(), String> {
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let mut pos = 0;

        while pos < pattern_chars.len() {
            let mut cur = pattern_chars[pos];

            if cur.is_ascii_uppercase() || cur.is_ascii_lowercase() {
                let mut start = pos;
                pos += 1;

                // Count consecutive occurrences of the same character
                while pos < pattern_chars.len() && pattern_chars[pos] == cur {
                    pos += 1;
                }

                let mut count = (pos - start) as isize;

                // padding
                if cur == 'p' {
                    let mut pad: isize = 0;
                    if pos < pattern_chars.len() {
                        cur = pattern_chars[pos];
                        if cur.is_ascii_uppercase() || cur.is_ascii_lowercase() {
                            pad = count;
                            start = pos;
                            pos += 1;

                            while pos < pattern_chars.len() && pattern_chars[pos] == cur {
                                pos += 1;
                            }

                            count = (pos - start) as isize;
                        }
                    }

                    if pad == 0 {
                        return Err(format!(
                            "Pad letter 'p' must be followed by valid pad pattern: {}",
                            pattern
                        ));
                    }

                    self.pad_next(pad); // pad and continue parsing
                }

                // main rules
                if let Some(field) = get_temporal_field(&cur) {
                    self.parse_field(cur, count, field)?;
                } else if cur == 'z' {
                    match count.cmp(&4) {
                        Ordering::Less => self.append_zone_text(TextStyle::Short),
                        Ordering::Equal => self.append_zone_text(TextStyle::Full),
                        Ordering::Greater => {
                            return Err(format!("Too many pattern letters: {}", cur))
                        }
                    }
                } else if cur == 'V' {
                    if count != 2 {
                        return Err(format!("Pattern letter count must be 2: {}", cur));
                    }
                    self.append_zone_id();
                } else if cur == 'v' {
                    match count {
                        1 => {
                            self.append_generic_zone_text(TextStyle::Short);
                        }
                        4 => {
                            self.append_generic_zone_text(TextStyle::Full);
                        }
                        _ => return Err(format!("Wrong number of pattern letters: {}", cur)),
                    }
                } else if cur == 'Z' {
                    if count < 4 {
                        self.append_offset("+HHMM", "+0000")?;
                    } else if count == 4 {
                        self.append_localized_offset(TextStyle::Full)?;
                    } else if count == 5 {
                        self.append_offset("+HH:MM:ss", "Z")?;
                    } else {
                        return Err(format!("Too many pattern letters: {}", cur));
                    }
                } else if cur == 'O' {
                    match count {
                        1 => {
                            self.append_localized_offset(TextStyle::Short)?;
                        }
                        4 => {
                            self.append_localized_offset(TextStyle::Full)?;
                        }
                        _ => return Err(format!("Pattern letter count must be 1 or 4: {}", cur)),
                    }
                } else if cur == 'X' {
                    if count > 5 {
                        return Err(format!("Too many pattern letters: {}", cur));
                    }
                    let pattern_index = count + if count == 1 { 0 } else { 1 };
                    let pattern = OffsetIdPrinterParser::PATTERNS[pattern_index as usize];
                    self.append_offset(pattern, "Z")?;
                } else if cur == 'x' {
                    if count > 5 {
                        return Err(format!("Too many pattern letters: {}", cur));
                    }
                    let zero = if count == 1 {
                        "+00"
                    } else if count % 2 == 0 {
                        "+0000"
                    } else {
                        "+00:00"
                    };
                    let pattern_index = count + if count == 1 { 0 } else { 1 };
                    self.append_offset(
                        OffsetIdPrinterParser::PATTERNS[pattern_index as usize],
                        zero,
                    )?;
                } else if cur == 'W' {
                    if count > 1 {
                        return Err(format!("Too many pattern letters: {}", cur));
                    }
                    self.append_value_with_number_printer_parser(Box::new(
                        WeekBasedFieldPrinterParser::new(cur, count, count, count),
                    ))?;
                } else if cur == 'w' {
                    if count > 2 {
                        return Err(format!("Too many pattern letters: {}", cur));
                    }
                    self.append_value_with_number_printer_parser(Box::new(
                        WeekBasedFieldPrinterParser::new(cur, count, count, 2),
                    ))?;
                } else if cur == 'Y' {
                    let max_width = if count == 2 { 2 } else { 19 };
                    self.append_value_with_number_printer_parser(Box::new(
                        WeekBasedFieldPrinterParser::new(cur, count, count, max_width),
                    ))?;
                } else if cur == 'B' {
                    match count {
                        1 => {
                            self.append_day_period_text(TextStyle::Short);
                        }
                        4 => {
                            self.append_day_period_text(TextStyle::Full);
                        }
                        5 => {
                            self.append_day_period_text(TextStyle::Narrow);
                        }
                        _ => return Err(format!("Wrong number of pattern letters: {}", cur)),
                    }
                } else {
                    return Err(format!("Unknown pattern letter: {}", cur));
                }

                pos -= 1;
            } else if cur == '\'' {
                // parse literals
                let start = pos;
                pos += 1;

                while pos < pattern_chars.len() {
                    if pattern_chars[pos] == '\'' {
                        if pos + 1 < pattern_chars.len() && pattern_chars[pos + 1] == '\'' {
                            pos += 1;
                        } else {
                            break; // end of literal
                        }
                    }
                    pos += 1;
                }

                if pos >= pattern_chars.len() {
                    return Err(format!(
                        "Pattern ends with an incomplete string literal: {}",
                        pattern
                    ));
                }

                let str_literal: String = pattern_chars[(start + 1)..pos].iter().collect();

                if str_literal.is_empty() {
                    self.append_literal_char('\'');
                } else {
                    self.append_literal_string(&str_literal.replace("''", "'"));
                }
            } else if cur == '[' {
                self.optional_start();
            } else if cur == ']' {
                if self.get_active_mut().parent.is_none() {
                    return Err(String::from(
                        "Pattern invalid as it contains ] without previous [",
                    ));
                }
                self.optional_end()?;
            } else if cur == '{' || cur == '}' || cur == '#' {
                return Err(format!("Pattern includes reserved character: '{}'", cur));
            } else {
                self.append_literal_char(cur);
            }

            pos += 1;
        }

        Ok(())
    }

    fn parse_field(&mut self, cur: char, count: isize, field: TemporalField) -> Result<(), String> {
        let mut standalone = false;
        match cur {
            'u' | 'y' => {
                if count == 2 {
                    self.append_value_reduced(field, 2, 2, ReducedPrinterParser::BASE_VALUE)?;
                } else if count < 4 {
                    self.append_value_with_field_min_width_max_width_and_sign_style(field, count, 19, SignStyle::Normal)?;
                } else {
                    self.append_value_with_field_min_width_max_width_and_sign_style(field, count, 19, SignStyle::ExceedsPad)?;
                }
            }
            'c' | // Need to handle separately
            'L' | 'q' | // Need to handle separately
            'M' | 'Q' | 'E' | 'e' => {
                if cur == 'c' {
                    if count == 1 {
                        self.append_value_with_number_printer_parser(Box::new(WeekBasedFieldPrinterParser::new(cur, count, count, count)))?;
                        return Ok(());
                    } else if count == 2 {
                        return Err("Invalid pattern \'cc\'".to_owned())
                    }
                }
                if cur == 'L' || cur == 'q' {
                    standalone = true;
                }
                match count {
                1 | 2 => {
                    if cur == 'e' {
                        self.append_value_with_number_printer_parser(Box::new(
                            WeekBasedFieldPrinterParser::new(cur, count, count, count),
                        ))?;
                    } else if cur == 'E' {
                        self.append_text_with_field_and_text_style(field, TextStyle::Short);
                    } else if count == 1 {
                        self.append_value_with_field(field)?;
                    } else {
                        self.append_value_with_field_and_width(field, 2)?;
                    }
                }
                3 => {
                    self.append_text_with_field_and_text_style(
                        field,
                        if standalone {
                            TextStyle::ShortStandalone
                        } else {
                            TextStyle::Short
                        },
                    );
                }
                4 => {
                    self.append_text_with_field_and_text_style(
                        field,
                        if standalone {
                            TextStyle::FullStandalone
                        } else {
                            TextStyle::Full
                        },
                    );
                }
                5 => {
                    self.append_text_with_field_and_text_style(
                        field,
                        if standalone {
                            TextStyle::NarrowStandalone
                        } else {
                            TextStyle::Narrow
                        },
                    );
                }
                _ => {
                    return Err(format!("Too many pattern letters: {}", cur));
                }
            }},
            'a' => {
                if count == 1 {
                    self.append_text_with_field_and_text_style(field, TextStyle::Short);
                } else {
                    return Err(format!("Too many pattern letters: {}", cur));
                }
            }
            'G' => match count {
                1..=3 => self.append_text_with_field_and_text_style(field, TextStyle::Short),
                4 => self.append_text_with_field_and_text_style(field, TextStyle::Full),
                5 => self.append_text_with_field_and_text_style(field, TextStyle::Narrow),
                _ => return Err(format!("Too many pattern letters: {}", cur)),
            },
            'S' => {
                self.append_fraction(TemporalField::NanoOfSecond, count, count, false)?;
            }
            'F' => {
                if count == 1 {
                    self.append_value_with_field(field)?;
                } else {
                    return Err(format!("Too many pattern letters: {}", cur));
                }
            }
            'd' | 'h' | 'H' | 'k' | 'K' | 'm' | 's' => {
                if count == 1 {
                    self.append_value_with_field(field)?;
                } else if count == 2 {
                    self.append_value_with_field_and_width(field, count)?;
                } else {
                    return Err(format!("Too many pattern letters: {}", cur));
                }
            }
            'D' => {
                if count == 1 {
                    self.append_value_with_field(field)?;
                } else if count == 2 || count == 3 {
                    self.append_value_with_field_min_width_max_width_and_sign_style(field, count, 3, SignStyle::NotNegative)?;
                } else {
                    return Err(format!("Too many pattern letters: {}", cur));
                }
            }
            'g' => {
                self.append_value_with_field_min_width_max_width_and_sign_style(field, count, 19, SignStyle::Normal)?;
            }
            'A' | 'n' | 'N' => {
                self.append_value_with_field_min_width_max_width_and_sign_style(field, count, 19, SignStyle::NotNegative)?;
            }
            _ => {
                if count == 1 {
                    self.append_value_with_field(field)?;
                } else {
                    self.append_value_with_field_and_width(field, count)?;
                }
            }
        }
        Ok(())
    }

    fn pad_next(&mut self, pad_width: isize) {
        self.get_active_mut().pad_next_width = pad_width;
        self.get_active_mut().pad_next_char = Some(' ');
        self.get_active_mut().value_parser_index = -1;
    }

    fn append_zone_id(&mut self) {
        self.append_internal(Box::new(ZoneIdPrinterParser::new()));
    }

    fn append_zone_text(&mut self, text_style: TextStyle) {
        self.append_internal(Box::new(ZoneTextPrinterParser::new(text_style, false)));
    }

    fn append_generic_zone_text(&mut self, text_style: TextStyle) {
        self.append_internal(Box::new(ZoneTextPrinterParser::new(text_style, true)));
    }

    fn append_offset(&mut self, pattern: &str, no_offset_text: &'static str) -> Result<(), String> {
        self.append_internal(Box::new(OffsetIdPrinterParser::new(
            pattern,
            no_offset_text,
        )?));
        Ok(())
    }

    fn append_localized_offset(&mut self, style: TextStyle) -> Result<(), String> {
        if !(style == TextStyle::Full || style == TextStyle::Short) {
            return Err("Style must be either full or short".to_owned());
        }
        self.append_internal(Box::new(LocalizedOffsetIdPrinterParser::new(style)));
        Ok(())
    }

    fn append_day_period_text(&mut self, style: TextStyle) {
        let style = match style {
            TextStyle::Full | TextStyle::Short | TextStyle::Narrow => style,
            TextStyle::FullStandalone => TextStyle::Full,
            TextStyle::ShortStandalone => TextStyle::Short,
            TextStyle::NarrowStandalone => TextStyle::Narrow,
        };
        self.append_internal(Box::new(DayPeriodPrinterParser::new(style)));
    }

    fn append_value_with_field(&mut self, field: TemporalField) -> Result<(), String> {
        self.append_value_with_number_printer_parser(Box::new(BaseNumberPrinterParser::new(
            field,
            1,
            19,
            SignStyle::Normal,
        )))
    }

    fn append_value_with_field_and_width(
        &mut self,
        field: TemporalField,
        width: isize,
    ) -> Result<(), String> {
        if !(1..=19).contains(&width) {
            return Err(format!(
                "The width must be from 1 to 19 inclusive but was {}",
                width
            ));
        }
        self.append_value_with_number_printer_parser(Box::new(BaseNumberPrinterParser::new(
            field,
            width,
            width,
            SignStyle::NotNegative,
        )))
    }

    fn append_value_with_field_min_width_max_width_and_sign_style(
        &mut self,
        field: TemporalField,
        min_width: isize,
        max_width: isize,
        sign_style: SignStyle,
    ) -> Result<(), String> {
        if min_width == max_width && sign_style == SignStyle::NotNegative {
            return self.append_value_with_field_and_width(field, max_width);
        }
        if !(1..=19).contains(&min_width) {
            return Err(format!(
                "The minimum width must be from 1 to 19 inclusive but was {}",
                min_width
            ));
        }
        if !(1..=19).contains(&max_width) {
            return Err(format!(
                "The maximum width must be from 1 to 19 inclusive but was {}",
                max_width
            ));
        }
        if max_width < min_width {
            return Err(format!(
                "The maximum width must exceed or equal the minimum width but {} < {}",
                max_width, min_width
            ));
        }
        self.append_value_with_number_printer_parser(Box::new(BaseNumberPrinterParser::new(
            field, min_width, max_width, sign_style,
        )))
    }

    fn append_value_reduced(
        &mut self,
        field: TemporalField,
        width: isize,
        max_width: isize,
        base_value: i64,
    ) -> Result<(), String> {
        self.append_value_with_number_printer_parser(Box::new(ReducedPrinterParser::new(
            field, width, max_width, base_value,
        )))
    }

    fn append_value_with_number_printer_parser(
        &mut self,
        mut pp: Box<dyn NumberPrinterParser>,
    ) -> Result<(), String> {
        if self.get_active_mut().value_parser_index >= 0 {
            let active_value_parser = self.get_active_mut().value_parser_index;

            // Get a reference to the existing printer parser
            let base_pp_ref =
                &mut self.get_active_mut().printer_parsers[active_value_parser as usize];

            // Use dynamic dispatch to check if it's a NumberPrinterParser
            if let Some(base_pp) = base_pp_ref.as_number_printer_parser() {
                if pp.get_min_width() == pp.get_max_width()
                    && pp.get_sign_style() == SignStyle::NotNegative
                {
                    // Modify the base printer parser
                    base_pp
                        .set_subsequent_width(base_pp.get_subsequent_width() + pp.get_max_width());

                    pp.set_subsequent_width(-1);

                    self.append_internal(pp);

                    self.get_active_mut().value_parser_index = active_value_parser;
                } else {
                    base_pp.set_subsequent_width(-1);
                    self.get_active_mut().value_parser_index = self.append_internal(pp);
                }
            } else {
                return Err(format!(
                    "Tried to cast {:?} to dyn NumberPrinterParser, found at value parser index",
                    base_pp_ref
                ));
            }
        } else {
            self.get_active_mut().value_parser_index = self.append_internal(pp);
        }
        Ok(())
    }

    fn append_literal_char(&mut self, literal: char) {
        self.append_internal(Box::new(CharLiteralPrinterParser::new(literal)));
    }

    fn append_literal_string(&mut self, literal: &str) {
        let length = literal.len();
        if length > 0 {
            if length == 1 {
                self.append_internal(Box::new(CharLiteralPrinterParser::new(
                    literal.chars().nth(0).unwrap(),
                )));
            } else {
                self.append_internal(Box::new(StringLiteralPrinterParser::new(
                    literal.to_owned(),
                )));
            }
        }
    }

    fn append_fraction(
        &mut self,
        field: TemporalField,
        min_width: isize,
        max_width: isize,
        decimal_point: bool,
    ) -> Result<(), String> {
        match field {
            TemporalField::NanoOfSecond => {
                if min_width == max_width && !decimal_point {
                    self.append_value_with_number_printer_parser(Box::new(
                        NanosPrinterParser::new(min_width, max_width, decimal_point)?,
                    ))?;
                } else {
                    self.append_internal(Box::new(NanosPrinterParser::new(
                        min_width,
                        max_width,
                        decimal_point,
                    )?));
                }
            }
            _ => {
                return Err(
                    "append_fraction should never be called with a field besides NanoOfSecond"
                        .to_owned(),
                );
            }
        }
        Ok(())
    }

    fn optional_start(&mut self) {
        // Set value_parser_index of current active builder to -1
        self.get_active_mut().value_parser_index = -1;

        let active_ptr = self.get_active_mut();

        // Create a new builder on the heap that won't be dropped when this function returns
        let new_builder = Box::new(DateTimeFormatterBuilder {
            active: None,
            parent: Some(active_ptr),
            printer_parsers: vec![],
            optional: true,
            pad_next_width: 0,
            pad_next_char: None,
            value_parser_index: -1,
        });

        // Get a raw pointer to the heap-allocated builder
        let new_builder_ptr = Box::into_raw(new_builder);

        // Update the active pointer in the current builder
        self.active = Some(new_builder_ptr);
    }

    fn optional_end(&mut self) -> Result<(), String> {
        match self.get_active_mut().parent {
            Some(parent) => {
                if !self.get_active_mut().printer_parsers.is_empty() {
                    let cpp = CompositePrinterParser::new(
                        mem::take(&mut self.get_active_mut().printer_parsers),
                        self.get_active_mut().optional,
                    );
                    self.append_internal(Box::new(cpp));
                }
                self.active = Some(parent);
            }
            None => {
                return Err(
                    "Cannot call optionalEnd() as there was no previous call to optionalStart()"
                        .to_owned(),
                )
            }
        }
        Ok(())
    }

    fn append_text_with_field_and_text_style(
        &mut self,
        field: TemporalField,
        text_style: TextStyle,
    ) {
        self.append_internal(Box::new(TextPrinterParser::new(field, text_style)));
    }

    fn append_internal(&mut self, pp: Box<dyn DateTimePrinterParser>) -> isize {
        if self.get_active_mut().pad_next_width > 0 {
            let pp = PadPrinterParserDecorator::new(
                pp,
                self.get_active_mut().pad_next_width,
                self.get_active_mut().pad_next_char.unwrap(), // At this point we know that pad_next_char is Some, see pad_next
            );
            self.get_active_mut().pad_next_width = 0;
            self.get_active_mut().pad_next_char = Some('\0');
            self.get_active_mut().printer_parsers.push(Box::new(pp));
            self.get_active_mut().value_parser_index = -1;
            return self.get_active_mut().printer_parsers.len() as isize - 1;
        }
        self.get_active_mut().printer_parsers.push(pp);
        self.get_active_mut().value_parser_index = -1;
        self.get_active_mut().printer_parsers.len() as isize - 1
    }
}
