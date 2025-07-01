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
