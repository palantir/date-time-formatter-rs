#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period {
    pub years: i32,
    pub months: i32,
    pub days: i32,
}

impl Period {
    pub fn new(years: i32, months: i32, days: i32) -> Self {
        Period {
            years,
            months,
            days,
        }
    }

    pub fn of_years(years: i32) -> Self {
        Period::new(years, 0, 0)
    }

    pub fn of_months(months: i32) -> Self {
        Period::new(0, months, 0)
    }

    pub fn of_days(days: i32) -> Self {
        Period::new(0, 0, days)
    }

    pub fn zero() -> Self {
        Period::new(0, 0, 0)
    }

    pub fn is_zero(&self) -> bool {
        self.years == 0 && self.months == 0 && self.days == 0
    }
}
