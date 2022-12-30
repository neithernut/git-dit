// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

//! Trailer related functionality
//!
//! This module offers types and functionality for handling git-trailers.
//! Trailers are key-value pairs which may be embedded in a message. "git-dit"
//! uses trailers as storage for issue metadata.
//!

pub mod accumulation;
pub mod filter;
pub mod iter;
pub mod spec;

use regex::Regex;
use std::fmt;
use std::result::Result as RResult;
use std::str::FromStr;

use error::*;
use error::Kind as EK;

/// The Key of a Trailer:
///
/// ```ignore
/// Signed-off-by: Hans Wurst <hans@wurstmail.tld>
/// ^^^^^^^^^^^^^
/// # This is the key
/// ```
///
#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct TrailerKey(String);

impl From<String> for TrailerKey {
    fn from(string: String) -> Self {
        TrailerKey(string)
    }
}

impl AsRef<String> for TrailerKey {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl fmt::Display for TrailerKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> RResult<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}


/// The Value of a Trailer:
///
/// ```ignore
/// Signed-off-by: Hans Wurst <hans@wurstmail.tld>
///                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///                # This is the value
/// ```
///
#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub enum TrailerValue {
    Int(i64),
    String(String),

    // Maybe something like Name { name: String, email: String } ?
}

impl TrailerValue {
    /// Parse a `TrailerValue` from a string slice
    ///
    /// This function will try to parse an integer and fall back to a plain
    /// string.
    ///
    pub fn from_slice(slice: &str) -> TrailerValue {
        match i64::from_str(slice) {
            Ok(i) => TrailerValue::Int(i),
            Err(_) => TrailerValue::String(String::from(slice)),
        }
    }

    /// Append a string to an existing trailer value
    ///
    /// This method may be used to construct multi line trailer values.
    /// Note that the result will always be a string value.
    ///
    pub fn append(&mut self, slice: &str) {
        match self {
            &mut TrailerValue::Int(i)    => *self = TrailerValue::String(i.to_string() + slice),
            &mut TrailerValue::String(ref mut s) => s.push_str(slice),
        }
    }
}

impl Default for TrailerValue {
    fn default() -> Self {
        TrailerValue::String(String::new())
    }
}

impl fmt::Display for TrailerValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> RResult<(), fmt::Error> {
        match *self {
            TrailerValue::Int(i)        => write!(f, "{}", i),
            TrailerValue::String(ref s) => write!(f, "{}", s),
        }
    }
}


/// Trailer representation
///
/// A trailer is nothing but the combination of a `TrailerKey` and a
/// `TrailerValue`.
///
#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct Trailer {
    pub key: TrailerKey,
    pub value: TrailerValue,
}

impl Trailer {
    /// Create a trailer from a key and the string representation of its value
    ///
    pub fn new(key: &str, value: &str) -> Trailer {
        Trailer {
            key  : TrailerKey::from(String::from(key)),
            value: TrailerValue::from_slice(value),
        }
    }
}

impl Into<(TrailerKey, TrailerValue)> for Trailer {
    fn into(self) -> (TrailerKey, TrailerValue) {
        (self.key, self.value)
    }
}

impl fmt::Display for Trailer {
    fn fmt(&self, f: &mut fmt::Formatter) -> RResult<(), fmt::Error> {
        write!(f, "{}: {}", self.key, self.value)
    }
}

impl FromStr for Trailer {
    type Err = Error<git2::Error>;

    fn from_str(s: &str) -> RResult<Self, Self::Err> {
        lazy_static! {
            // regex to match the beginning of a trailer
            static ref RE: Regex = Regex::new(r"^([[:alnum:]-]+)[:=](.*)$").unwrap();
        }

        match RE.captures(s).map(|c| (c.get(1), c.get(2))) {
            Some((Some(key), Some(value))) => Ok(Trailer::new(key.as_str(), value.as_str().trim())),
            _ => Err(EK::TrailerFormatError(s.to_owned()).into())
        }
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    // Trailer tests

    #[test]
    fn string_trailer() {
        let (key, value) = Trailer::from_str("foo-bar: test1 test2 test3")
            .expect("Couldn't parse test string")
            .into();
        assert_eq!(key, TrailerKey("foo-bar".to_string()));
        assert_eq!(value, TrailerValue::String("test1 test2 test3".to_string()));
    }

    #[test]
    fn string_numstart_trailer() {
        let (key, value) = Trailer::from_str("foo-bar: 123test")
            .expect("Couldn't parse test string")
            .into();
        assert_eq!(key, TrailerKey("foo-bar".to_string()));
        assert_eq!(value, TrailerValue::String("123test".to_string()));
    }

    #[test]
    fn numeric_trailer() {
        let (key, value) = Trailer::from_str("foo-bar: 123")
            .expect("Couldn't parse test string")
            .into();
        assert_eq!(key, TrailerKey("foo-bar".to_string()));
        assert_eq!(value, TrailerValue::Int(123));
    }

    #[test]
    fn faulty_trailer() {
        assert!(Trailer::from_str("foo-bar 123").is_err());
    }

    #[test]
    fn faulty_trailer_2() {
        assert!(Trailer::from_str("foo-bar").is_err());
    }

    #[test]
    fn faulty_trailer_3() {
        assert!(Trailer::from_str("foo bar: baz").is_err());
    }

    #[test]
    fn empty_trailer() {
        assert!(Trailer::from_str("").is_err());
    }
}
