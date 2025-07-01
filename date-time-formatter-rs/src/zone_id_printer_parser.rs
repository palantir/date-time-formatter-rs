use crate::{
    date_time_parse_context::DateTimeParseContext, date_time_print_context::DateTimePrintContext,
    date_time_printer_parser::DateTimePrinterParser,
};

#[derive(Debug)]
pub struct ZoneIdPrinterParser {}
impl ZoneIdPrinterParser {
    pub fn new() -> Self {
        ZoneIdPrinterParser {}
    }
}
impl DateTimePrinterParser for ZoneIdPrinterParser {
    fn format(
        &self,
        _context: &mut DateTimePrintContext,
        _buf: &mut String,
    ) -> Result<bool, String> {
        Err("ZoneIdPrinterParser format not implemented yet".to_owned())
    }

    fn parse(
        &self,
        _context: &mut DateTimeParseContext,
        _text: &str,
        _position: isize,
    ) -> Result<isize, String> {
        Err("ZoneIdPrinterParser parse not implemented yet".to_owned())
    }
}
