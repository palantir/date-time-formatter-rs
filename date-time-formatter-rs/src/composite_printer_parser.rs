use crate::{
    date_time_parse_context::DateTimeParseContext, date_time_print_context::DateTimePrintContext,
    date_time_printer_parser::DateTimePrinterParser,
};

#[derive(Debug)]
pub struct CompositePrinterParser {
    printer_parsers: Vec<Box<dyn DateTimePrinterParser>>,
    optional: bool,
}
impl CompositePrinterParser {
    pub fn new(printer_parsers: Vec<Box<dyn DateTimePrinterParser>>, optional: bool) -> Self {
        CompositePrinterParser {
            printer_parsers,
            optional,
        }
    }
}
impl DateTimePrinterParser for CompositePrinterParser {
    fn format(&self, context: &mut DateTimePrintContext, buf: &mut String) -> Result<bool, String> {
        let length = buf.len();
        if self.optional {
            context.start_optional();
        }
        for pp in self.printer_parsers.iter() {
            if !pp.format(context, buf)? {
                buf.truncate(length);
                return Ok(true);
            }
        }
        if self.optional {
            context.end_optional();
        }
        Ok(true)
    }

    fn parse(
        &self,
        context: &mut DateTimeParseContext,
        text: &str,
        position: isize,
    ) -> Result<isize, String> {
        let mut position = position;
        if self.optional {
            context.start_optional()?;
            let mut pos = position;
            for pp in self.printer_parsers.iter() {
                pos = pp.parse(context, text, pos)?;
                if pos < 0 {
                    context.end_optional(false);
                    return Ok(position);
                }
            }
            context.end_optional(true);
            Ok(pos)
        } else {
            for pp in self.printer_parsers.iter() {
                position = pp.parse(context, text, position)?;
                if position < 0 {
                    break;
                }
            }
            Ok(position)
        }
    }
}
