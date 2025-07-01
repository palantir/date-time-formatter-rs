#[derive(Clone, Debug)]
pub struct DecimalStyle {
    zero_digit: char,
    positive_sign: char,
    negative_sign: char,
    decimal_separator: char,
}
impl DecimalStyle {
    pub const STANDARD: DecimalStyle = DecimalStyle {
        zero_digit: '0',
        positive_sign: '+',
        negative_sign: '-',
        decimal_separator: '.',
    };

    pub fn get_positive_sign(&self) -> char {
        self.positive_sign
    }

    pub fn get_negative_sign(&self) -> char {
        self.negative_sign
    }

    pub fn get_zero_digit(&self) -> char {
        self.zero_digit
    }

    pub fn get_decimal_separator(&self) -> char {
        self.decimal_separator
    }

    pub fn convert_to_digit(&self, ch: char) -> i64 {
        // Convert characters to their Unicode scalar values
        let ch_value = ch as i64;
        let zero_value = self.zero_digit as i64;

        // Calculate the difference
        let val = ch_value - zero_value;

        // Check if the result is in the valid digit range (0-9)
        if val <= 9 {
            val
        } else {
            -1
        }
    }

    pub fn convert_number_to_i18n(&self, numeric_text: &str) -> Result<String, String> {
        if self.zero_digit == '0' {
            return Ok(numeric_text.to_owned());
        }

        Err("Zero digit should always be 0 since we don't support other locales".to_owned())
    }
}
