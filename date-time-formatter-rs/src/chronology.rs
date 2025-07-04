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

use chrono::{Datelike, NaiveDate};

use crate::temporal_field::TemporalField;

// Placeholder for Chronology, currently we only support ISO Chronology
pub struct Chronology {}

impl Chronology {
    pub fn resolve_date(
        field_values: &mut HashMap<TemporalField, i64>,
    ) -> Result<Option<NaiveDate>, String> {
        // invent era if necessary to resolve year of era
        let resolved = Self::resolve_year_of_era(field_values)?;
        if let Some(resolved) = resolved {
            // NOTE: We should never actually be in this branch,
            // as resolve_year_of_era should always return None
            // We've kept this for consistency with the original code.
            return Ok(Some(resolved));
        }

        // build date
        if field_values.contains_key(&TemporalField::Year) {
            if field_values.contains_key(&TemporalField::MonthOfYear)
                && field_values.contains_key(&TemporalField::DayOfMonth)
            {
                return Self::resolve_ymd(field_values);
            }
            if field_values.contains_key(&TemporalField::DayOfYear) {
                return Self::resolve_yd(field_values);
            }
            // We don't have AlignedWeekOfYear
        }

        Ok(None)
    }

    // NOTE: This function will always return Ok(None) if it does not error,
    // however in the original code it is treated as if it could return a date.
    // For consistency with the original code, we keep this signature.
    pub fn resolve_year_of_era(
        field_values: &mut HashMap<TemporalField, i64>,
    ) -> Result<Option<NaiveDate>, String> {
        let yoe_long = field_values.remove(&TemporalField::YearOfEra);
        if let Some(yoe_long) = yoe_long {
            let yoe = TemporalField::YearOfEra
                .range()
                .check_valid_int_value(yoe_long)?;
            if let Some(era) = field_values.remove(&TemporalField::Era) {
                let era = TemporalField::Era.range().check_valid_int_value(era)?;
                Self::add_field_value(
                    field_values,
                    TemporalField::Year,
                    Self::proleptic_year(era, yoe),
                )?;
            } else if let Some(&year) = field_values.get(&TemporalField::Year) {
                let year = TemporalField::Year.range().check_valid_int_value(year)?;
                let chrono_date = NaiveDate::from_yo_opt(year, 1)
                    .ok_or("Could not construct date from given year")?;
                Self::add_field_value(
                    field_values,
                    TemporalField::Year,
                    Self::proleptic_year(if chrono_date.year() > 0 { 1 } else { 0 }, yoe),
                )?;
            } else {
                // We default to CE here
                Self::add_field_value(
                    field_values,
                    TemporalField::Year,
                    Self::proleptic_year(1, yoe),
                )?;
            }
        } else if field_values.contains_key(&TemporalField::Era) {
            TemporalField::Era
                .range()
                .check_valid_int_value(field_values[&TemporalField::Era])?; // always validated
        }

        Ok(None)
    }

    fn resolve_ymd(
        field_values: &mut HashMap<TemporalField, i64>,
    ) -> Result<Option<NaiveDate>, String> {
        let y = TemporalField::Year.range().check_valid_int_value(
            field_values
                .remove(&TemporalField::Year)
                .ok_or("Expected Year to be present")?,
        )?;
        let moy = TemporalField::MonthOfYear.range().check_valid_int_value(
            field_values
                .remove(&TemporalField::MonthOfYear)
                .ok_or("Expected MonthOfYear to be present")?,
        )?;
        let dom_range = TemporalField::DayOfMonth.range();
        let dom = dom_range.check_valid_int_value(
            field_values
                .remove(&TemporalField::DayOfMonth)
                .ok_or("Expected DayOfMonth to be present")?,
        )?;
        if let Some(date) = NaiveDate::from_ymd_opt(y, moy as u32, dom as u32) {
            // If the date is valid, we can return it
            Ok(Some(date))
        } else {
            // NOTE: We assume we are always in Smart ResolverStyle
            // This is the equivalent of using TemporalAdjusters.lastDayOfMonth()
            let date = Self::last_day_of_month(y, moy as u32)
                .ok_or("Invalid date: could not find last day of month")?;
            Ok(Some(date))
        }
    }

    fn resolve_yd(
        field_values: &mut HashMap<TemporalField, i64>,
    ) -> Result<Option<NaiveDate>, String> {
        // Unlike resolve_ymd, if the day of year is greater than the last day of the year,
        // we just return an error, we don't adjust to the last day of the year.
        let y = TemporalField::Year.range().check_valid_int_value(
            field_values
                .remove(&TemporalField::Year)
                .ok_or("Expected Year to be present")?,
        )?;
        let doy = TemporalField::DayOfYear.range().check_valid_int_value(
            field_values
                .remove(&TemporalField::DayOfYear)
                .ok_or("Expected DayOfYear to be present")?,
        )?;
        let date = NaiveDate::from_yo_opt(y, doy as u32)
            .ok_or("Invalid date: could not construct date from year and day of year")?;
        Ok(Some(date))
    }

    fn proleptic_year(era: i32, yoe: i32) -> i64 {
        if era == 0 {
            return (1 - yoe) as i64;
        }
        yoe as i64
    }

    fn add_field_value(
        field_values: &mut HashMap<TemporalField, i64>,
        field: TemporalField,
        value: i64,
    ) -> Result<(), String> {
        let old = field_values.get(&field);
        if let Some(&old_value) = old {
            if old_value != value {
                return Err(format!(
                    "Field {:?} already has value {}, cannot set to {}",
                    field, old_value, value
                ));
            }
        }
        field_values.insert(field, value);
        Ok(())
    }

    fn last_day_of_month(year: i32, month: u32) -> Option<NaiveDate> {
        let days_in_month = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if NaiveDate::from_ymd_opt(year, 2, 29).is_some() {
                    29
                } else {
                    28
                }
            }
            _ => return None,
        };
        NaiveDate::from_ymd_opt(year, month, days_in_month)
    }
}
