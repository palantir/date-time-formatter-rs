use crate::number_printer_parser::BaseNumberPrinterParser;
use crate::{
    date_time_parse_context::DateTimeParseContext, date_time_print_context::DateTimePrintContext,
    date_time_printer_parser::DateTimePrinterParser, number_printer_parser::NumberPrinterParser,
    reduced_printer_parser::ReducedPrinterParser, sign_style::SignStyle,
    temporal_field::TemporalField,
};

#[derive(Debug)]
pub struct WeekBasedFieldPrinterParser {
    field: TemporalField,
    min_width: isize,
    max_width: isize,
    sign_style: SignStyle,
    subsequent_width: isize,

    // Fields not from NumberPrinterParser
    chr: char,
    count: isize,
}
impl WeekBasedFieldPrinterParser {
    pub fn new(chr: char, count: isize, min_width: isize, max_width: isize) -> Self {
        WeekBasedFieldPrinterParser {
            field: TemporalField::DayOfWeek,
            min_width,
            max_width,
            sign_style: SignStyle::NotNegative,
            subsequent_width: 0,
            chr,
            count,
        }
    }

    fn printer_parser(&self) -> Result<Box<dyn DateTimePrinterParser>, String> {
        match self.chr {
            'Y' => {
                let field = TemporalField::WeekBasedYear;
                if self.count == 2 {
                    let mut pp =
                        ReducedPrinterParser::new(field, 2, 2, ReducedPrinterParser::BASE_VALUE);
                    pp.set_subsequent_width(self.subsequent_width);
                    Ok(Box::new(pp))
                } else {
                    let mut pp = BaseNumberPrinterParser::new(
                        field,
                        self.count,
                        19,
                        if self.count < 4 {
                            SignStyle::Normal
                        } else {
                            SignStyle::ExceedsPad
                        },
                    );
                    pp.set_subsequent_width(self.subsequent_width);
                    Ok(Box::new(pp))
                }
            }
            'e' | 'c' => {
                let mut pp = BaseNumberPrinterParser::new(
                    // NOTE: This will always intercept these patterns,
                    // so the map entry in `TemporalField` which points
                    // to DayOfWeek instead is somewhat misleading.
                    TemporalField::LocaleDayOfWeek,
                    self.min_width,
                    self.max_width,
                    SignStyle::NotNegative,
                );
                pp.set_subsequent_width(self.subsequent_width);
                Ok(Box::new(pp))
            }
            'w' => {
                let mut pp = BaseNumberPrinterParser::new(
                    TemporalField::WeekOfWeekBasedYear,
                    self.min_width,
                    self.max_width,
                    SignStyle::NotNegative,
                );
                pp.set_subsequent_width(self.subsequent_width);
                Ok(Box::new(pp))
            }
            'W' => {
                let mut pp = BaseNumberPrinterParser::new(
                    TemporalField::WeekOfMonth,
                    self.min_width,
                    self.max_width,
                    SignStyle::NotNegative,
                );
                pp.set_subsequent_width(self.subsequent_width);
                Ok(Box::new(pp))
            }
            _ => Err("unreachable".to_owned()),
        }
    }
}
impl DateTimePrinterParser for WeekBasedFieldPrinterParser {
    fn format(&self, context: &mut DateTimePrintContext, buf: &mut String) -> Result<bool, String> {
        self.printer_parser()?.format(context, buf)
    }
    fn parse(
        &self,
        context: &mut DateTimeParseContext,
        text: &str,
        position: isize,
    ) -> Result<isize, String> {
        self.printer_parser()?.parse(context, text, position)
    }
    fn as_number_printer_parser(&mut self) -> Option<&mut dyn NumberPrinterParser> {
        Some(self)
    }
}
impl NumberPrinterParser for WeekBasedFieldPrinterParser {
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
