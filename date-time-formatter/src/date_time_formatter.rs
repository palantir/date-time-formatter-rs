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
    composite_printer_parser::CompositePrinterParser,
    date_time_formatter_builder::DateTimeFormatterBuilder,
    date_time_parse_context::DateTimeParseContext, date_time_print_context::DateTimePrintContext,
    date_time_printer_parser::DateTimePrinterParser, decimal_style::DecimalStyle,
    parse_position::ParsePosition, parsed::Parsed, resolver_style::ResolverStyle,
    temporal_field::TemporalField,
};

#[derive(Debug)]
pub struct DateTimeFormatter {
    printer_parser: CompositePrinterParser,
    decimal_style: DecimalStyle,
    resolver_style: ResolverStyle,
    resolver_fields: HashSet<TemporalField>,
}

impl DateTimeFormatter {
    pub fn new(
        printer_parser: CompositePrinterParser,
        decimal_style: DecimalStyle,
        resolver_style: ResolverStyle,
        resolver_fields: HashSet<TemporalField>,
    ) -> Self {
        DateTimeFormatter {
            printer_parser,
            decimal_style,
            resolver_style,
            resolver_fields,
        }
    }

    pub fn of_pattern(pattern: &str) -> Result<Self, String> {
        let mut builder = DateTimeFormatterBuilder::new();
        builder.parse_pattern(pattern)?;
        Ok(builder.create_formatter())
    }

    pub fn of_pattern_with_defaults(
        pattern: &str,
        defaults: Vec<(TemporalField, i64)>,
    ) -> Result<Self, String> {
        let mut builder = DateTimeFormatterBuilder::new();
        builder.parse_pattern(pattern)?;
        for (field, value) in defaults {
            builder.parse_defaulting(field, value)?;
        }
        Ok(builder.create_formatter())
    }

    pub fn get_decimal_style(&self) -> &DecimalStyle {
        &self.decimal_style
    }

    pub fn parse(&self, text: &str) -> Result<Parsed, String> {
        let mut pos = ParsePosition::new(0);
        let context = self.parse_unresolved(text, &mut pos);
        if context.is_err() || pos.get_error_index() >= 0 || pos.get_index() < text.len() as isize {
            return Err(format!(
                "Failed to parse text: '{}', error index: {}, position: {}",
                text,
                pos.get_error_index(),
                pos.get_index()
            ));
        }
        context?.resolve_to_parsed(&self.resolver_style, &self.resolver_fields)
    }

    fn parse_unresolved(
        &self,
        text: &str,
        position: &mut ParsePosition,
    ) -> Result<DateTimeParseContext, String> {
        let mut context = DateTimeParseContext::new(self);
        let pos = self
            .printer_parser
            .parse(&mut context, text, position.get_index())?;
        if pos < 0 {
            position.set_error_index(!pos);
            return Err("Failed to parse".to_owned());
        }
        position.set_index(pos);
        Ok(context)
    }

    pub fn format(&self, input: &Parsed) -> Result<String, String> {
        let mut buf = String::new();
        let mut context = DateTimePrintContext::new(input, self);
        self.printer_parser.format(&mut context, &mut buf)?;
        Ok(buf)
    }
}
