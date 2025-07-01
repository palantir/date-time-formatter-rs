#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalUnit {
    // From ChronoUnit
    Nanos,
    Micros,
    Millis,
    Seconds,
    Minutes,
    Hours,
    HalfDays,
    Days,
    Weeks,
    Months,
    Years,
    Decades,
    Centuries,
    Millennia,
    Eras,
    Forever,

    // From IsoFields
    WeekBasedYears,
    QuarterYears,
}
