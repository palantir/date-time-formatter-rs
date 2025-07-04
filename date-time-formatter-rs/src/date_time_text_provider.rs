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

use crate::{temporal_field::TemporalField, text_style::TextStyle};

#[derive(Debug)]
pub struct DateTimeTextProvider {}
impl DateTimeTextProvider {
    pub fn new() -> Self {
        DateTimeTextProvider {}
    }

    pub fn get_text(
        &self,
        field: TemporalField,
        value: i64,
        style: TextStyle,
    ) -> Option<&'static str> {
        self.create_store(field)
            .and_then(|store| store.get_text(value, style))
    }

    pub fn get_text_iterator(
        &self,
        field: TemporalField,
        style: Option<TextStyle>,
    ) -> Option<Vec<(&'static str, i64)>> {
        self.create_store(field)
            .and_then(|store| store.get_text_iterator(style))
    }

    fn create_store(&self, field: TemporalField) -> Option<Store> {
        // NOTE: I got these values by setting the locale to US and just running Java code
        // The 1s and 0s are the defaults for when standalone isn't set I would guess
        match field {
            TemporalField::Era => Some(Store::new(HashMap::from([
                (
                    TextStyle::Full,
                    HashMap::from([(0, "Before Christ"), (1, "Anno Domini")]),
                ),
                (TextStyle::Short, HashMap::from([(0, "BC"), (1, "AD")])),
                (TextStyle::Narrow, HashMap::from([(0, "B"), (1, "A")])),
            ]))),
            TemporalField::MonthOfYear => Some(Store::new(HashMap::from([
                (
                    TextStyle::Full,
                    HashMap::from([
                        (1, "January"),
                        (2, "February"),
                        (3, "March"),
                        (4, "April"),
                        (5, "May"),
                        (6, "June"),
                        (7, "July"),
                        (8, "August"),
                        (9, "September"),
                        (10, "October"),
                        (11, "November"),
                        (12, "December"),
                    ]),
                ),
                (
                    TextStyle::FullStandalone,
                    HashMap::from([
                        (1, "January"),
                        (2, "February"),
                        (3, "March"),
                        (4, "April"),
                        (5, "May"),
                        (6, "June"),
                        (7, "July"),
                        (8, "August"),
                        (9, "September"),
                        (10, "October"),
                        (11, "November"),
                        (12, "December"),
                    ]),
                ),
                (
                    TextStyle::Short,
                    HashMap::from([
                        (1, "Jan"),
                        (2, "Feb"),
                        (3, "Mar"),
                        (4, "Apr"),
                        (5, "May"),
                        (6, "Jun"),
                        (7, "Jul"),
                        (8, "Aug"),
                        (9, "Sep"),
                        (10, "Oct"),
                        (11, "Nov"),
                        (12, "Dec"),
                    ]),
                ),
                (
                    TextStyle::ShortStandalone,
                    HashMap::from([
                        (1, "Jan"),
                        (2, "Feb"),
                        (3, "Mar"),
                        (4, "Apr"),
                        (5, "May"),
                        (6, "Jun"),
                        (7, "Jul"),
                        (8, "Aug"),
                        (9, "Sep"),
                        (10, "Oct"),
                        (11, "Nov"),
                        (12, "Dec"),
                    ]),
                ),
                (
                    TextStyle::Narrow,
                    HashMap::from([
                        (1, "J"),
                        (2, "F"),
                        (3, "M"),
                        (4, "A"),
                        (5, "M"),
                        (6, "J"),
                        (7, "J"),
                        (8, "A"),
                        (9, "S"),
                        (10, "O"),
                        (11, "N"),
                        (12, "D"),
                    ]),
                ),
                (
                    TextStyle::NarrowStandalone,
                    HashMap::from([
                        (1, "J"),
                        (2, "F"),
                        (3, "M"),
                        (4, "A"),
                        (5, "M"),
                        (6, "J"),
                        (7, "J"),
                        (8, "A"),
                        (9, "S"),
                        (10, "O"),
                        (11, "N"),
                        (12, "D"),
                    ]),
                ),
            ]))),
            TemporalField::DayOfWeek => Some(Store::new(HashMap::from([
                (
                    TextStyle::Full,
                    HashMap::from([
                        (1, "Monday"),
                        (2, "Tuesday"),
                        (3, "Wednesday"),
                        (4, "Thursday"),
                        (5, "Friday"),
                        (6, "Saturday"),
                        (7, "Sunday"),
                    ]),
                ),
                (
                    TextStyle::FullStandalone,
                    HashMap::from([
                        (1, "Monday"),
                        (2, "Tuesday"),
                        (3, "Wednesday"),
                        (4, "Thursday"),
                        (5, "Friday"),
                        (6, "Saturday"),
                        (7, "Sunday"),
                    ]),
                ),
                (
                    TextStyle::Short,
                    HashMap::from([
                        (1, "Mon"),
                        (2, "Tue"),
                        (3, "Wed"),
                        (4, "Thu"),
                        (5, "Fri"),
                        (6, "Sat"),
                        (7, "Sun"),
                    ]),
                ),
                (
                    TextStyle::ShortStandalone,
                    HashMap::from([
                        (1, "Mon"),
                        (2, "Tue"),
                        (3, "Wed"),
                        (4, "Thu"),
                        (5, "Fri"),
                        (6, "Sat"),
                        (7, "Sun"),
                    ]),
                ),
                (
                    TextStyle::Narrow,
                    HashMap::from([
                        (1, "M"),
                        (2, "T"),
                        (3, "W"),
                        (4, "T"),
                        (5, "F"),
                        (6, "S"),
                        (7, "S"),
                    ]),
                ),
                (
                    TextStyle::NarrowStandalone,
                    HashMap::from([
                        (1, "M"),
                        (2, "T"),
                        (3, "W"),
                        (4, "T"),
                        (5, "F"),
                        (6, "S"),
                        (7, "S"),
                    ]),
                ),
            ]))),
            TemporalField::LocaleDayOfWeek => Some(Store::new(HashMap::from([
                (
                    TextStyle::Full,
                    HashMap::from([
                        (1, "Sunday"),
                        (2, "Monday"),
                        (3, "Tuesday"),
                        (4, "Wednesday"),
                        (5, "Thursday"),
                        (6, "Friday"),
                        (7, "Saturday"),
                    ]),
                ),
                (
                    TextStyle::FullStandalone,
                    HashMap::from([
                        (1, "Sunday"),
                        (2, "Monday"),
                        (3, "Tuesday"),
                        (4, "Wednesday"),
                        (5, "Thursday"),
                        (6, "Friday"),
                        (7, "Saturday"),
                    ]),
                ),
                (
                    TextStyle::Short,
                    HashMap::from([
                        (1, "Sun"),
                        (2, "Mon"),
                        (3, "Tue"),
                        (4, "Wed"),
                        (5, "Thu"),
                        (6, "Fri"),
                        (7, "Sat"),
                    ]),
                ),
                (
                    TextStyle::ShortStandalone,
                    HashMap::from([
                        (1, "Sun"),
                        (2, "Mon"),
                        (3, "Tue"),
                        (4, "Wed"),
                        (5, "Thu"),
                        (6, "Fri"),
                        (7, "Sat"),
                    ]),
                ),
                (
                    TextStyle::Narrow,
                    HashMap::from([
                        (1, "S"),
                        (2, "M"),
                        (3, "T"),
                        (4, "W"),
                        (5, "T"),
                        (6, "F"),
                        (7, "S"),
                    ]),
                ),
                (
                    TextStyle::NarrowStandalone,
                    HashMap::from([
                        (1, "S"),
                        (2, "M"),
                        (3, "T"),
                        (4, "W"),
                        (5, "T"),
                        (6, "F"),
                        (7, "S"),
                    ]),
                ),
            ]))),
            TemporalField::AmPmOfDay => Some(Store::new(HashMap::from([
                (TextStyle::Full, HashMap::from([(0, "AM"), (1, "PM")])),
                (TextStyle::Short, HashMap::from([(0, "AM"), (1, "PM")])),
                (TextStyle::Narrow, HashMap::from([(0, "a"), (1, "b")])),
            ]))),
            TemporalField::QuarterOfYear => Some(Store::new(HashMap::from([
                (
                    TextStyle::Full,
                    HashMap::from([
                        (1, "1st quarter"),
                        (2, "2nd quarter"),
                        (3, "3rd quarter"),
                        (4, "4th quarter"),
                    ]),
                ),
                (
                    TextStyle::FullStandalone,
                    HashMap::from([
                        (1, "1st quarter"),
                        (2, "2nd quarter"),
                        (3, "3rd quarter"),
                        (4, "4th quarter"),
                    ]),
                ),
                (
                    TextStyle::Short,
                    HashMap::from([(1, "Q1"), (2, "Q2"), (3, "Q3"), (4, "Q4")]),
                ),
                (
                    TextStyle::ShortStandalone,
                    HashMap::from([(1, "Q1"), (2, "Q2"), (3, "Q3"), (4, "Q4")]),
                ),
                (
                    TextStyle::Narrow,
                    HashMap::from([(1, "1"), (2, "2"), (3, "3"), (4, "4")]),
                ),
                (
                    TextStyle::NarrowStandalone,
                    HashMap::from([(1, "1"), (2, "2"), (3, "3"), (4, "4")]),
                ),
            ]))),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Store {
    value_text_map: HashMap<TextStyle, HashMap<i64, &'static str>>,
    parsable: HashMap<Option<TextStyle>, Vec<(&'static str, i64)>>,
}
impl Store {
    pub fn new(value_text_map: HashMap<TextStyle, HashMap<i64, &'static str>>) -> Self {
        let mut map: HashMap<Option<TextStyle>, Vec<(&'static str, i64)>> = HashMap::new();
        let mut all_list: Vec<(&'static str, i64)> = Vec::new();
        for (&vtm_key, vtm_value) in value_text_map.iter() {
            let mut reverse: HashMap<&'static str, (&'static str, i64)> = HashMap::new();
            for (entry_key, &entry_value) in vtm_value.iter() {
                reverse
                    .entry(entry_value)
                    .and_modify(|existing| {
                        if entry_key < &existing.1 {
                            *existing = (entry_value, *entry_key);
                        }
                    })
                    .or_insert((entry_value, *entry_key));
            }
            let mut list: Vec<(&'static str, i64)> = reverse.into_values().collect();
            list.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
            all_list.append(&mut list.clone());
            map.insert(Some(vtm_key), list);
        }
        all_list.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        // If style is None we should get all
        map.insert(None, all_list);

        Store {
            value_text_map,
            parsable: map,
        }
    }

    pub fn get_text(&self, value: i64, style: TextStyle) -> Option<&'static str> {
        self.value_text_map
            .get(&style)
            .and_then(|map| map.get(&value).cloned())
    }

    pub fn get_text_iterator(&self, style: Option<TextStyle>) -> Option<Vec<(&'static str, i64)>> {
        self.parsable.get(&style).cloned()
    }
}
