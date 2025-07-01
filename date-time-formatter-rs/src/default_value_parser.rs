use crate::{
    date_time_parse_context::DateTimeParseContext, date_time_print_context::DateTimePrintContext,
    date_time_printer_parser::DateTimePrinterParser, temporal_field::TemporalField,
};

// This is used to set a default value for a field if it is not already set in the parsed context.
// It should not have any effect on the formatting process.
#[derive(Debug)]
pub struct DefaultValueParser {
    field: TemporalField,
    value: i64,
}

impl DefaultValueParser {
    pub fn new(field: TemporalField, value: i64) -> Self {
        DefaultValueParser { field, value }
    }
}

impl DateTimePrinterParser for DefaultValueParser {
    fn format(
        &self,
        _context: &mut DateTimePrintContext,
        _buf: &mut String,
    ) -> Result<bool, String> {
        Ok(true)
    }

    fn parse(
        &self,
        context: &mut DateTimeParseContext,
        _text: &str,
        position: isize,
    ) -> Result<isize, String> {
        if !context
            .current_parsed()?
            .field_values
            .contains_key(&self.field)
        {
            context.set_parsed_field(self.field, self.value, position, position)?;
        }
        Ok(position)
    }
}
