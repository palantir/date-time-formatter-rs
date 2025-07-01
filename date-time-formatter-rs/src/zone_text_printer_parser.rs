use crate::{date_time_printer_parser::DateTimePrinterParser, text_style::TextStyle};

#[derive(Debug)]
pub struct ZoneTextPrinterParser {
    _text_style: TextStyle,
    _is_generic: bool,
}

impl ZoneTextPrinterParser {
    pub fn new(text_style: TextStyle, is_generic: bool) -> Self {
        ZoneTextPrinterParser {
            _text_style: text_style,
            _is_generic: is_generic,
        }
    }
}

impl DateTimePrinterParser for ZoneTextPrinterParser {
    fn format(
        &self,
        _context: &mut crate::date_time_print_context::DateTimePrintContext,
        _buf: &mut String,
    ) -> Result<bool, String> {
        Err("ZoneTextPrinterParser format not implemented yet".to_owned())
    }

    fn parse(
        &self,
        _context: &mut crate::date_time_parse_context::DateTimeParseContext,
        _text: &str,
        _position: isize,
    ) -> Result<isize, String> {
        Err("ZoneTextPrinterParser parse not implemented yet".to_owned())
    }
}
