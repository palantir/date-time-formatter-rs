use crate::{date_time_printer_parser::DateTimePrinterParser, text_style::TextStyle};

#[derive(Debug)]
pub struct DayPeriodPrinterParser {
    _style: TextStyle,
}
impl DayPeriodPrinterParser {
    pub fn new(style: TextStyle) -> Self {
        DayPeriodPrinterParser { _style: style }
    }
}
impl DateTimePrinterParser for DayPeriodPrinterParser {
    fn format(
        &self,
        _context: &mut crate::date_time_print_context::DateTimePrintContext,
        _buf: &mut String,
    ) -> Result<bool, String> {
        Err("DayPeriodPrinterParser not implemented yet".to_owned())
    }

    fn parse(
        &self,
        _context: &mut crate::date_time_parse_context::DateTimeParseContext,
        _text: &str,
        _position: isize,
    ) -> Result<isize, String> {
        Err("DayPeriodPrinterParser not implemented yet".to_owned())
    }
}
