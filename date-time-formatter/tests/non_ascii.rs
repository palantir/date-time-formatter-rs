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

use std::panic::{catch_unwind, AssertUnwindSafe};

use date_time_formatter::date_time_formatter::DateTimeFormatter;

#[test]
fn test_non_ascii_literals() -> Result<(), String> {
    // Expected timestamps created with Java
    let cases = [
        ("'é' yyyy-MM-dd", "é 2023-05-15", 1684108800000000),
        (
            "yyyy-MM-dd 'à' HH:mm",
            "2023-05-15 à 10:30",
            1684146600000000,
        ),
        ("yyyy·MM·dd", "2023·05·15", 1684108800000000),
        (
            "'日付' yyyy-MM-dd HH:mm XXX",
            "日付 2023-05-15 10:30 +05:30",
            1684126800000000,
        ),
    ];
    for (pattern, text, micros) in cases {
        let formatter = DateTimeFormatter::of_pattern(pattern)?;
        let parsed = formatter.parse(text)?;
        assert_eq!(parsed.to_epoch_microseconds()?, micros, "{pattern:?}");
        assert_eq!(formatter.format(&parsed)?, text);
    }
    Ok(())
}

#[test]
fn test_malformed_input_is_rejected() {
    // Java rejects every combination of these
    let patterns = [
        "yyyy-MM-dd",
        "ppd/MM/yyyy",
        "yyyy-MM-dd'T'HH:mm:ssXXX",
        "MMMM d, yyyy",
        "EEEE, MMMM d, yyyy",
        "yyyy-MM-dd'T'HH:mm:ss.SSSSSS",
        "yyyy-MM-dd HH:mm:ss.n",
        "yyyy-MM-dd 'é' HH",
        "yyyy-MM-dd h:mm a",
        "yyyy-MM-dd'T'HH:mm:ssO",
        "yyyy-MM-dd'T'HH:mm:ssOOOO",
        "dd MMM yy",
        "pppMMM",
        "yyyy-MM-dd'T'HH:mm:ssxxxxx",
    ];
    let inputs = [
        "",
        "2023",
        "2023-05-1",
        "2023-05-15x",
        "é",
        "2023-05-é5",
        "ñ2023-05-15",
        "2023-05-15T10:00:00+05:3é",
        "Ma 5, 2023",
        "€€€€€€€€€€€€€€€",
        "1/05/2023",
        "5/05/2023",
        "1é/05/2023",
        "-2023-05-15",
        "2023-05-15T10:00:00.123",
        "2023-05-15T10:00:00GMT+5:3é",
        "2023-05-15T10:00:00GMT+é",
        "2023-05-15T10:00:00GMT+05:é",
        "01 Maé 23",
        "12:00 é",
        " é€",
        "2023-05-15 é 1é",
        "2023-05-15T10:00:00+05:é0",
        "2023-05-15 10:00:00.é",
    ];
    let mut failures = vec![];
    for pattern in patterns {
        let formatter = DateTimeFormatter::of_pattern(pattern).unwrap();
        for input in inputs {
            match catch_unwind(AssertUnwindSafe(|| formatter.parse(input))) {
                Ok(Err(_)) => {}
                Ok(Ok(parsed)) => {
                    failures.push(format!("{pattern:?} accepted {input:?}: {parsed:?}"))
                }
                Err(_) => failures.push(format!("{pattern:?} panicked on {input:?}")),
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
