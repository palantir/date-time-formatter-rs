use crate::{
    date_time_parse_context::DateTimeParseContext, date_time_print_context::DateTimePrintContext,
    date_time_printer_parser::DateTimePrinterParser,
};

#[derive(Debug)]
pub struct StringLiteralPrinterParser {
    literal: String,
}
impl StringLiteralPrinterParser {
    pub fn new(literal: String) -> Self {
        StringLiteralPrinterParser { literal }
    }
}
impl DateTimePrinterParser for StringLiteralPrinterParser {
    fn format(
        &self,
        _context: &mut DateTimePrintContext,
        buf: &mut String,
    ) -> Result<bool, String> {
        buf.push_str(&self.literal);
        Ok(true)
    }

    fn parse(
        &self,
        context: &mut DateTimeParseContext,
        text: &str,
        position: isize,
    ) -> Result<isize, String> {
        let length = text.len();
        if position > length as isize || position < 0 {
            return Err("Index out of bounds".to_owned());
        }
        if !context.sub_sequence_equals(
            text,
            position as usize,
            &self.literal,
            0,
            self.literal.len(),
        ) {
            return Ok(!position);
        }
        Ok(position + self.literal.len() as isize)
    }
}
