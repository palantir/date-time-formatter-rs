pub struct ValueRange {
    min_smallest: i64,
    _min_largest: i64,
    _max_smallest: i64,
    max_largest: i64,
}

impl ValueRange {
    pub fn of(min: i64, max: i64) -> Self {
        ValueRange {
            min_smallest: min,
            _min_largest: min,
            _max_smallest: max,
            max_largest: max,
        }
    }

    pub fn of_with_max_smallest_and_largest(min: i64, max_smallest: i64, max_largest: i64) -> Self {
        ValueRange {
            min_smallest: min,
            _min_largest: min,
            _max_smallest: max_smallest,
            max_largest,
        }
    }

    pub fn check_valid_value(&self, value: i64) -> Result<i64, String> {
        if !(self.min_smallest..=self.max_largest).contains(&value) {
            return Err(format!(
                "Value {} is outside the range [{}, {}]",
                value, self.min_smallest, self.max_largest
            ));
        }
        Ok(value)
    }

    pub fn check_valid_int_value(&self, value: i64) -> Result<i32, String> {
        if !(self.min_smallest..=self.max_largest).contains(&value) {
            return Err(format!(
                "Value {} is outside the range [{}, {}]",
                value, self.min_smallest, self.max_largest
            ));
        }
        if value < i32::MIN as i64 || value > i32::MAX as i64 {
            return Err(format!("Value {} is outside the range of i32", value));
        }
        Ok(value as i32)
    }
}
