// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use regex::Regex;
use std::str::Lines;

#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct TrailerKey(String);

impl From<String> for TrailerKey {
    fn from(string: String) -> Self {
        TrailerKey(string)
    }
}


#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub enum TrailerValue {
    Int(i64),
    String(String),

    // Maybe something like Name { name: String, email: String } ?
}

impl TrailerValue {

    pub fn from_slice(slice: &str) -> TrailerValue {
        use std::str::FromStr;

        match i64::from_str(slice) {
            Ok(i) => TrailerValue::Int(i),
            Err(_) => TrailerValue::String(String::from(slice)),
        }
    }

}

#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct Trailer {
    pub key: TrailerKey,
    pub value: TrailerValue,
}

impl Into<(TrailerKey, TrailerValue)> for Trailer {
    fn into(self) -> (TrailerKey, TrailerValue) {
        (self.key, self.value)
    }
}

pub struct Trailers<'a>(Lines<'a>);

impl<'a> Trailers<'a> {

    /// Create a new Trailers iterator from a commit message
    pub fn new(text: &'a str) -> Trailers<'a> {
        Trailers(text.lines())
    }

    pub fn only_dit(self) -> DitTrailers<'a> {
        DitTrailers(self)
    }

}

impl<'a> Iterator for Trailers<'a> {
    type Item = Trailer;

    fn next(&mut self) -> Option<Self::Item> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(?P<key>(.*)):\ (?P<value>(.*))$").unwrap();
        }

        loop {
            match self.0.next() {
                None => return None,
                Some(line) => {
                    match RE.captures(line) {
                        None => continue,
                        Some(capture) => {
                            let key = capture.name("key").map(|m| {
                                TrailerKey(String::from(m.as_str()))
                            }).unwrap(); // TODO: fix unwrap()

                            let value = capture.name("value").map(|m| {
                                TrailerValue::from_slice(m.as_str())
                            }).unwrap(); // TODO: fix unwrap()

                            return Some(Trailer {
                                key  : key,
                                value: value,
                            });
                        }
                    }
                }
            }
        }
    }

}

pub struct DitTrailers<'a>(Trailers<'a>);

impl<'a> Iterator for DitTrailers<'a> {
    type Item = Trailer;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next() {
                None => return None,
                Some(trailer) => {
                    if trailer.key.0.starts_with("Dit") {
                        return Some(trailer);
                    } else {
                        continue;
                    }

                }
            }
        }
    }

}

