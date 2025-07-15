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

use std::cmp::min;

use chrono::{Datelike, NaiveDate};

pub fn floor_mod(x: i32, y: i32) -> i32 {
    let mut m = x % y;
    if (m ^ y) < 0 && m != 0 {
        m += y;
    }
    m
}

pub fn localized_day_of_week(temporal: NaiveDate) -> i32 {
    // Assuming US locale for simplicity, where the week starts on Sunday
    let sow = get_first_day_of_week();
    let iso_dow = temporal.weekday().num_days_from_monday() + 1;
    floor_mod((iso_dow as i32) - sow, 7) + 1
}

pub fn localized_day_of_week_of_iso_dow(iso_dow: i32) -> i32 {
    // Assuming US locale for simplicity, where 1 is Sunday and 7 is Saturday
    let sow = get_first_day_of_week();
    floor_mod(iso_dow - sow, 7) + 1
}

pub fn localized_week_of_month(temporal: NaiveDate) -> i32 {
    let dow = localized_day_of_week(temporal);
    let dom = temporal.day();
    let offset = start_of_week_offset(dom, dow);
    compute_week(offset, dom)
}

pub fn localized_week_of_year(temporal: NaiveDate) -> i32 {
    let dow = localized_day_of_week(temporal);
    let doy = temporal.ordinal();
    let offset = start_of_week_offset(doy, dow);
    compute_week(offset, doy)
}

pub fn of_week_based_year(yowby: i32, wowby: i32, dow: i32) -> Result<NaiveDate, String> {
    let date = NaiveDate::from_ymd_opt(yowby, 1, 1)
        .ok_or("Could not create date using week based year")?;
    let ldow = localized_day_of_week(date);
    let offset = start_of_week_offset(1, ldow);

    // Clamp the week of year to keep it in the same year
    let year_len = get_days_in_year(&date);
    let new_year_week = compute_week(offset, (year_len + get_minimal_days_in_first_week()) as u32);
    let wowby = min(wowby, new_year_week - 1);

    let days = -offset + (dow - 1) + (wowby - 1) * 7;

    Ok(date + chrono::Duration::days(days as i64))
}

pub fn start_of_week_offset(day: u32, dow: i32) -> i32 {
    let week_start = floor_mod((day as i32) - dow, 7);
    let mut offset = -week_start;
    if week_start + 1 > get_minimal_days_in_first_week() {
        offset = 7 - week_start;
    }
    offset
}

pub fn get_minimal_days_in_first_week() -> i32 {
    // In US locale, the minimal days in the first week is 1
    1
}

pub fn get_first_day_of_week() -> i32 {
    // In US locale, the first day of the week is Sunday
    7
}

pub fn compute_week(offset: i32, day: u32) -> i32 {
    (7 + offset + ((day as i32) - 1)) / 7
}

pub fn get_days_in_year(date: &NaiveDate) -> i32 {
    if date.leap_year() {
        366
    } else {
        365
    }
}
