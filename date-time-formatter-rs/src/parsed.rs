use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use chrono::{
    DateTime, Datelike, Duration, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Offset,
    TimeZone, Timelike, Utc,
};

use crate::{
    chronology::Chronology,
    period::Period,
    resolver_style::ResolverStyle,
    temporal_field::{
        TemporalField, JULIAN_DAY_TO_CE_DAYS, JULIAN_DAY_TO_MODIFIED_JULIAN_DAY_OFFSET,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parsed {
    pub field_values: HashMap<TemporalField, i64>,
    pub zone_id: Option<String>,
    pub leap_second: bool,
    pub date: Option<NaiveDate>,
    pub time: Option<NaiveTime>,
    pub excess_days: Period,
    pub resolver_style: ResolverStyle,
}

impl Parsed {
    pub fn new() -> Self {
        Parsed {
            field_values: HashMap::new(),
            zone_id: None,
            leap_second: false,
            date: None,
            time: None,
            excess_days: Period::zero(),
            resolver_style: ResolverStyle::Smart, // Default resolver style
        }
    }

    pub fn resolve(
        &mut self,
        resolver_style: &ResolverStyle,
        _resolver_fields: &HashSet<TemporalField>,
    ) -> Result<Self, String> {
        // Ignore resolve_fields for now, in the original if it's null we ignore it
        // and it always seems to be null.
        // This means we always use all the fields we're parsed.
        self.resolver_style = *resolver_style;
        self.resolve_fields()?;
        self.resolve_time_lenient()?;
        self.cross_check()?;
        self.resolve_period()?;
        self.resolve_fractional()?;
        // self.resolve_instant()?; Don't think we need this
        Ok(self.clone())
    }

    fn resolve_fields(&mut self) -> Result<(), String> {
        // resolve ChronoField
        self.resolve_date_fields()?;
        self.resolve_time_fields()?;

        // if any other fields, handle them
        // any lenient date resolution should return epoch-day

        if !self.field_values.is_empty() {
            let mut changed_count = 0;
            // The limit of 50 changes is arbitrary but prevents infinite loops
            'outer: while changed_count < 50 {
                // Clone keys to avoid borrowing issues when modifying the map
                let keys: Vec<_> = self.field_values.keys().cloned().collect();

                // If we've resolved all fields, break out
                if keys.is_empty() {
                    break;
                }

                for target_field in keys {
                    let resolved_object: Option<NaiveDate> =
                        target_field.resolve(&mut self.field_values)?;

                    if let Some(resolved_object) = resolved_object {
                        self.update_check_conflict_date(Some(resolved_object))?;
                        changed_count += 1;
                        continue 'outer; // Restart iteration with potentially modified map
                    } else if !self.field_values.contains_key(&target_field) {
                        // Field was removed during resolution
                        changed_count += 1;
                        continue 'outer; // Restart iteration with potentially modified map
                    }
                }

                // If we went through all fields and made no changes, we're done
                break;
            }

            if changed_count >= 50 {
                return Err(
                    "One of the parsed fields has an incorrectly implemented resolve method"
                        .to_owned(),
                );
            }

            // if something changed then have to redo ChronoField resolve
            if changed_count > 0 {
                self.resolve_date_fields()?;
                self.resolve_time_fields()?;
            }
        }

        Ok(())
    }

    fn resolve_date_fields(&mut self) -> Result<(), String> {
        let resolved_date = Chronology::resolve_date(&mut self.field_values)?;
        self.update_check_conflict_date(resolved_date)
    }

    fn resolve_time_fields(&mut self) -> Result<(), String> {
        // simplify fields
        if self
            .field_values
            .contains_key(&TemporalField::ClockHourOfDay)
        {
            // smart allows 0-24
            let ch = self
                .field_values
                .remove(&TemporalField::ClockHourOfDay)
                .ok_or("Expected ClockHourOfDay to be present")?;
            if ch != 0 {
                TemporalField::ClockHourOfDay
                    .range()
                    .check_valid_value(ch)?;
            }
            self.update_check_conflict_field(
                TemporalField::ClockHourOfDay,
                TemporalField::HourOfDay,
                if ch == 24 { 0 } else { ch },
            )?;
        }
        if self
            .field_values
            .contains_key(&TemporalField::ClockHourOfAmPm)
        {
            // smart allows 0-12
            let ch = self
                .field_values
                .remove(&TemporalField::ClockHourOfAmPm)
                .ok_or("Expected ClockHourOfAmPm to be present")?;
            if ch != 0 {
                TemporalField::ClockHourOfAmPm
                    .range()
                    .check_valid_value(ch)?;
            }
            self.update_check_conflict_field(
                TemporalField::ClockHourOfAmPm,
                TemporalField::HourOfAmPm,
                if ch == 12 { 0 } else { ch },
            )?;
        }
        if self.field_values.contains_key(&TemporalField::AmPmOfDay)
            && self.field_values.contains_key(&TemporalField::HourOfAmPm)
        {
            let ap = self
                .field_values
                .remove(&TemporalField::AmPmOfDay)
                .ok_or("Expected AmPmOfDay to be present")?;
            let hap = self
                .field_values
                .remove(&TemporalField::HourOfAmPm)
                .ok_or("Expected HourOfAmPm to be present")?;
            TemporalField::AmPmOfDay.range().check_valid_value(ap)?;
            TemporalField::HourOfAmPm.range().check_valid_value(hap)?;
            self.update_check_conflict_field(
                TemporalField::AmPmOfDay,
                TemporalField::HourOfDay,
                ap * 12 + hap,
            )?;
        }
        if self.field_values.contains_key(&TemporalField::NanoOfDay) {
            let nod = self
                .field_values
                .remove(&TemporalField::NanoOfDay)
                .ok_or("Expected NanoOfDay to be present")?;
            TemporalField::NanoOfDay.range().check_valid_value(nod)?;
            self.update_check_conflict_field(
                TemporalField::NanoOfDay,
                TemporalField::HourOfDay,
                nod / 3_600_000_000_000,
            )?;
            self.update_check_conflict_field(
                TemporalField::NanoOfDay,
                TemporalField::MinuteOfHour,
                (nod / 60_000_000_000) % 60,
            )?;
            self.update_check_conflict_field(
                TemporalField::NanoOfDay,
                TemporalField::SecondOfMinute,
                (nod / 1_000_000_000) % 60,
            )?;
            self.update_check_conflict_field(
                TemporalField::NanoOfDay,
                TemporalField::NanoOfSecond,
                nod % 1_000_000_000,
            )?;
        }
        if self.field_values.contains_key(&TemporalField::MicroOfDay) {
            let cod = self
                .field_values
                .remove(&TemporalField::MicroOfDay)
                .ok_or("Expected MicroOfDay to be present")?;
            TemporalField::MicroOfDay.range().check_valid_value(cod)?;
            self.update_check_conflict_field(
                TemporalField::MicroOfDay,
                TemporalField::SecondOfDay,
                cod / 1_000_000,
            )?;
            self.update_check_conflict_field(
                TemporalField::MicroOfDay,
                TemporalField::MicroOfSecond,
                cod % 1_000_000,
            )?;
        }
        if self.field_values.contains_key(&TemporalField::MilliOfDay) {
            let lod = self
                .field_values
                .remove(&TemporalField::MilliOfDay)
                .ok_or("Expected MilliOfDay to be present")?;
            TemporalField::MilliOfDay.range().check_valid_value(lod)?;
            self.update_check_conflict_field(
                TemporalField::MilliOfDay,
                TemporalField::SecondOfDay,
                lod / 1_000,
            )?;
            self.update_check_conflict_field(
                TemporalField::MilliOfDay,
                TemporalField::MilliOfSecond,
                lod % 1_000,
            )?;
        }
        if self.field_values.contains_key(&TemporalField::SecondOfDay) {
            let sod = self
                .field_values
                .remove(&TemporalField::SecondOfDay)
                .ok_or("Expected SecondOfDay to be present")?;
            TemporalField::SecondOfDay.range().check_valid_value(sod)?;
            self.update_check_conflict_field(
                TemporalField::SecondOfDay,
                TemporalField::HourOfDay,
                sod / 3600,
            )?;
            self.update_check_conflict_field(
                TemporalField::SecondOfDay,
                TemporalField::MinuteOfHour,
                (sod / 60) % 60,
            )?;
            self.update_check_conflict_field(
                TemporalField::SecondOfDay,
                TemporalField::SecondOfMinute,
                sod % 60,
            )?;
        }
        if self.field_values.contains_key(&TemporalField::MinuteOfDay) {
            let mod_ = self
                .field_values
                .remove(&TemporalField::MinuteOfDay)
                .ok_or("Expected MinuteOfDay to be present")?;
            TemporalField::MinuteOfDay.range().check_valid_value(mod_)?;
            self.update_check_conflict_field(
                TemporalField::MinuteOfDay,
                TemporalField::HourOfDay,
                mod_ / 60,
            )?;
            self.update_check_conflict_field(
                TemporalField::MinuteOfDay,
                TemporalField::MinuteOfHour,
                mod_ % 60,
            )?;
        }

        // combine partial second fields strictly, leaving lenient expansion to later
        if self.field_values.contains_key(&TemporalField::NanoOfSecond) {
            let &nos = self
                .field_values
                .get(&TemporalField::NanoOfSecond)
                .ok_or("Expected NanoOfSecond to be present")?;
            TemporalField::NanoOfSecond.range().check_valid_value(nos)?;
            if self
                .field_values
                .contains_key(&TemporalField::MicroOfSecond)
            {
                let cos = self
                    .field_values
                    .remove(&TemporalField::MicroOfSecond)
                    .ok_or("Expected MicroOfSecond to be present")?;
                TemporalField::MicroOfSecond
                    .range()
                    .check_valid_value(cos)?;
                let nos = cos * 1000 + (nos % 1000);
                self.update_check_conflict_field(
                    TemporalField::MicroOfSecond,
                    TemporalField::NanoOfSecond,
                    nos,
                )?;
            }
            if self
                .field_values
                .contains_key(&TemporalField::MilliOfSecond)
            {
                let los = self
                    .field_values
                    .remove(&TemporalField::MilliOfSecond)
                    .ok_or("Expected MilliOfSecond to be present")?;
                TemporalField::MilliOfSecond
                    .range()
                    .check_valid_value(los)?;
                self.update_check_conflict_field(
                    TemporalField::MilliOfSecond,
                    TemporalField::NanoOfSecond,
                    los * 1_000_000 + (nos % 1_000_000),
                )?;
            }
        }

        // TODO: Deal with day period

        // convert to time if all four fields available (optimization)
        if self.field_values.contains_key(&TemporalField::HourOfDay)
            && self.field_values.contains_key(&TemporalField::MinuteOfHour)
            && self
                .field_values
                .contains_key(&TemporalField::SecondOfMinute)
            && self.field_values.contains_key(&TemporalField::NanoOfSecond)
        {
            let hod = self
                .field_values
                .remove(&TemporalField::HourOfDay)
                .ok_or("Expected HourOfDay to be present")?;
            let moh = self
                .field_values
                .remove(&TemporalField::MinuteOfHour)
                .ok_or("Expected MinuteOfHour to be present")?;
            let som = self
                .field_values
                .remove(&TemporalField::SecondOfMinute)
                .ok_or("Expected SecondOfMinute to be present")?;
            let nos = self
                .field_values
                .remove(&TemporalField::NanoOfSecond)
                .ok_or("Expected NanoOfSecond to be present")?;
            self.resolve_time(hod, moh, som, nos)?;
        }

        Ok(())
    }

    fn resolve_time(&mut self, hod: i64, moh: i64, som: i64, nos: i64) -> Result<(), String> {
        let moh_val = TemporalField::MinuteOfHour
            .range()
            .check_valid_int_value(moh)?;
        let nos_val = TemporalField::NanoOfSecond
            .range()
            .check_valid_int_value(nos)?;
        // handle 24:00 end of day
        if hod == 24 && moh_val == 0 && som == 0 && nos_val == 0 {
            self.update_check_conflict_time(NaiveTime::from_hms_opt(0, 0, 0), Period::of_days(1))?;
        } else {
            let hod_val = TemporalField::HourOfDay
                .range()
                .check_valid_int_value(hod)?;
            let som_val = TemporalField::SecondOfMinute
                .range()
                .check_valid_int_value(som)?;
            self.update_check_conflict_time(
                NaiveTime::from_hms_nano_opt(
                    hod_val as u32,
                    moh_val as u32,
                    som_val as u32,
                    nos_val as u32,
                ),
                Period::zero(),
            )?;
        }
        Ok(())
    }

    fn update_check_conflict_date(&mut self, cld: Option<NaiveDate>) -> Result<(), String> {
        match (self.date, cld) {
            (None, None) => return Ok(()),
            (Some(_), None) => return Ok(()),
            (None, Some(_)) => self.date = cld,
            (Some(date), Some(cld)) => {
                if date != cld {
                    return Err(format!(
                        "Date conflict: existing date {:?} vs new date {:?}",
                        date, cld
                    ));
                }
            }
        }
        Ok(())
    }

    fn update_check_conflict_time(
        &mut self,
        time_to_set: Option<NaiveTime>,
        period_to_set: Period,
    ) -> Result<(), String> {
        if self.time.is_some() {
            if self.time != time_to_set {
                return Err(format!(
                    "Conflict found: Fields resolved to different times: {:?} {:?}",
                    self.time, time_to_set
                ));
            }
            if self.excess_days != Period::zero()
                && period_to_set != Period::zero()
                && self.excess_days != period_to_set
            {
                return Err(format!(
                    "Conflict found: Fields resolve to different excess periods: {:?} {:?}",
                    self.excess_days, period_to_set
                ));
            } else {
                self.excess_days = period_to_set;
            }
        } else {
            self.time = time_to_set;
            self.excess_days = period_to_set;
        }

        Ok(())
    }

    fn update_check_conflict_field(
        &mut self,
        target_field: TemporalField,
        change_field: TemporalField,
        change_value: i64,
    ) -> Result<(), String> {
        let old = self.field_values.insert(change_field, change_value);
        if let Some(old) = old {
            if old != change_value {
                return Err(format!(
                    "Conflict found: {:?} {} differs from {:?} {} while resolving {:?}",
                    change_field, old, change_field, change_value, target_field
                ));
            }
        }
        Ok(())
    }

    fn resolve_time_lenient(&mut self) -> Result<(), String> {
        // leniently create a time from incomplete information
        // done after everything else as it creates information from nothing
        // which would break updateCheckConflict(field)

        if self.time.is_none() {
            if self
                .field_values
                .contains_key(&TemporalField::MilliOfSecond)
            {
                let los = self
                    .field_values
                    .remove(&TemporalField::MilliOfSecond)
                    .ok_or("Expected MilliOfSecond to be present")?;
                if self
                    .field_values
                    .contains_key(&TemporalField::MicroOfSecond)
                {
                    // merge milli-of-second and micro-of-second for better error message
                    let cos = los * 1000
                        + (self
                            .field_values
                            .get(&TemporalField::MicroOfSecond)
                            .ok_or("Expected MicroOfSecond to be present")?
                            % 1000);
                    self.update_check_conflict_field(
                        TemporalField::MilliOfSecond,
                        TemporalField::MicroOfSecond,
                        cos,
                    )?;
                    self.field_values.remove(&TemporalField::MicroOfSecond);
                    self.field_values
                        .insert(TemporalField::NanoOfSecond, cos * 1000);
                } else {
                    // convert milli-of-second to nano-of-second
                    self.field_values
                        .insert(TemporalField::NanoOfSecond, los * 1_000_000);
                }
            } else if self
                .field_values
                .contains_key(&TemporalField::MicroOfSecond)
            {
                let cos = self
                    .field_values
                    .remove(&TemporalField::MicroOfSecond)
                    .ok_or("Expected MicroOfSecond to be present")?;
                self.field_values
                    .insert(TemporalField::NanoOfSecond, cos * 1000);
            }

            // Set the hour-of-day, if not exist and not in STRICT, to the mid point of the day period or am/pm.
            if !self.field_values.contains_key(&TemporalField::HourOfDay)
                && !self.field_values.contains_key(&TemporalField::MinuteOfHour)
                && !self
                    .field_values
                    .contains_key(&TemporalField::SecondOfMinute)
                && !self.field_values.contains_key(&TemporalField::NanoOfSecond)
            {
                // Note: DayPeriod is not yet implemented, so we skip this part
                // if let Some(day_period) = self.day_period {
                //    let midpoint = day_period.mid();
                //    self.resolve_time(midpoint / 60, midpoint % 60, 0, 0)?;
                //    self.day_period = None;
                // } else
                if self.field_values.contains_key(&TemporalField::AmPmOfDay) {
                    let ap = self
                        .field_values
                        .remove(&TemporalField::AmPmOfDay)
                        .ok_or("Expected AmPmOfDay to be present")?;
                    // Smart
                    TemporalField::AmPmOfDay.range().check_valid_int_value(ap)?;
                    self.resolve_time(ap * 12 + 6, 0, 0, 0)?;
                }
            }

            // merge hour/minute/second/nano leniently
            if let Some(&hod) = self.field_values.get(&TemporalField::HourOfDay) {
                let moh = self.field_values.get(&TemporalField::MinuteOfHour).copied();
                let som = self
                    .field_values
                    .get(&TemporalField::SecondOfMinute)
                    .copied();
                let nos = self.field_values.get(&TemporalField::NanoOfSecond).copied();

                // check for invalid combinations that cannot be defaulted
                if (moh.is_none() && (som.is_some() || nos.is_some()))
                    || (moh.is_some() && som.is_none() && nos.is_some())
                {
                    return Ok(());
                }

                // default as necessary and build time
                let moh_val = moh.unwrap_or(0);
                let som_val = som.unwrap_or(0);
                let nos_val = nos.unwrap_or(0);

                // Note: DayPeriod is not yet implemented, so we skip this check
                // if let Some(day_period) = self.day_period {
                //     if self.resolver_style != ResolverStyle::_Lenient {
                //         // Check whether the hod/mohVal is within the day period
                //         if !day_period.includes(hod * 60 + moh_val) {
                //             return Err(format!(
                //                 "Conflict found: Resolved time {:02}:{:02} conflicts with {:?}",
                //                 hod, moh_val, day_period
                //             ));
                //         }
                //     }
                // }

                self.resolve_time(hod, moh_val, som_val, nos_val)?;
                self.field_values.remove(&TemporalField::HourOfDay);
                self.field_values.remove(&TemporalField::MinuteOfHour);
                self.field_values.remove(&TemporalField::SecondOfMinute);
                self.field_values.remove(&TemporalField::NanoOfSecond);
            }
        }

        // validate remaining
        if self.resolver_style != ResolverStyle::_Lenient && !self.field_values.is_empty() {
            for (&field, &value) in self.field_values.iter() {
                if field.is_time_based() {
                    field.range().check_valid_int_value(value)?;
                }
            }
        }

        Ok(())
    }

    fn cross_check(&mut self) -> Result<(), String> {
        // NOTE: This differs slightly from the original which mutates an iterator while iterating it
        // because that pattern is not allowed in Rust due to borrow checker rules.
        let mut to_remove: HashSet<TemporalField> = HashSet::new();
        for (&field, &val2) in self.field_values.iter() {
            if (field.is_date_based() && self.date.is_some())
                || (field.is_time_based() && self.time.is_some())
            {
                if let Some(val1) = self.get_long_using_date_or_time(field) {
                    if val1 != val2 {
                        return Err(format!(
                            "Conflict found: Fields resolved to different values for {:?}: {} vs {}",
                            field, val1, val2
                        ));
                    } else {
                        to_remove.insert(field);
                    }
                }
            }
        }
        for field in to_remove {
            self.field_values.remove(&field);
        }
        Ok(())
    }

    fn resolve_period(&mut self) -> Result<(), String> {
        match (self.date, self.time, self.excess_days == Period::zero()) {
            (Some(date), Some(_), false) => {
                self.date = Some(
                    date + Duration::days(self.excess_days.days as i64),
                    // NOTE: Handle months and years if needed, although currently we don't need them
                    // since we only care about the midnight case.
                    // + Duration::months(months as i64)
                    // + Duration::years(years as i64),
                );
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn resolve_fractional(&mut self) -> Result<(), String> {
        // This method is simpler for us because we don't have MicroOfSecond or MilliOfSecond
        if self.time.is_none()
            && self
                .field_values
                .contains_key(&TemporalField::SecondOfMinute)
            && !self.field_values.contains_key(&TemporalField::NanoOfSecond)
        {
            self.field_values.insert(TemporalField::NanoOfSecond, 0);
        }
        Ok(())
    }

    pub fn to_chrono_naive_date(&self) -> Result<NaiveDate, String> {
        self.date.ok_or_else(|| "Date not set in Parsed".to_owned())
    }

    pub fn to_chrono_naive_time(&self) -> Result<NaiveTime, String> {
        self.time.ok_or_else(|| "Time not set in Parsed".to_owned())
    }

    pub fn to_chrono_naive_datetime(&self) -> Result<NaiveDateTime, String> {
        let date = self.to_chrono_naive_date()?;
        let time = match self.to_chrono_naive_time() {
            Ok(time) => time,
            Err(_) => NaiveTime::from_hms_opt(0, 0, 0)
                .ok_or_else(|| "Failed to create NaiveTime from hms".to_owned())?,
        };
        Ok(NaiveDateTime::new(date, time))
    }

    pub fn to_chrono_datetime(&self) -> Result<DateTime<FixedOffset>, String> {
        let naive_datetime = self.to_chrono_naive_datetime();
        if let Some(&offset_seconds) = self.field_values.get(&TemporalField::OffsetSeconds) {
            if let Some(offset) = FixedOffset::east_opt(offset_seconds as i32) {
                naive_datetime.map(|ndt| ndt.and_local_timezone(offset).unwrap())
            } else {
                Err(format!("Invalid offset seconds: {}", offset_seconds))
            }
        } else {
            Err("Offset seconds not provided".to_owned())
        }
    }

    pub fn to_epoch_microseconds(&self) -> Result<i64, String> {
        if let Ok(dt) = self.to_chrono_datetime() {
            return Ok(dt.timestamp_micros());
        } else if let Ok(ndt) = self.to_chrono_naive_datetime() {
            // If we don't have an offset, assume UTC
            return Ok(ndt.and_utc().timestamp_micros());
        }
        Err("Failed to convert to epoch microseconds".to_owned())
    }

    pub fn of_epoch_microseconds(micros: i64) -> Self {
        let datetime = chrono::DateTime::<Utc>::from_timestamp_micros(micros);
        Self::of_chrono_datetime(datetime.unwrap())
    }

    pub fn is_supported(&self, field: &TemporalField) -> bool {
        self.field_values.contains_key(field)
            || (field.is_date_based() && self.date.is_some())
            || (field.is_time_based() && self.time.is_some())
    }

    pub fn get_long(&self, field: TemporalField) -> Option<i64> {
        // First we check our field_values map
        if let Some(value_from_field_values) = self.field_values.get(&field) {
            return Some(value_from_field_values).copied();
        }

        // Then we check our date and time if they were resolved
        if let Some(val) = self.get_long_using_date_or_time(field) {
            return Some(val);
        }

        // Finally I added these implicit conversions for DayOfWeek and LocaleDayOfWeek
        // I need to investigate how the implicit conversions work in the original code
        // but for now this should make the tests pass.
        if field == TemporalField::DayOfWeek
            && self
                .field_values
                .contains_key(&TemporalField::LocaleDayOfWeek)
        {
            // ldow will have Sunday=1, Saturday=7
            let ldow = self
                .field_values
                .get(&TemporalField::LocaleDayOfWeek)
                .unwrap();
            // dow will have Monday=1, Sunday=7
            let dow = (ldow + 6) % 7 + 1; // Convert Sunday=1 to Sunday=7
            return Some(dow);
        }
        if field == TemporalField::LocaleDayOfWeek
            && self.field_values.contains_key(&TemporalField::DayOfWeek)
        {
            // dow will have Monday=1, Sunday=7
            let dow = self.field_values.get(&TemporalField::DayOfWeek).unwrap();
            // ldow will have Sunday=1, Saturday=7
            let ldow = dow % 7 + 1; // Convert Sunday=7 to Sunday=1
            return Some(ldow);
        }
        None
    }

    fn get_long_using_date_or_time(&self, field: TemporalField) -> Option<i64> {
        match field {
            TemporalField::NanoOfSecond => self.time.map(|t| (t.nanosecond() as i64)),
            TemporalField::SecondOfMinute => self.time.map(|t| (t.second() as i64)),
            TemporalField::MinuteOfHour => self.time.map(|t| (t.minute() as i64)),
            TemporalField::HourOfDay => self.time.map(|t| t.hour() as i64),
            TemporalField::ClockHourOfDay => {
                self.time
                    .map(|t| if t.hour() == 0 { 24 } else { t.hour() as i64 })
            }
            TemporalField::AmPmOfDay => self.time.map(|t| if t.hour() < 12 { 0 } else { 1 }),
            TemporalField::DayOfWeek => self
                .date
                .map(|d| d.weekday().num_days_from_monday() as i64 + 1),
            TemporalField::DayOfMonth => self.date.map(|d| d.day() as i64),
            TemporalField::DayOfYear => self.date.map(|d| d.ordinal() as i64),
            TemporalField::MonthOfYear => self.date.map(|d| d.month() as i64),
            TemporalField::Year => self.date.map(|d| d.year() as i64),
            TemporalField::YearOfEra => self.date.map(|d| {
                if d.year() > 0 {
                    d.year() as i64
                } else {
                    -d.year() as i64 + 1
                }
            }),
            TemporalField::Era => self.date.map(|d| if d.year() > 0 { 1 } else { 0 }),
            TemporalField::AlignedDayOfWeekInMonth => self.date.map(|d| {
                // This is the day of the week in the month, 1-7
                i64::from(match d.day() % 7 {
                    0 => 7,
                    x => x,
                })
            }),
            TemporalField::HourOfAmPm => self.time.map(|t| t.hour() as i64 % 12),
            TemporalField::ClockHourOfAmPm => self.time.map(|t| {
                if t.hour() % 12 == 0 {
                    12
                } else {
                    t.hour() as i64 % 12
                }
            }),
            TemporalField::MilliOfDay => self.time.map(|t| {
                t.hour() as i64 * 3_600_000
                    + t.minute() as i64 * 60_000
                    + t.second() as i64 * 1_000
                    + t.nanosecond() as i64 / 1_000_000
            }),
            TemporalField::MilliOfSecond => None, // Only used for resolving
            TemporalField::MicroOfDay => None,    // Only used for resolving
            TemporalField::MicroOfSecond => None, // Only used for resolving
            TemporalField::SecondOfDay => None,   // Only used for resolving
            TemporalField::MinuteOfDay => None,   // Only used for resolving
            TemporalField::NanoOfDay => self.time.map(|t| {
                t.hour() as i64 * 3_600_000_000_000
                    + t.minute() as i64 * 60_000_000_000
                    + t.second() as i64 * 1_000_000_000
                    + t.nanosecond() as i64
            }),
            TemporalField::OffsetSeconds => {
                if let Ok(datetime) = self.to_chrono_datetime() {
                    Some(datetime.offset().local_minus_utc() as i64)
                } else if self.to_chrono_naive_datetime().is_ok() {
                    // If we don't have an offset, assume UTC
                    Some(0)
                } else {
                    None
                }
            }
            TemporalField::LocaleDayOfWeek => self
                .date
                .map(|d| i64::from(d.weekday().num_days_from_sunday() + 1)),
            TemporalField::WeekBasedYear => self.date.map(|d| {
                let (_, week_overflow) = get_us_locale_week_of_year_and_overflow(d);
                if week_overflow {
                    d.year() as i64 + 1
                } else {
                    d.year() as i64
                }
            }),
            TemporalField::WeekOfWeekBasedYear => self.date.map(|d| {
                let (us_locale_week_of_year, _) = get_us_locale_week_of_year_and_overflow(d);
                us_locale_week_of_year
            }),
            TemporalField::WeekOfMonth => self.date.map(|d| {
                // This is the number of Sundays in the month before this date + 1
                get_week_of_month(&d) as i64
            }),
            TemporalField::QuarterOfYear => self.date.map(|d| ((d.month() - 1) / 3 + 1) as i64),
            TemporalField::ModifiedJulianDay => self.date.map(|d| {
                d.num_days_from_ce() as i64 + JULIAN_DAY_TO_CE_DAYS
                    - JULIAN_DAY_TO_MODIFIED_JULIAN_DAY_OFFSET
            }),
        }
    }

    pub fn of_chrono_datetime<Tz: TimeZone>(datetime: DateTime<Tz>) -> Self {
        let mut parsed = Parsed::new();
        parsed.date = Some(datetime.date_naive());
        parsed.time = Some(datetime.time());
        parsed.field_values.insert(
            TemporalField::OffsetSeconds,
            datetime.offset().fix().local_minus_utc() as i64,
        );
        parsed
    }
}

impl Default for Parsed {
    fn default() -> Self {
        Self::new()
    }
}

// In the US locale, the week of the year is defined as follows:
// - the first week of the year is the one that contains Jan 1st whenever that is
// - the week starts with Sunday, not Monday like in ISO
// This means that the week based year is almost always the same as the regular year, except in the
// last week of the year, where if the date is like December 28th and it's a Monday, then the last day
// of that week is a Saturday which happens in the next year, so the week based year becomes the next year
fn get_us_locale_week_of_year_and_overflow(date: NaiveDate) -> (i64, bool) {
    if does_week_span_into_next_year(date) {
        return (1, true);
    }

    // Get the date components
    let year = date.year();

    // Get January 1st of the year
    let jan1 = chrono::NaiveDate::from_ymd_opt(year, 1, 1).expect("Invalid date");

    // Get day of week for January 1st (0 = Sunday, 6 = Saturday)
    let jan1_wday = jan1.weekday().num_days_from_sunday();

    // Calculate days since start of year (0-indexed)
    let days_since_year_start = date.ordinal0() as i64;

    // Calculate US week number:
    // Add the weekday of Jan 1 to get offset, divide by 7, and add 1 to make 1-indexed
    (((days_since_year_start + jan1_wday as i64) / 7 + 1), false)
}

/// Checks if the current week spans into the next year
fn does_week_span_into_next_year(naive_date: NaiveDate) -> bool {
    // Get the current weekday (0 = Mon, 6 = Sun in chrono)
    let days_from_sunday = naive_date.weekday().num_days_from_sunday();

    // Add days to reach Saturday
    let end_of_week = naive_date + Duration::days((6 - days_from_sunday) as i64);

    // Check if end of week is in a different year
    end_of_week.year() != naive_date.year()
}

fn get_week_of_month(date: &NaiveDate) -> u32 {
    let first_of_month = date.with_day(1).unwrap();
    let first_weekday = first_of_month.weekday().num_days_from_sunday();
    let day_of_month = date.day();

    // (day_of_month + first_weekday - 1) / 7 gives the zero-based week index
    // Add 1 to make it 1-based (week 1, 2, 3, ...)
    ((day_of_month + first_weekday - 1) / 7) + 1
}
