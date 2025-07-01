use crate::{
    date_time_parse_context::DateTimeParseContext, date_time_print_context::DateTimePrintContext,
    number_printer_parser, number_printer_parser::DefaultNumberPrinterParser,
    number_printer_parser::NumberPrinterParser, sign_style::SignStyle,
    temporal_field::TemporalField,
};

#[derive(Debug)]
pub struct ReducedPrinterParser {
    field: TemporalField,
    min_width: isize,
    max_width: isize,
    sign_style: SignStyle,
    subsequent_width: isize,

    // Not from NumberPrinterParser
    base_value: i64,
}
impl ReducedPrinterParser {
    pub const BASE_VALUE: i64 = 2000;

    pub fn new(field: TemporalField, min_width: isize, max_width: isize, base_value: i64) -> Self {
        ReducedPrinterParser {
            field,
            min_width,
            max_width,
            sign_style: SignStyle::NotNegative,
            subsequent_width: 0,
            base_value,
        }
    }
}

impl NumberPrinterParser for ReducedPrinterParser {
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

impl DefaultNumberPrinterParser for ReducedPrinterParser {
    fn get_value(&self, _context: &DateTimePrintContext, value: i64) -> i64 {
        let abs_value = value.abs();
        // NOTE: We are ignoring the baseDate branch from the Java code
        // because we do not handle different Chronologies
        if value >= self.base_value
            && value
                < self.base_value + number_printer_parser::EXCEED_POINTS[self.min_width as usize]
        {
            return abs_value % number_printer_parser::EXCEED_POINTS[self.min_width as usize];
        }
        abs_value % number_printer_parser::EXCEED_POINTS[self.max_width as usize]
    }

    fn set_value(
        &self,
        context: &mut DateTimeParseContext,
        value: i64,
        error_pos: isize,
        success_pos: isize,
    ) -> Result<isize, String> {
        let mut value = value;
        let parse_len = success_pos - error_pos;
        if parse_len == self.min_width && value >= 0 {
            let range = number_printer_parser::EXCEED_POINTS[self.min_width as usize];
            let last_part = self.base_value % range;
            let base_part = self.base_value - last_part;
            if self.base_value > 0 {
                value += base_part;
            } else {
                value = base_part - value;
            }
            if value < self.base_value {
                value += range
            }
        }
        context.set_parsed_field(self.field, value, error_pos, success_pos)
    }
}
