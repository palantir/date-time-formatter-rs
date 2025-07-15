// Copyright (C) 2025 Palantir
// This program is free software; you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation; either version 2 of the License, or (at your option)
// any later version.
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for
// more details.
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc., 59
// Temple Place, Suite 330, Boston, MA 02111-1307 USA
// sbhatia@palantir.com
// "CLASSPATH" EXCEPTION TO THE GPL
// Certain source files distributed by Oracle America and/or its affiliates are
// subject to the following clarification and special exception to the GPL, but
// only where Oracle has expressly included in the particular source file's header
// the words "Oracle designates this particular file as subject to the "Classpath"
// exception as provided by Oracle in the LICENSE file that accompanied this code."
// Linking this library statically or dynamically with other modules is making
// a combined work based on this library.  Thus, the terms and conditions of
// the GNU General Public License cover the whole combination.
// As a special exception, the copyright holders of this library give you
// permission to link this library with independent modules to produce an
// executable, regardless of the license terms of these independent modules,
// and to copy and distribute the resulting executable under terms of your
// choice, provided that you also meet, for each linked independent module,
// the terms and conditions of the license of that module.  An independent
// module is a module which is not derived from or based on this library.  If
// you modify this library, you may extend this exception to your version of
// the library, but you are not obligated to do so.  If you do not wish to do
// so, delete this exception statement from your version. "

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
