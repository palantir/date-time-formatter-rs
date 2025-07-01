use crate::{
    date_time_parse_context::DateTimeParseContext, date_time_print_context::DateTimePrintContext,
    date_time_printer_parser::DateTimePrinterParser,
};

#[derive(Debug)]
pub struct PadPrinterParserDecorator {
    printer_parser: Box<dyn DateTimePrinterParser>,
    pad_width: isize,
    pad_char: char,
}
impl PadPrinterParserDecorator {
    pub fn new(
        printer_parser: Box<dyn DateTimePrinterParser>,
        pad_width: isize,
        pad_char: char,
    ) -> Self {
        PadPrinterParserDecorator {
            printer_parser,
            pad_width,
            pad_char,
        }
    }
}
impl DateTimePrinterParser for PadPrinterParserDecorator {
    fn format(&self, context: &mut DateTimePrintContext, buf: &mut String) -> Result<bool, String> {
        let pre_len = buf.len();
        if !self.printer_parser.format(context, buf)? {
            return Ok(false);
        }
        let len = (buf.len() - pre_len) as isize;
        if len > self.pad_width {
            return Err(format!(
                "Cannot print as output of {len} characters exceeds pad with of current pad_width"
            ));
        }
        let count = self.pad_width - len;
        if count == 0 {
            return Ok(false);
        }
        if count == 1 {
            buf.insert(pre_len, self.pad_char);
            return Ok(true);
        }
        buf.insert_str(pre_len, &self.pad_char.to_string().repeat(count as usize));
        Ok(true)
    }

    fn parse(
        &self,
        context: &mut DateTimeParseContext,
        text: &str,
        position: isize,
    ) -> Result<isize, String> {
        let strict = context.is_strict();
        if position > text.len() as isize {
            return Err("Index out of bounds".to_owned());
        }
        if position == text.len() as isize {
            return Ok(!position);
        }
        let mut end_pos = position + self.pad_width;
        if end_pos > text.len() as isize {
            if strict {
                return Ok(!position);
            }
            end_pos = text.len() as isize;
        }
        let mut pos = position;
        while pos < end_pos
            && context.char_equals(text.chars().nth(pos as usize).unwrap(), self.pad_char)
        {
            pos += 1;
        }
        let text = &text[0..end_pos as usize];
        let result_pos = self.printer_parser.parse(context, text, pos)?;
        if result_pos != end_pos && strict {
            return Ok(!(position + pos));
        }
        Ok(result_pos)
    }
}
