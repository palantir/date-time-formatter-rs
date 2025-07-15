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

use std::collections::HashSet;

use crate::{
    date_time_formatter::DateTimeFormatter, decimal_style::DecimalStyle, parsed::Parsed,
    resolver_style::ResolverStyle, temporal_field::TemporalField,
};

pub struct DateTimeParseContext<'a> {
    formatter: &'a DateTimeFormatter,
    case_sensitive: bool,
    strict: bool,
    parsed: Vec<Parsed>,
}

impl<'a> DateTimeParseContext<'a> {
    pub fn new(formatter: &'a DateTimeFormatter) -> Self {
        DateTimeParseContext {
            formatter,
            // We set case sensitive to false by default
            case_sensitive: false,
            strict: true,
            parsed: vec![Parsed::new()],
        }
    }

    pub fn get_decimal_style(&self) -> &DecimalStyle {
        self.formatter.get_decimal_style()
    }

    pub fn sub_sequence_equals(
        &self,
        cs1: &str,
        offset1: usize,
        cs2: &str,
        offset2: usize,
        length: usize,
    ) -> bool {
        if offset1 + length > cs1.len() || offset2 + length > cs2.len() {
            return false;
        }

        let slice1 = &cs1[offset1..offset1 + length];
        let slice2 = &cs2[offset2..offset2 + length];

        if self.case_sensitive {
            slice1 == slice2
        } else {
            slice1.eq_ignore_ascii_case(slice2)
        }
    }

    pub fn char_equals(&self, ch1: char, ch2: char) -> bool {
        if self.case_sensitive {
            return ch1 == ch2;
        }
        DateTimeParseContext::char_equals_ignore_case(ch1, ch2)
    }

    fn char_equals_ignore_case(c1: char, c2: char) -> bool {
        c1 == c2 || c1.eq_ignore_ascii_case(&c2)
    }

    pub fn start_optional(&mut self) -> Result<(), String> {
        let cloned_current_parsed = self.current_parsed()?.clone();
        self.parsed.push(cloned_current_parsed);
        Ok(())
    }

    pub fn end_optional(&mut self, successful: bool) {
        if successful {
            self.parsed.remove(self.parsed.len() - 2);
        } else {
            self.parsed.remove(self.parsed.len() - 1);
        }
    }

    pub fn current_parsed(&mut self) -> Result<&mut Parsed, String> {
        if let Some(parsed) = self.parsed.last_mut() {
            return Ok(parsed);
        }
        Err("Parsed should always have at least one element".to_owned())
    }

    pub fn resolve_to_parsed(
        &mut self,
        resolver_style: &ResolverStyle,
        resolver_fields: &HashSet<TemporalField>,
    ) -> Result<Parsed, String> {
        self.current_parsed()?
            .resolve(resolver_style, resolver_fields)
    }

    pub fn set_parsed_field(
        &mut self,
        field: TemporalField,
        value: i64,
        error_pos: isize,
        success_pos: isize,
    ) -> Result<isize, String> {
        match self.current_parsed()?.field_values.insert(field, value) {
            Some(old) => {
                if old != value {
                    return Ok(!error_pos);
                }
                Ok(success_pos)
            }
            None => Ok(success_pos),
        }
    }

    pub fn is_strict(&self) -> bool {
        self.strict
    }

    pub fn is_case_sensitive(&self) -> bool {
        self.case_sensitive
    }
}
