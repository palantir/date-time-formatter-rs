use crate::{
    date_time_formatter::DateTimeFormatter, parsed::Parsed, temporal_field::TemporalField,
};

#[derive(Debug)]
pub struct DateTimePrintContext<'a, 'b> {
    parsed: &'a Parsed,
    formatter: &'b DateTimeFormatter,
    optional: usize,
}

impl<'a, 'b> DateTimePrintContext<'a, 'b> {
    pub fn new(parsed: &'a Parsed, formatter: &'b DateTimeFormatter) -> Self {
        DateTimePrintContext {
            parsed,
            formatter,
            optional: 0,
        }
    }

    pub fn start_optional(&mut self) {
        self.optional += 1;
    }

    pub fn end_optional(&mut self) {
        self.optional -= 1;
    }

    pub fn get_value(&self, field: TemporalField) -> Option<i64> {
        if self.optional > 0 && !self.parsed.is_supported(&field) {
            return None;
        }
        self.parsed.get_long(field)
    }

    pub fn get_formatter(&self) -> &DateTimeFormatter {
        self.formatter
    }
}
