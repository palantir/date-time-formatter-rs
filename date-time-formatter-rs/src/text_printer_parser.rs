use crate::{
    date_time_parse_context::DateTimeParseContext, date_time_print_context::DateTimePrintContext,
    date_time_printer_parser::DateTimePrinterParser, date_time_text_provider::DateTimeTextProvider,
    number_printer_parser::BaseNumberPrinterParser, sign_style::SignStyle,
    temporal_field::TemporalField, text_style::TextStyle,
};

#[derive(Debug)]
pub struct TextPrinterParser {
    field: TemporalField,
    text_style: TextStyle,
    provider: DateTimeTextProvider,
    number_printer_parser: BaseNumberPrinterParser,
}
impl TextPrinterParser {
    pub fn new(field: TemporalField, text_style: TextStyle) -> Self {
        TextPrinterParser {
            field,
            text_style,
            provider: DateTimeTextProvider::new(),
            number_printer_parser: BaseNumberPrinterParser::new(field, 1, 19, SignStyle::Normal),
        }
    }
}
impl DateTimePrinterParser for TextPrinterParser {
    fn format(&self, context: &mut DateTimePrintContext, buf: &mut String) -> Result<bool, String> {
        let value = match context.get_value(self.field) {
            Some(value) => value,
            None => return Ok(false),
        };
        match self.provider.get_text(self.field, value, self.text_style) {
            Some(text) => {
                buf.push_str(text);
                Ok(true)
            }
            None => self.number_printer_parser.format(context, buf),
        }
    }

    fn parse(
        &self,
        context: &mut DateTimeParseContext,
        text: &str,
        position: isize,
    ) -> Result<isize, String> {
        let length = text.len();
        if position < 0 || position > length as isize {
            return Err("Index out of bounds".to_owned());
        }
        let style = if context.is_strict() {
            Some(self.text_style)
        } else {
            None
        };
        if let Some(it) = self.provider.get_text_iterator(self.field, style) {
            for (it_text, it_value) in it {
                if context.sub_sequence_equals(it_text, 0, text, position as usize, it_text.len()) {
                    return context.set_parsed_field(
                        self.field,
                        it_value,
                        position,
                        position + it_text.len() as isize,
                    );
                }
            }
            if self.field == TemporalField::Era && !context.is_strict() {
                // Taken from IsoEras, can probably add more here without causing too many problems
                let eras = ["BCE", "CE"];
                for (era_value, era) in eras.iter().enumerate() {
                    if context.sub_sequence_equals(era, 0, text, position as usize, era.len()) {
                        return context.set_parsed_field(
                            self.field,
                            era_value as i64,
                            position,
                            position + era.len() as isize,
                        );
                    }
                }
            }
            if context.is_strict() {
                return Ok(!position);
            }
        }
        self.number_printer_parser.parse(context, text, position)
    }
}
