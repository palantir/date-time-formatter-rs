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
    date_time_formatter::DateTimeFormatter, parsed::Parsed, temporal_field::TemporalField,
};

#[derive(Debug)]
pub struct DateTimePrintContext<'a, 'b> {
    parsed: &'a Parsed,
    formatter: &'b DateTimeFormatter,
    optional: usize,
}

impl<'a, 'b> DateTimePrintContext<'a, 'b> {
    pub fn new(parsed: &'a Parsed, formatter: &'b DateTimeFormatter) -> Self {
        DateTimePrintContext {
            parsed,
            formatter,
            optional: 0,
        }
    }

    pub fn start_optional(&mut self) {
        self.optional += 1;
    }

    pub fn end_optional(&mut self) {
        self.optional -= 1;
    }

    pub fn get_value(&self, field: TemporalField) -> Option<i64> {
        if self.optional > 0 && !self.parsed.is_supported(&field) {
            return None;
        }
        self.parsed.get_long(field)
    }

    pub fn get_formatter(&self) -> &DateTimeFormatter {
        self.formatter
    }
}
