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

mod char_literal_printer_parser;
mod chronology;
mod composite_printer_parser;
pub mod date_time_formatter;
mod date_time_formatter_builder;
mod date_time_parse_context;
mod date_time_print_context;
mod date_time_printer_parser;
mod date_time_text_provider;
mod day_period_printer_parser;
mod decimal_style;
mod default_value_parser;
mod localized_offset_id_printer_parser;
mod nanos_printer_parser;
mod number_printer_parser;
mod offset_id_printer_parser;
mod pad_printer_parser_decorator;
mod parse_position;
pub mod parsed;
mod period;
mod reduced_printer_parser;
mod resolver_style;
mod sign_style;
mod string_literal_printer_parser;
pub mod temporal_field;
mod temporal_unit;
mod text_printer_parser;
mod text_style;
mod value_range;
mod week_based_field_printer_parser;
mod week_fields_utils;
mod zone_id_printer_parser;
mod zone_text_printer_parser;

#[cfg(test)]
mod tests {
    use std::fs::File;

    use chrono::NaiveDate;

    use crate::{
        date_time_formatter::DateTimeFormatter, parsed::Parsed, temporal_field::TemporalField,
    };

    #[test]
    fn basic_parsing_and_printing() -> Result<(), String> {
        let parsing_formatter = DateTimeFormatter::of_pattern("yyyy-MM-dd'T'HH:mm:ss")?;
        let printing_formatter = DateTimeFormatter::of_pattern(
            "'Year:' yyyy, 'Month:' MM, 'Date:' dd, 'Hour:' HH, 'Minute:' mm, 'Second:' ss",
        )?;
        let original_string = "2025-05-13T14:30:00";
        let parsed: Parsed = parsing_formatter.parse(original_string)?;
        println!("{parsed:?}");
        let recovered_string = printing_formatter.format(&parsed)?;
        assert_eq!(
            "Year: 2025, Month: 05, Date: 13, Hour: 14, Minute: 30, Second: 00",
            recovered_string
        );
        Ok(())
    }

    #[test]
    fn test_default_behaviour() -> Result<(), String> {
        let formatter = DateTimeFormatter::of_pattern_with_defaults(
            "yyyy-MM",
            vec![(TemporalField::DayOfMonth, 1)],
        )?;
        let parsed = formatter.parse("2020-01")?;
        println!("{parsed:?}");
        let date = parsed.to_chrono_naive_date()?;
        assert_eq!(date, NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        Ok(())
    }

    #[test]
    fn test_format_cases() -> Result<(), String> {
        let cases = get_format_characteristics_cases();
        for (pattern, examples) in cases.iter() {
            println!("====");
            println!("Pattern: {pattern}");
            let formatter = DateTimeFormatter::of_pattern(pattern)?;
            println!("{formatter:?}");
            for example in examples.iter() {
                println!("Example: {example}");
                let parsed = formatter.parse(example)?;
                println!("{parsed:?}");
                let formatted = formatter.format(&parsed)?;
                println!("Formatted: {formatted}");
                assert_eq!(&formatted, example);
                let recovered_parsed = formatter.parse(&formatted).unwrap();
                println!("Recovered {recovered_parsed:?}");
                assert_eq!(recovered_parsed, parsed);
            }
            println!("====");
        }
        Ok(())
    }

    #[test]
    fn test_complex_cases() -> Result<(), String> {
        let cases = get_complex_pattern_cases();
        for (pattern, examples) in cases.iter() {
            println!("====");
            println!("Pattern: {pattern}");
            let formatter = DateTimeFormatter::of_pattern(pattern)?;
            for example in examples.iter() {
                println!("Example: {example}");
                let parsed = formatter.parse(example)?;
                println!("{parsed:?}");
                if let Ok(timestamp) = parsed.to_epoch_microseconds() {
                    println!("Epoch microseconds: {timestamp:?}");
                    let reconstructed = Parsed::of_epoch_microseconds(timestamp);
                    println!("Reconstructed Parsed: {reconstructed:?}");
                    let reconstructed_string = &formatter.format(&reconstructed)?;
                    println!("Reconstructed string: {reconstructed_string}");
                    assert_eq!(reconstructed_string, example);
                }
                assert_eq!(&formatter.format(&parsed)?, example);
            }
            println!("====");
        }
        Ok(())
    }

    #[test]
    fn test_week_based_field_printer_parser() -> Result<(), String> {
        let pattern = "YYYY-'W'ww-e G";
        println!("Testing pattern: {pattern}");
        let formatter = DateTimeFormatter::of_pattern(pattern)?;
        let parsed = Parsed::of_epoch_microseconds(49987414950352279); //
        println!("{parsed:?}");
        let formatted = formatter.format(&parsed)?;
        assert_eq!(formatted, "3554-W03-6 AD");
        Ok(())
    }

    #[test]
    fn test_datetime_formatter_with_csv_data() -> Result<(), String> {
        // Load the test data from CSV file
        let file = File::open("java_generated_test_cases/datetime_test_data.csv")
            .map_err(|e| format!("Failed to open test data file: {}", e))?;

        let mut reader = csv::Reader::from_reader(file);

        // Process each row independently
        for (row_idx, result) in reader.records().enumerate() {
            let record = result.map_err(|e| format!("Error reading CSV row {}: {}", row_idx, e))?;

            // Check if we have at least 4 columns
            if record.len() < 3 {
                return Err(format!("Row {} has fewer than 3 columns", row_idx));
            }

            println!("===== Row {} =====", row_idx);

            // Get the four columns from the row
            let original_timestamp_str = &record[0];
            let pattern = &record[1];
            let expected_formatted = &record[2];

            // Parse the original timestamp
            let original_timestamp = original_timestamp_str.parse::<i64>().map_err(|e| {
                format!(
                    "Failed to parse timestamp {}: {}",
                    original_timestamp_str, e
                )
            })?;

            println!("Original timestamp: {}", original_timestamp);
            println!("Pattern: '{}'", pattern);
            println!("Expected formatted: '{}'", expected_formatted);

            // Create a formatter for this pattern
            let formatter = DateTimeFormatter::of_pattern(pattern).map_err(|e| {
                format!(
                    "Failed to create formatter for pattern '{}': {}",
                    pattern, e
                )
            })?;

            // Create a Parsed object from the timestamp
            let parsed = Parsed::of_epoch_microseconds(original_timestamp);
            println!("Parsed (original timestamp): {:?}", parsed);

            // Format the timestamp and compare with expected
            let formatted = formatter.format(&parsed)?;
            println!("Actual formatted:   '{}'", formatted);
            assert_eq!(formatted, expected_formatted);

            // Parse back to timestamp if a recovered timestamp is expected
            if record.len() >= 4 && !record[3].is_empty() {
                let expected_recovered_timestamp = record[3].parse::<i64>().map_err(|e| {
                    format!(
                        "Failed to parse recovered timestamp {}: {}",
                        record[3].to_owned(),
                        e
                    )
                })?;

                let recovered_parsed = formatter.parse(expected_formatted)?;
                println!("Recovered Parsed: {:?}", recovered_parsed);

                let recovered_timestamp = recovered_parsed.to_epoch_microseconds();

                println!(
                    "Expected recovered timestamp: {}",
                    expected_recovered_timestamp
                );
                println!("Actual recovered timestamp:   {:?}", recovered_timestamp);

                assert_eq!(recovered_timestamp, Ok(expected_recovered_timestamp));
            } else {
                // If the recovered timestamp field is empty or doesn't exist, verify that either:
                // 1. Parsing fails entirely (formatter.parse returns None), or
                // 2. Parsing succeeds but to_epoch_microseconds returns None

                let parsed_result = formatter.parse(expected_formatted);

                if let Ok(parsed_result) = parsed_result {
                    let recovered_timestamp = parsed_result.to_epoch_microseconds();
                    println!("Expected recovered timestamp: None (either parse failure or invalid date/time)");
                    println!("Actual recovered timestamp:   {:?}", recovered_timestamp);

                    // If parsing succeeded but we expected None, then to_epoch_microseconds should return an error
                    assert!(
                        recovered_timestamp.is_err(),
                        "Expected recovered timestamp to be None, but got: {:?}",
                        recovered_timestamp
                    );
                } else {
                    // Parsing failed entirely, which is acceptable when we expect None
                    println!("Parsing failed as expected for invalid format");
                }
            }

            println!("=====");
        }

        println!("All tests passed successfully!");
        Ok(())
    }

    fn get_format_characteristics_cases() -> Vec<(&'static str, Vec<&'static str>)> {
        vec![
            ("a", vec!["AM", "AM", "AM"]),
            ("c", vec!["3", "5", "3"]),
            ("ccc", vec!["Tue", "Thu", "Tue"]),
            ("cccc", vec!["Tuesday", "Thursday", "Tuesday"]),
            ("ccccc", vec!["T", "T", "T"]),
            ("d", vec!["23", "1", "31"]),
            ("dd", vec!["23", "01", "31"]),
            ("e", vec!["3", "5", "3"]),
            ("ee", vec!["03", "05", "03"]),
            ("eee", vec!["Tue", "Thu", "Tue"]),
            ("eeee", vec!["Tuesday", "Thursday", "Tuesday"]),
            ("eeeee", vec!["T", "T", "T"]),
            ("g", vec!["60514", "40587", "60675"]),
            ("gg", vec!["60514", "40587", "60675"]),
            ("ggg", vec!["60514", "40587", "60675"]),
            ("gggg", vec!["60514", "40587", "60675"]),
            ("ggggg", vec!["60514", "40587", "60675"]),
            ("gggggg", vec!["060514", "040587", "060675"]),
            ("ggggggg", vec!["0060514", "0040587", "0060675"]),
            ("gggggggg", vec!["00060514", "00040587", "00060675"]),
            ("ggggggggg", vec!["000060514", "000040587", "000060675"]),
            ("k", vec!["10", "24", "1"]),
            ("kk", vec!["10", "24", "01"]),
            ("m", vec!["18", "0", "1"]),
            ("mm", vec!["18", "00", "01"]),
            ("n", vec!["584186000", "0", "0"]),
            ("nn", vec!["584186000", "00", "00"]),
            ("nnn", vec!["584186000", "000", "000"]),
            ("nnnn", vec!["584186000", "0000", "0000"]),
            ("nnnnn", vec!["584186000", "00000", "00000"]),
            ("nnnnnn", vec!["584186000", "000000", "000000"]),
            ("nnnnnnn", vec!["584186000", "0000000", "0000000"]),
            ("nnnnnnnn", vec!["584186000", "00000000", "00000000"]),
            ("nnnnnnnnn", vec!["584186000", "000000000", "000000000"]),
            ("q", vec!["3", "1", "4"]),
            ("qq", vec!["03", "01", "04"]),
            ("qqq", vec!["Q3", "Q1", "Q4"]),
            ("qqqq", vec!["3rd quarter", "1st quarter", "4th quarter"]),
            ("qqqqq", vec!["3", "1", "4"]),
            ("s", vec!["7", "0", "1"]),
            ("ss", vec!["07", "00", "01"]),
            ("u", vec!["2024", "1970", "2024"]),
            ("uu", vec!["24", "70", "24"]),
            ("uuu", vec!["2024", "1970", "2024"]),
            ("uuuu", vec!["2024", "1970", "2024"]),
            ("uuuuu", vec!["02024", "01970", "02024"]),
            ("uuuuuu", vec!["002024", "001970", "002024"]),
            ("uuuuuuu", vec!["0002024", "0001970", "0002024"]),
            ("uuuuuuuu", vec!["00002024", "00001970", "00002024"]),
            ("uuuuuuuuu", vec!["000002024", "000001970", "000002024"]),
            // ("v", vec!["Z", "Z", "Z"]), // named timezone not supported
            // ("vvvv", vec!["Z", "Z", "Z"]),
            ("w", vec!["30", "1", "1"]),
            ("ww", vec!["30", "01", "01"]),
            ("x", vec!["+00", "+00", "+00"]),
            ("xx", vec!["+0000", "+0000", "+0000"]),
            ("xxx", vec!["+00:00", "+00:00", "+00:00"]),
            ("xxxx", vec!["+0000", "+0000", "+0000"]),
            ("xxxxx", vec!["+00:00", "+00:00", "+00:00"]),
            // ("z", vec!["Z", "Z", "Z"]), // named timezone not supported
            // ("zz", vec!["Z", "Z", "Z"]),
            // ("zzz", vec!["Z", "Z", "Z"]),
            // ("zzzz", vec!["Z", "Z", "Z"]),
            ("A", vec!["37087584", "0", "3661000"]),
            ("AA", vec!["37087584", "00", "3661000"]),
            ("AAA", vec!["37087584", "000", "3661000"]),
            ("AAAA", vec!["37087584", "0000", "3661000"]),
            ("AAAAA", vec!["37087584", "00000", "3661000"]),
            ("AAAAAA", vec!["37087584", "000000", "3661000"]),
            ("AAAAAAA", vec!["37087584", "0000000", "3661000"]),
            ("AAAAAAAA", vec!["37087584", "00000000", "03661000"]),
            ("AAAAAAAAA", vec!["037087584", "000000000", "003661000"]),
            // ("B", vec!["in the morning", "midnight", "at night"]), // day period not supported
            // ("BBBB", vec!["in the morning", "midnight", "at night"]),
            // ("BBBBB", vec!["in the morning", "mi", "at night"]),
            ("D", vec!["205", "1", "366"]),
            ("DD", vec!["205", "01", "366"]),
            ("DDD", vec!["205", "001", "366"]),
            ("E", vec!["Tue", "Thu", "Tue"]),
            ("EE", vec!["Tue", "Thu", "Tue"]),
            ("EEE", vec!["Tue", "Thu", "Tue"]),
            ("EEEE", vec!["Tuesday", "Thursday", "Tuesday"]),
            ("EEEEE", vec!["T", "T", "T"]),
            ("F", vec!["4", "1", "5"]),
            ("G", vec!["AD", "AD", "AD"]),
            ("GG", vec!["AD", "AD", "AD"]),
            ("GGG", vec!["AD", "AD", "AD"]),
            ("GGGG", vec!["Anno Domini", "Anno Domini", "Anno Domini"]),
            ("GGGGG", vec!["A", "A", "A"]),
            ("H", vec!["10", "0", "1"]),
            ("HH", vec!["10", "00", "01"]),
            ("K", vec!["10", "0", "1"]),
            ("KK", vec!["10", "00", "01"]),
            ("L", vec!["7", "1", "12"]),
            ("LL", vec!["07", "01", "12"]),
            ("LLL", vec!["Jul", "Jan", "Dec"]),
            ("LLLL", vec!["July", "January", "December"]),
            ("LLLLL", vec!["J", "J", "D"]),
            ("M", vec!["7", "1", "12"]),
            ("MM", vec!["07", "01", "12"]),
            ("MMM", vec!["Jul", "Jan", "Dec"]),
            ("MMMM", vec!["July", "January", "December"]),
            ("MMMMM", vec!["J", "J", "D"]),
            ("N", vec!["37087584186000", "0", "3661000000000"]),
            ("NN", vec!["37087584186000", "00", "3661000000000"]),
            ("NNN", vec!["37087584186000", "000", "3661000000000"]),
            ("NNNN", vec!["37087584186000", "0000", "3661000000000"]),
            ("NNNNN", vec!["37087584186000", "00000", "3661000000000"]),
            ("NNNNNN", vec!["37087584186000", "000000", "3661000000000"]),
            (
                "NNNNNNN",
                vec!["37087584186000", "0000000", "3661000000000"],
            ),
            (
                "NNNNNNNN",
                vec!["37087584186000", "00000000", "3661000000000"],
            ),
            (
                "NNNNNNNNN",
                vec!["37087584186000", "000000000", "3661000000000"],
            ),
            ("O", vec!["GMT", "GMT", "GMT"]),
            ("OOOO", vec!["GMT", "GMT", "GMT"]),
            ("Q", vec!["3", "1", "4"]),
            ("QQ", vec!["03", "01", "04"]),
            ("QQQ", vec!["Q3", "Q1", "Q4"]),
            ("QQQQ", vec!["3rd quarter", "1st quarter", "4th quarter"]),
            ("QQQQQ", vec!["3", "1", "4"]),
            ("S", vec!["5", "0", "0"]),
            ("SS", vec!["58", "00", "00"]),
            ("SSS", vec!["584", "000", "000"]),
            ("SSSS", vec!["5841", "0000", "0000"]),
            ("SSSSS", vec!["58418", "00000", "00000"]),
            ("SSSSSS", vec!["584186", "000000", "000000"]),
            ("SSSSSSS", vec!["5841860", "0000000", "0000000"]),
            ("SSSSSSSS", vec!["58418600", "00000000", "00000000"]),
            ("SSSSSSSSS", vec!["584186000", "000000000", "000000000"]),
            // ("VV", vec!["Z", "Z", "Z"]), // time zone ID not supported
            ("W", vec!["4", "1", "5"]),
            ("X", vec!["Z", "Z", "Z"]),
            ("XX", vec!["Z", "Z", "Z"]),
            ("XXX", vec!["Z", "Z", "Z"]),
            ("XXXX", vec!["Z", "Z", "Z"]),
            ("XXXXX", vec!["Z", "Z", "Z"]),
            ("Y", vec!["2024", "1970", "2025"]),
            ("YY", vec!["24", "70", "25"]),
            ("YYY", vec!["2024", "1970", "2025"]),
            ("YYYY", vec!["2024", "1970", "2025"]),
            ("YYYYY", vec!["02024", "01970", "02025"]),
            ("YYYYYY", vec!["002024", "001970", "002025"]),
            ("YYYYYYY", vec!["0002024", "0001970", "0002025"]),
            ("YYYYYYYY", vec!["00002024", "00001970", "00002025"]),
            ("YYYYYYYYY", vec!["000002024", "000001970", "000002025"]),
            ("Z", vec!["+0000", "+0800", "-0800"]),
            ("ZZ", vec!["+0000", "+0800", "-0800"]),
            ("ZZZ", vec!["+0000", "+0800", "-0800"]),
            ("ZZZZ", vec!["GMT", "GMT", "GMT"]),
            ("ZZZZZ", vec!["Z", "Z", "Z"]),
        ]
    }

    fn get_complex_pattern_cases() -> Vec<(&'static str, Vec<&'static str>)> {
        vec![
            ("yyyy-MM-dd", vec!["2024-07-23", "1970-01-01", "2024-12-31"]),
            ("dd/MM/yyyy", vec!["23/07/2024", "01/01/1970", "31/12/2024"]),
            (
                "MMM d, yyyy",
                vec!["Jul 23, 2024", "Jan 1, 1970", "Dec 31, 2024"],
            ),
            (
                "E, MMM d, yyyy",
                vec!["Tue, Jul 23, 2024", "Thu, Jan 1, 1970", "Tue, Dec 31, 2024"],
            ),
            ("HH:mm:ss", vec!["10:18:07", "00:00:00", "01:01:01"]),
            ("h:mm a", vec!["10:18 AM", "12:00 AM", "1:01 AM"]),
            (
                "yyyy-MM-dd'T'HH:mm:ss",
                vec![
                    "2024-07-23T10:18:07",
                    "1970-01-01T00:00:00",
                    "2024-12-31T01:01:01",
                ],
            ),
            (
                "yyyy-MM-dd'T'HH:mm:ssZ",
                vec![
                    "2024-07-23T10:18:07+0000",
                    "1970-01-01T00:00:00+0000",
                    "2024-12-31T01:01:01+0000",
                ],
            ),
            (
                "yyyy-MM-dd'T'HH:mm:ss.SSS",
                vec![
                    "2024-07-23T10:18:07.584",
                    "1970-01-01T00:00:00.000",
                    "2024-12-31T01:01:01.000",
                ],
            ),
            (
                "yyyy-MM-dd'T'HH:mm:ss.SSSZ",
                vec![
                    "2024-07-23T10:18:07.584+0000",
                    "1970-01-01T00:00:00.000+0000",
                    "2024-12-31T01:01:01.000+0000",
                ],
            ),
            (
                "yyyy-MM-dd'T'HH:mm:ss.SSSSSS",
                vec![
                    "2024-07-23T10:18:07.584186",
                    "1970-01-01T00:00:00.000000",
                    "2024-12-31T01:01:01.000000",
                ],
            ),
            (
                "yyyy-MM-dd'T'HH:mm:ss.SSSSSSZ",
                vec![
                    "2024-07-23T10:18:07.584186+0000",
                    "1970-01-01T00:00:00.000000+0000",
                    "2024-12-31T01:01:01.000000+0000",
                ],
            ),
            ("yyyy/MM/dd", vec!["2024/07/23", "1970/01/01", "2024/12/31"]),
            ("dd.MM.yyyy", vec!["23.07.2024", "01.01.1970", "31.12.2024"]),
            ("dd-MM-yy", vec!["23-07-24", "01-01-70", "31-12-24"]),
            (
                "MMMM d, yyyy",
                vec!["July 23, 2024", "January 1, 1970", "December 31, 2024"],
            ),
            (
                "y-M-d'T'H:m:s",
                vec!["2024-7-23T10:18:7", "1970-1-1T0:0:0", "2024-12-31T1:1:1"],
            ),
            (
                "Y-M-d'T'H:m:s",
                vec!["2024-7-23T10:18:7", "1970-1-1T0:0:0", "2026-12-31T1:1:1"],
            ),
            (
                "Y-M-d'T'H:m:s.SSS",
                vec![
                    "2024-7-23T10:18:7.584",
                    "1970-1-1T0:0:0.000",
                    "2026-12-31T1:1:1.000",
                ],
            ),
            (
                "Y-M-d'T'H:m:s.SSSSSS",
                vec![
                    "2024-7-23T10:18:7.584186",
                    "1970-1-1T0:0:0.000000",
                    "2026-12-31T1:1:1.000000",
                ],
            ),
            (
                "Y-M-d'T'H:m:s.SSSSSSSSS",
                vec![
                    "2024-7-23T10:18:7.584186000",
                    "1970-1-1T0:0:0.000000000",
                    "2026-12-31T1:1:1.000000000",
                ],
            ),
            ("HH % mm", vec!["10 % 18", "00 % 00", "01 % 01"]),
        ]
    }

    #[test]
    fn test_resolution_edge_cases() {
        // Expected timestamps created with Java
        let cases = vec![
            ("yyyy-MM-dd yyyy", "2023-05-15 2022", None),
            ("yyyy-MM-dd yyyy", "2023-05-15 2023", Some(1684108800000000)),
            ("yyyy-MM-dd yy", "2023-05-15 22", None),
            ("yyyy-MM-dd yy", "2023-05-15 23", Some(1684108800000000)),
            ("yyyy-MM-dd u", "2023-05-15 2022", None),
            ("yyyy-MM-dd u", "2023-05-15 2023", Some(1684108800000000)),
            ("yyyy-MM-dd MM", "2023-05-15 06", None),
            ("yyyy-MM-dd MM", "2023-05-15 05", Some(1684108800000000)),
            ("yyyy-MM-dd MMMM", "2023-05-15 June", None),
            ("yyyy-MM-dd MMMM", "2023-05-15 May", Some(1684108800000000)),
            ("yyyy-MM-dd MMM", "2023-05-15 Jun", None),
            ("yyyy-MM-dd MMM", "2023-05-15 May", Some(1684108800000000)),
            ("yyyy-MM-dd L", "2023-05-15 6", None),
            ("yyyy-MM-dd L", "2023-05-15 5", Some(1684108800000000)),
            ("yyyy-MM-dd dd", "2023-05-15 16", None),
            ("yyyy-MM-dd dd", "2023-05-15 15", Some(1684108800000000)),
            ("yyyy-MM-dd d", "2023-05-15 16", None),
            ("yyyy-MM-dd d", "2023-05-15 15", Some(1684108800000000)),
            ("yyyy-MM-dd D", "2023-05-15 136", None),
            ("yyyy-MM-dd D", "2023-05-15 135", Some(1684108800000000)),
            ("yyyy-MM-dd e", "2023-05-15 3", None),
            ("yyyy-MM-dd e", "2023-05-15 2", Some(1684108800000000)),
            ("YYYY-MM-dd yyyy", "2023-01-01 2022", None),
            ("YYYY-MM-dd yyyy", "2023-01-01 2023", Some(1672531200000000)),
            ("YYYY-ww-e yyyy", "2023-01-1 2022", None),
            ("YYYY-ww-e yyyy", "2023-01-1 2023", None),
            ("YYYY-ww-e", "2023-54-1", None),
            ("YYYY-ww-e", "2023-53-1", Some(1703376000000000)),
            ("YYYY-ww-e", "2023-52-1", Some(1703376000000000)),
            ("YYYY-ww-e", "2023-51-1", Some(1702771200000000)),
            ("YYYY-ww-e", "2023-53-2", Some(1703462400000000)),
            ("YYYY-ww-e", "2023-00-1", None),
            ("YYYY-ww-e", "2023-01-1", Some(1672531200000000)),
            ("yyyy-MM-dd", "2023-00-15", None),
            ("yyyy-MM-dd", "2023-01-15", Some(1673740800000000)),
            ("yyyy-MM-dd", "2023-13-15", None),
            ("yyyy-MM-dd", "2023-12-15", Some(1702598400000000)),
            ("yyyy-MM-dd", "2023-05-00", None),
            ("yyyy-MM-dd", "2023-05-01", Some(1682899200000000)),
            ("yyyy-MM-dd HH:mm:ss HH", "2023-05-15 14:30:45 15", None),
            (
                "yyyy-MM-dd HH:mm:ss HH",
                "2023-05-15 14:30:45 14",
                Some(1684161045000000),
            ),
            (
                "yyyy-MM-dd hh:mm:ss a hh",
                "2023-05-15 02:30:45 PM 03",
                None,
            ),
            (
                "yyyy-MM-dd hh:mm:ss a hh",
                "2023-05-15 02:30:45 PM 02",
                Some(1684161045000000),
            ),
            ("yyyy-MM-dd HH:mm:ss K", "2023-05-15 14:30:45 3", None),
            (
                "yyyy-MM-dd HH:mm:ss K",
                "2023-05-15 14:30:45 2",
                Some(1684161045000000),
            ),
            ("yyyy-MM-dd HH:mm:ss k", "2023-05-15 14:30:45 13", None),
            (
                "yyyy-MM-dd HH:mm:ss k",
                "2023-05-15 14:30:45 14",
                Some(1684161045000000),
            ),
            ("yyyy-MM-dd HH:mm:ss h", "2023-05-15 14:30:45 3", None),
            (
                "yyyy-MM-dd HH:mm:ss h",
                "2023-05-15 14:30:45 2",
                Some(1684161045000000),
            ),
            ("yyyy-MM-dd HH:mm:ss mm", "2023-05-15 14:30:45 31", None),
            (
                "yyyy-MM-dd HH:mm:ss mm",
                "2023-05-15 14:30:45 30",
                Some(1684161045000000),
            ),
            ("yyyy-MM-dd HH:mm:ss", "2023-05-15 14:60:45", None),
            (
                "yyyy-MM-dd HH:mm:ss",
                "2023-05-15 14:59:45",
                Some(1684162785000000),
            ),
            ("yyyy-MM-dd HH:mm:ss", "2023-05-15 14:-1:45", None),
            (
                "yyyy-MM-dd HH:mm:ss",
                "2023-05-15 14:00:45",
                Some(1684159245000000),
            ),
            ("yyyy-MM-dd HH:mm:ss ss", "2023-05-15 14:30:45 46", None),
            (
                "yyyy-MM-dd HH:mm:ss ss",
                "2023-05-15 14:30:45 45",
                Some(1684161045000000),
            ),
            ("yyyy-MM-dd HH:mm:ss", "2023-05-15 14:30:60", None),
            (
                "yyyy-MM-dd HH:mm:ss",
                "2023-05-15 14:30:59",
                Some(1684161059000000),
            ),
            ("yyyy-MM-dd HH:mm:ss", "2023-05-15 14:30:-1", None),
            (
                "yyyy-MM-dd HH:mm:ss",
                "2023-05-15 14:30:00",
                Some(1684161000000000),
            ),
            ("yyyy-MM-dd hh:mm:ss a a", "2023-05-15 02:30:45 PM AM", None),
            (
                "yyyy-MM-dd hh:mm:ss a a",
                "2023-05-15 02:30:45 PM PM",
                Some(1684161045000000),
            ),
            ("yyyy-MM-dd HH:mm:ss a", "2023-05-15 14:30:45 AM", None),
            (
                "yyyy-MM-dd HH:mm:ss a",
                "2023-05-15 14:30:45 PM",
                Some(1684161045000000),
            ),
            ("yyyy-MM-dd hh:mm:ss", "2023-05-15 14:30:45", None),
            (
                "yyyy-MM-dd hh:mm:ss",
                "2023-05-15 02:30:45",
                Some(1684108800000000),
            ),
            (
                "yyyy-MM-dd hh:mm:ss a",
                "2023-05-15 00:30:45 PM",
                Some(1684153845000000),
            ),
            (
                "yyyy-MM-dd hh:mm:ss a",
                "2023-05-15 12:30:45 PM",
                Some(1684153845000000),
            ),
            ("yyyy-MM-dd hh:mm:ss a", "2023-05-15 13:30:45 PM", None),
            (
                "yyyy-MM-dd hh:mm:ss a",
                "2023-05-15 01:30:45 PM",
                Some(1684157445000000),
            ),
            ("yyyy-MM-dd HH:mm:ss", "2023-05-15 24:30:45", None),
            (
                "yyyy-MM-dd HH:mm:ss",
                "2023-05-15 23:30:45",
                Some(1684193445000000),
            ),
            ("yyyy-MM-dd HH:mm:ss", "2023-05-15 -1:30:45", None),
            (
                "yyyy-MM-dd HH:mm:ss",
                "2023-05-15 00:30:45",
                Some(1684110645000000),
            ),
            ("yyyy-MM-dd hh:mm:ss a", "2023-05-15 13:30:45 PM", None),
            (
                "yyyy-MM-dd hh:mm:ss a",
                "2023-05-15 01:30:45 PM",
                Some(1684157445000000),
            ),
            ("yyyy-MM-dd kk:mm:ss", "2023-05-15 25:30:45", None),
            (
                "yyyy-MM-dd kk:mm:ss",
                "2023-05-15 24:30:45",
                Some(1684110645000000),
            ),
            ("yyyy-MM-dd G G", "2023-05-15 AD BC", None),
            ("yyyy-MM-dd G G", "2023-05-15 AD AD", Some(1684108800000000)),
            ("GG yyyy-MM-dd", "ADAD 2023-05-15", None),
            ("GG yyyy-MM-dd", "AD 2023-05-15", Some(1684108800000000)),
            ("yyyy-MM-dd E", "2023-05-15 Tuesday", None),
            ("yyyy-MM-dd E", "2023-05-15 Monday", None),
            ("yyyy-MM-dd w", "2023-05-15 21", None),
            ("yyyy-MM-dd w", "2023-05-15 20", Some(1684108800000000)),
            ("YYYY-MM-dd ww-e", "2023-05-15 20-1", None),
            ("YYYY-MM-dd ww-e", "2023-05-15 20-2", Some(1684108800000000)),
            ("w yyyy-MM-dd", "21 2023-05-15", None),
            ("w yyyy-MM-dd", "20 2023-05-15", Some(1684108800000000)),
            ("yyyy-MM-dd W", "2023-05-15 2", None),
            ("yyyy-MM-dd W", "2023-05-15 3", Some(1684108800000000)),
            ("yyyy-DDD", "2023-366", None),
            ("yyyy-DDD", "2023-365", Some(1703980800000000)),
            ("yyyy-DDD", "2020-367", None),
            ("yyyy-DDD", "2020-366", Some(1609372800000000)),
            ("yyyy-MM-dd Q", "2023-05-15 3", None),
            ("yyyy-MM-dd Q", "2023-05-15 2", Some(1684108800000000)),
            ("yyyy-MM-dd QQQ", "2023-05-15 Q3", None),
            ("yyyy-MM-dd QQQ", "2023-05-15 Q2", Some(1684108800000000)),
            ("yyyy-MM-dd QQQQ", "2023-05-15 3rd quarter", None),
            (
                "yyyy-MM-dd QQQQ",
                "2023-05-15 2nd quarter",
                Some(1684108800000000),
            ),
            (
                "yyyy-MM-dd'T'HH:mm:ssXXX XXX",
                "2023-05-15T14:30:45+01:00 +02:00",
                None,
            ),
            (
                "yyyy-MM-dd'T'HH:mm:ssXXX XXX",
                "2023-05-15T14:30:45+01:00 +01:00",
                Some(1684157445000000),
            ),
            // (
            //     "yyyy-MM-dd'T'HH:mm:ssz z",
            //     "2023-05-15T14:30:45GMT EST",
            //     Some(1684175445000000),
            // ),
            // (
            //     "yyyy-MM-dd'T'HH:mm:ssz z",
            //     "2023-05-15T14:30:45GMT UTC",
            //     Some(1684161045000000),
            // ),
            (
                "yyyy-MM-dd'T'HH:mm:ssZ Z",
                "2023-05-15T14:30:45+0100 +0200",
                None,
            ),
            (
                "yyyy-MM-dd'T'HH:mm:ssZ Z",
                "2023-05-15T14:30:45+0100 +0100",
                Some(1684157445000000),
            ),
            // (
            //     "yyyy-MM-dd'T'HH:mm:ssZ z",
            //     "2023-05-15T14:30:45+0100 UTC",
            //     Some(1684157445000000),
            // ),
            // (
            //     "yyyy-MM-dd'T'HH:mm:ssZ z",
            //     "2023-05-15T14:30:45+0000 +0100",
            //     None,
            // ),
            // (
            //     "yyyy-MM-dd'T'HH:mm:ss VV VV",
            //     "2023-05-15T14:30:45 Europe/Paris Europe/London",
            //     Some(1684157445000000),
            // ),
            // (
            //     "yyyy-MM-dd'T'HH:mm:ss VV VV",
            //     "2023-05-15T14:30:45 Europe/Paris Europe/Paris",
            //     Some(1684153845000000),
            // ),
            (
                "yyyy-MM-dd'T'HH:mm:ssO O",
                "2023-05-15T14:30:45GMT+1 GMT+2",
                None,
            ),
            (
                "yyyy-MM-dd'T'HH:mm:ssO O",
                "2023-05-15T14:30:45GMT+1 GMT+1",
                Some(1684157445000000),
            ),
            (
                "yyyy-MM-dd'T'HH:mm:ssO O",
                "2023-05-15T14:30:45GMT+1 UTC+1",
                None,
            ),
            (
                "yyyy-MM-dd'T'HH:mm:ssO O",
                "2023-05-15T14:30:45GMT+1 GMT+1",
                Some(1684157445000000),
            ),
            ("yyyy-MM-dd", "2023-O5-15", None),
            ("yyyy-MM-dd", "2023-05-15", Some(1684108800000000)),
            ("yyyy-MM-dd", "2023-05-I5", None),
            ("yyyy-MM-dd", "2023-05-15", Some(1684108800000000)),
            ("yyyy-MM-dd", "2O23-05-15", None),
            ("yyyy-MM-dd", "2023-05-15", Some(1684108800000000)),
            ("yyyy/MM/dd", "2023/05/15extra", None),
            ("yyyy/MM/dd", "2023/05/15", Some(1684108800000000)),
            ("'Date: 'yyyy-MM-dd", "2023-05-15", None),
            (
                "'Date: 'yyyy-MM-dd",
                "Date: 2023-05-15",
                Some(1684108800000000),
            ),
        ];

        let mut failures = 0;
        for case in &cases {
            let (pattern, str, maybe_timestamp_micros) = case;
            let formatter = match DateTimeFormatter::of_pattern(pattern) {
                Ok(formatter) => formatter,
                Err(e) => {
                    println!("Failed to create formatter for pattern {pattern}: {e}");
                    failures += 1;
                    continue;
                }
            };
            let parsed = formatter.parse(str);
            println!("Pattern: {pattern} string: {str} - parsed: {parsed:?}");
            let ts = parsed.and_then(|p| p.to_epoch_microseconds());
            let printed = ts
                .clone()
                .and_then(|micros| formatter.format(&Parsed::of_epoch_microseconds(micros)));
            let expected_printed = maybe_timestamp_micros.and_then(|micros| {
                formatter
                    .format(&Parsed::of_epoch_microseconds(micros))
                    .ok()
            });
            match (&ts, &printed, &maybe_timestamp_micros, &expected_printed) {
                (Ok(micros), Ok(printed), Some(expected_micros), Some(expected_printed)) => {
                    if micros != expected_micros {
                        println!("Format: {pattern} string: {str} - got {micros:?} / {printed} but expected {expected_micros} / {expected_printed}");
                        failures += 1;
                    }
                }
                (Ok(micros), Ok(printed), None, _) => {
                    println!("Format: {pattern} string: {str} - got {micros:?} / {printed} but wasn't supposed to compute");
                    failures += 1;
                }
                (Err(e), _, Some(expected_micros), Some(expected_printed)) => {
                    println!("Format: {pattern} string: {str} - failed to resolve but expected {expected_micros} / {expected_printed}: {e}");
                    failures += 1;
                }
                (Err(_), _, None, _) => {}
                _ => println!("Unexpected case: {ts:?} {printed:?} {maybe_timestamp_micros:?}"),
            }
        }
        println!("Total failures: {failures} out of {} cases", cases.len());
        assert_eq!(failures, 0);
    }
}
