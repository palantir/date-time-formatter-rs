use crate::{
    date_time_parse_context::DateTimeParseContext, date_time_print_context::DateTimePrintContext,
    date_time_printer_parser::DateTimePrinterParser,
};

#[derive(Debug)]
pub struct CharLiteralPrinterParser {
    literal: char,
    is_space_separator: bool,
}
impl CharLiteralPrinterParser {
    pub fn new(literal: char) -> Self {
        CharLiteralPrinterParser {
            literal,
            is_space_separator: literal.is_whitespace(),
        }
    }

    fn space_equals(&self, context: &DateTimeParseContext, ch: char) -> bool {
        !context.is_strict() && self.is_space_separator && ch.is_whitespace()
    }
}
impl DateTimePrinterParser for CharLiteralPrinterParser {
    fn format(
        &self,
        _context: &mut DateTimePrintContext,
        buf: &mut String,
    ) -> Result<bool, String> {
        buf.push(self.literal);
        Ok(true)
    }

    fn parse(
        &self,
        context: &mut DateTimeParseContext,
        text: &str,
        position: isize,
    ) -> Result<isize, String> {
        let length = text.len();
        if position == length as isize {
            return Ok(!position);
        }
        let ch = text.chars().nth(position as usize).unwrap();
        if ch != self.literal && context.is_case_sensitive()
            || !ch.eq_ignore_ascii_case(&self.literal) && !self.space_equals(context, ch)
        {
            return Ok(!position);
        }
        Ok(position + 1)
    }
}
