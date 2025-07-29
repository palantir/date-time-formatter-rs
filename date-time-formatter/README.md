## About date-time-formatter

'date-time-formatter' is a Rust crate that provides a simple and efficient way to parse and format date and time values. It is a rewrite of a subset of the Java time library, and aims to match its behaviour.

This ports DateTimeFormatterBuilder 1:1 from Java into Rust, with the following simplifying assumptions:

- ResolverStyle is set to SMART
- CaseSensitivity is set to false
- Locale is always assumed to be U.S.
- We only support the default Gregorian Chronology

Additionally, this provides logic for converting from the Parsed representation into a Chrono (Naive)DateTime, and back. The resolution from Parsed into Chrono aims to replicate the results of Java equivalents, but is not 1:1 port.

## Example usage

```rust
let parsing_formatter = DateTimeFormatter::of_pattern("yyyy-MM-dd'T'hh:mm:ss")?;
let printing_formatter = DateTimeFormatter::of_pattern(
    "'Year:' yyyy, 'Month:' MM, 'Date:' dd, 'Hour:' hh, 'Minute:' mm, 'Second:' ss",
)?;
let original_string = "2025-05-13T14:30:00";
let parsed: Parsed = parsing_formatter.parse(original_string)?;
let recovered_string = printing_formatter.format(&parsed)?;
assert_eq!(
    "Year: 2025, Month: 05, Date: 13, Hour: 14, Minute: 30, Second: 00",
    recovered_string
);
```

## Contact

Created by [Shiv Bhatia](sbhatia@palantir.com)
