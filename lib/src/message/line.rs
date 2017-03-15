// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use message::trailer::{Trailer, TrailerKey, TrailerValue};
use regex::Regex;


/// A line of an issue message
///
/// We differentiate between different type of lines. Trailers are special in
/// this context, since they may span multiple lines but are represented as a
/// single `Line`.
///
pub enum Line {
    Text(String),
    Trailer(Trailer),
    Blank
}

impl<'a> From<&'a str> for Line {
    fn from(line :&'a str) -> Self {
        lazy_static! {
            // regex to match the beginning of a trailer
            static ref RE: Regex = Regex::new(r"^(?P<key>([^[:space:]]+)):\ (?P<value>(.*))$").unwrap();
        }

        let trimmed = line.trim_right();
        if trimmed.is_empty() {
            return Line::Blank;
        }

        match RE.captures(trimmed).map(|c| (c.name("key"), c.name("value"))) {
            Some((Some(key), Some(value))) => Line::Trailer(Trailer {
                key  : TrailerKey::from(String::from(key.as_str())),
                value: TrailerValue::from_slice(value.as_str()),
            }),
            _ => Line::Text(String::from(trimmed)),
        }
    }
}
