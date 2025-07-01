use crate::{
    date_time_parse_context::DateTimeParseContext, date_time_print_context::DateTimePrintContext,
    number_printer_parser::NumberPrinterParser,
};
use std::fmt::Debug;

pub trait AsAny {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl<T: 'static> AsAny for T {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub trait DateTimePrinterParser: AsAny + Debug {
    fn format(&self, context: &mut DateTimePrintContext, buf: &mut String) -> Result<bool, String>;
    fn parse(
        &self,
        context: &mut DateTimeParseContext,
        text: &str,
        position: isize,
    ) -> Result<isize, String>;

    fn as_number_printer_parser(&mut self) -> Option<&mut dyn NumberPrinterParser> {
        None
    }
}
