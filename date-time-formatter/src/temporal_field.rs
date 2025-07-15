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

use std::collections::HashMap;

use chrono::NaiveDate;

use crate::{
    temporal_unit::TemporalUnit,
    value_range::ValueRange,
    week_fields_utils::{
        floor_mod, get_first_day_of_week, localized_day_of_week, localized_day_of_week_of_iso_dow,
        localized_week_of_month, localized_week_of_year, of_week_based_year,
    },
};

const JULIAN_DAY_OFFSET: i64 = 40587;
pub const JULIAN_DAY_TO_MODIFIED_JULIAN_DAY_OFFSET: i64 = 2400000;
pub const JULIAN_DAY_TO_CE_DAYS: i64 = 1721424;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum TemporalField {
    // From ChronoField
    NanoOfSecond,
    SecondOfMinute,
    MinuteOfHour,
    HourOfDay,
    ClockHourOfDay,
    AmPmOfDay,
    DayOfWeek, // This is the ISO day of week, 1 is Monday, 7 is Sunday
    DayOfMonth,
    DayOfYear,
    MonthOfYear,
    Year,
    YearOfEra,
    Era,
    AlignedDayOfWeekInMonth,
    HourOfAmPm,
    ClockHourOfAmPm,
    NanoOfDay,
    MilliOfDay,
    // NOTE: These fields are used for resolution and not for actual time representation
    MilliOfSecond,
    MicroOfSecond,
    MicroOfDay,
    SecondOfDay,
    MinuteOfDay,
    // NOTE: OffsetSeconds is neither a date nor time field
    OffsetSeconds,

    // From WeekFields
    // NOTE: LocaleDayOfWeek always overrides DayOfWeek in the field map when
    // we encounter e/c pattern chars. The get_temporal_field function below is
    // somewhat misleading in that it returns DayOfWeek for both e and c pattern chars,
    // but this is the behaviour of the original Java code so we've kept it for now.
    LocaleDayOfWeek,
    WeekBasedYear,
    WeekOfWeekBasedYear,
    WeekOfMonth,

    // From IsoFields
    QuarterOfYear,

    // From JulianFields
    ModifiedJulianDay,
}

impl TemporalField {
    pub fn get_base_unit(&self) -> TemporalUnit {
        self.get_properties().0
    }

    pub fn get_range_unit(&self) -> TemporalUnit {
        self.get_properties().1
    }

    pub fn range(&self) -> ValueRange {
        self.get_properties().2
    }

    fn get_properties(&self) -> (TemporalUnit, TemporalUnit, ValueRange) {
        // Properties are of the form:
        // (base_unit, range_unit, value_range)

        use TemporalField::*;
        use TemporalUnit::*;

        match self {
            NanoOfSecond => (Nanos, Seconds, ValueRange::of(0, 999_999_999)),
            SecondOfMinute => (Seconds, Minutes, ValueRange::of(0, 59)),
            MinuteOfHour => (Minutes, Hours, ValueRange::of(0, 59)),
            HourOfDay => (Hours, Days, ValueRange::of(0, 23)),
            ClockHourOfDay => (Hours, Days, ValueRange::of(1, 24)),
            AmPmOfDay => (HalfDays, Days, ValueRange::of(0, 1)),
            DayOfWeek => (
                Days,
                Weeks,
                ValueRange::of(1, 7), // This is ISO day of week, 1 is Monday, 7 is Sunday
            ),
            DayOfMonth => (
                Days,
                Months,
                ValueRange::of_with_max_smallest_and_largest(1, 28, 31),
            ),
            DayOfYear => (
                Days,
                Years,
                ValueRange::of_with_max_smallest_and_largest(1, 365, 366),
            ),
            MonthOfYear => (Months, Years, ValueRange::of(1, 12)),
            Year => (Years, Forever, ValueRange::of(-999_999_999, 999_999_999)),
            YearOfEra => (
                Years,
                Forever,
                ValueRange::of_with_max_smallest_and_largest(1, 999_999_999, 999_999_999 + 1),
            ),
            Era => (Eras, Forever, ValueRange::of(0, 1)),
            AlignedDayOfWeekInMonth => (Days, Weeks, ValueRange::of(1, 7)),
            HourOfAmPm => (Hours, HalfDays, ValueRange::of(0, 11)),
            ClockHourOfAmPm => (Hours, HalfDays, ValueRange::of(1, 12)),
            MilliOfDay => (Millis, Days, ValueRange::of(0, 86400 * 1000 - 1)),
            MilliOfSecond => (Millis, Seconds, ValueRange::of(0, 999)),
            MicroOfDay => (Micros, Days, ValueRange::of(0, 86400 * 1_000_000 - 1)),
            SecondOfDay => (Seconds, Days, ValueRange::of(0, 86400 - 1)),
            MicroOfSecond => (Micros, Seconds, ValueRange::of(0, 999_999)),
            MinuteOfDay => (Minutes, Days, ValueRange::of(0, 1440 - 1)),
            NanoOfDay => (Nanos, Days, ValueRange::of(0, 86400 * 1_000_000_000 - 1)),
            OffsetSeconds => (Seconds, Forever, ValueRange::of(-18 * 3600, 18 * 3600)),
            LocaleDayOfWeek => (
                Days,
                Weeks,
                ValueRange::of(1, 7), // In the US locale, 1 is Sunday, 7 is Saturday
            ),
            WeekBasedYear => (
                WeekBasedYears,
                Forever,
                ValueRange::of(-999_999_999, 999_999_999),
            ),
            WeekOfWeekBasedYear => (
                Weeks,
                WeekBasedYears,
                ValueRange::of_with_max_smallest_and_largest(1, 52, 53),
            ),
            WeekOfMonth => (
                Weeks,
                Months,
                ValueRange::of_with_max_smallest_and_largest(1, 4, 5),
            ),
            QuarterOfYear => (QuarterYears, Years, ValueRange::of(1, 4)),
            ModifiedJulianDay => (
                Days,
                Forever,
                ValueRange::of(
                    -365243219162 + JULIAN_DAY_OFFSET,
                    365241780471 + JULIAN_DAY_OFFSET,
                ),
            ),
        }
    }

    pub fn is_date_based(&self) -> bool {
        use TemporalField::*;
        matches!(
            self,
            DayOfWeek
                | DayOfMonth
                | DayOfYear
                | MonthOfYear
                | Year
                | YearOfEra
                | Era
                | AlignedDayOfWeekInMonth
                | LocaleDayOfWeek
                | WeekBasedYear
                | WeekOfWeekBasedYear
                | WeekOfMonth
                | QuarterOfYear
                | ModifiedJulianDay
        )
    }

    pub fn is_time_based(&self) -> bool {
        use TemporalField::*;
        matches!(
            self,
            NanoOfSecond
                | SecondOfMinute
                | MinuteOfHour
                | HourOfDay
                | ClockHourOfDay
                | AmPmOfDay
                | HourOfAmPm
                | ClockHourOfAmPm
                | MilliOfDay
                | NanoOfDay
        )
    }

    pub fn resolve(
        &self,
        field_values: &mut HashMap<TemporalField, i64>,
    ) -> Result<Option<NaiveDate>, String> {
        // NOTE: We don't need partial_temporal because it is only used to get the Chronology,
        // and we only support ISO Chronology for now.
        // We also only support the strict resolver style for now.
        // Only WeekFields and JulianFields override this method, the rest return None since
        // that's the default implementation.
        // Also, the original returns a TemporalAccessor, we got lucky in the sense that
        // the temporal fields we care about only resolve to a NaiveDate.
        match self {
            // This is the resolve() method for all WeekFields TemporalFields.
            TemporalField::WeekBasedYear
            | TemporalField::WeekOfWeekBasedYear
            | TemporalField::WeekOfMonth
            | TemporalField::LocaleDayOfWeek => {
                let &value = field_values
                    .get(self)
                    .ok_or(format!("Expected {:?} to be present", self))?;
                let new_value = value as i32;
                if self.get_range_unit() == TemporalUnit::Weeks {
                    // NOTE: This is where we should resolve LocaleDayOfWeek to DayOfWeek
                    let checked_value = self.range().check_valid_int_value(value)?;
                    let start_dow = get_first_day_of_week();
                    let iso_dow = floor_mod((start_dow - 1) + (checked_value - 1), 7) + 1;
                    field_values.remove(self);
                    field_values.insert(TemporalField::DayOfWeek, iso_dow as i64);
                    return Ok(None);
                }

                // can only build date if ISO day-of-week
                if !field_values.contains_key(&TemporalField::DayOfWeek) {
                    return Ok(None);
                }
                let iso_dow = TemporalField::DayOfWeek.range().check_valid_int_value(
                    *field_values
                        .get(&TemporalField::DayOfWeek)
                        .ok_or("Expected DayOfWeek to be present")?,
                )?;
                let dow = localized_day_of_week_of_iso_dow(iso_dow);

                // build date
                if field_values.contains_key(&TemporalField::Year) {
                    let year = TemporalField::Year.range().check_valid_int_value(
                        *field_values
                            .get(&TemporalField::Year)
                            .ok_or("Expected Year to be present")?,
                    )?;

                    if self.get_range_unit() == TemporalUnit::Months
                        && field_values.contains_key(&TemporalField::MonthOfYear)
                    {
                        let &month = field_values
                            .get(&TemporalField::MonthOfYear)
                            .ok_or("Expected MonthOfYear to be present")?; // not validated yet
                        return self.resolve_wom(field_values, year, month, new_value as i64, dow);
                    }
                    if self.get_range_unit() == TemporalUnit::Years {
                        return self.resolve_woy(field_values, year, new_value, dow);
                    }
                } else if (self.get_range_unit() == TemporalUnit::WeekBasedYears
                    || self.get_range_unit() == TemporalUnit::Forever)
                    && field_values.contains_key(&TemporalField::WeekBasedYear)
                    && field_values.contains_key(&TemporalField::WeekOfWeekBasedYear)
                {
                    return self.resolve_wby(field_values, dow);
                }

                Ok(None)
            }

            TemporalField::ModifiedJulianDay => {
                let value = field_values
                    .remove(self)
                    .ok_or("Expected ModifiedJulianDay to be present")?;
                self.range().check_valid_value(value)?;
                Ok(NaiveDate::from_num_days_from_ce_opt(
                    (value + JULIAN_DAY_TO_MODIFIED_JULIAN_DAY_OFFSET - JULIAN_DAY_TO_CE_DAYS)
                        as i32,
                ))
            }

            _ => Ok(None),
        }
    }

    fn resolve_wom(
        &self,
        field_values: &mut HashMap<TemporalField, i64>,
        year: i32,
        month: i64,
        wom: i64,
        local_dow: i32,
    ) -> Result<Option<NaiveDate>, String> {
        let month_valid = TemporalField::MonthOfYear
            .range()
            .check_valid_int_value(month)?;
        let mut date = NaiveDate::from_ymd_opt(year, month_valid as u32, 1)
            .ok_or("Could not construct date from given year and month")?;
        let wom_int = self.range().check_valid_int_value(wom)?;
        let weeks = wom_int - localized_week_of_month(date);
        let days = local_dow - localized_day_of_week(date);
        date += chrono::Duration::days((weeks * 7 + days) as i64);

        field_values.remove(self);
        field_values.remove(&TemporalField::Year);
        field_values.remove(&TemporalField::MonthOfYear);
        field_values.remove(&TemporalField::DayOfWeek);
        Ok(Some(date))
    }

    fn resolve_woy(
        &self,
        field_values: &mut HashMap<TemporalField, i64>,
        year: i32,
        woy: i32,
        local_dow: i32,
    ) -> Result<Option<NaiveDate>, String> {
        let mut date = NaiveDate::from_ymd_opt(year, 1, 1)
            .ok_or("Could not construct date from given year")?;
        let wom_int = self.range().check_valid_int_value(woy as i64)?;
        let weeks = wom_int - localized_week_of_year(date);
        let days = local_dow - localized_day_of_week(date);
        date += chrono::Duration::days((weeks * 7 + days) as i64);

        field_values.remove(self);
        field_values.remove(&TemporalField::Year);
        field_values.remove(&TemporalField::DayOfWeek);
        Ok(Some(date))
    }

    fn resolve_wby(
        &self,
        field_values: &mut HashMap<TemporalField, i64>,
        local_dow: i32,
    ) -> Result<Option<NaiveDate>, String> {
        let yowby = TemporalField::WeekBasedYear.range().check_valid_int_value(
            *field_values
                .get(&TemporalField::WeekBasedYear)
                .ok_or("Expected WeekBasedYear to be present")?,
        )?;
        let wowby = TemporalField::WeekOfWeekBasedYear
            .range()
            .check_valid_int_value(
                *field_values
                    .get(&TemporalField::WeekOfWeekBasedYear)
                    .ok_or("Expected WeekOfWeekBasedYear to be present")?,
            )?;
        let date = of_week_based_year(yowby, wowby, local_dow)?;

        field_values.remove(self);
        field_values.remove(&TemporalField::WeekBasedYear);
        field_values.remove(&TemporalField::WeekOfWeekBasedYear);
        field_values.remove(&TemporalField::DayOfWeek);
        Ok(Some(date))
    }
}

pub fn get_temporal_field(c: &char) -> Option<TemporalField> {
    use TemporalField::*;
    match c {
        'A' => Some(MilliOfDay),
        'D' => Some(DayOfYear),
        'E' => Some(DayOfWeek),
        'F' => Some(AlignedDayOfWeekInMonth),
        'G' => Some(Era),
        'H' => Some(HourOfDay),
        'K' => Some(HourOfAmPm),
        'M' | 'L' => Some(MonthOfYear),
        'N' => Some(NanoOfDay),
        'Q' | 'q' => Some(QuarterOfYear),
        'S' | 'n' => Some(NanoOfSecond),
        'a' => Some(AmPmOfDay),
        'd' => Some(DayOfMonth),
        'e' | 'c' => Some(DayOfWeek), // Although this is later set to LocaleDayOfWeek, it is used as DayOfWeek in the field map
        'g' => Some(ModifiedJulianDay),
        'h' => Some(ClockHourOfAmPm),
        'k' => Some(ClockHourOfDay),
        'm' => Some(MinuteOfHour),
        's' => Some(SecondOfMinute),
        'u' => Some(Year),
        'y' => Some(YearOfEra),
        _ => None,
    }
}
