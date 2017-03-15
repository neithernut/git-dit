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
use std::iter::Peekable;
use std::str;


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


#[derive(Debug)]
pub struct Lines<'a>(Peekable<str::Lines<'a>>);

impl<'a> Lines<'a> {
    pub fn new(text: &'a str) -> Lines<'a> {
        Lines::from(text.lines())
    }
}

impl<'a> From<str::Lines<'a>> for Lines<'a> {
    fn from(lines: str::Lines<'a>) -> Self {
        Lines(lines.peekable())
    }
}

impl<'a> Iterator for Lines<'a> {
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next().map(|line| Line::from(line)) {
            Some(Line::Trailer(mut trailer)) => {
                // accumulate potential multiline trailer
                // TODO: also respect other whitespace
                while self.0.peek().map_or(false, |l| l.starts_with(" ")) {
                    // we have to consume the line we peeked at
                    trailer.value = trailer.value.append(self.0.next().unwrap());
                }

                Some(Line::Trailer(trailer))
            },
            next => next,
        }
    }
}


