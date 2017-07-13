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

use regex::Regex;
use std::collections::VecDeque;
use std::fmt;
use std::result::Result as RResult;
use std::str::FromStr;

use error::*;
use error::ErrorKind as EK;
use message::line::{Line, Lines};

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
    pub fn append(self, slice: &str) -> TrailerValue {
        TrailerValue::String(match self {
            TrailerValue::Int(i)    => i.to_string() + slice,
            TrailerValue::String(s) => s + slice,
        })
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
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            // regex to match the beginning of a trailer
            static ref RE: Regex = Regex::new(r"^([[:alnum:]-]+)[:=](.*)$").unwrap();
        }

        match RE.captures(s).map(|c| (c.get(1), c.get(2))) {
            Some((Some(key), Some(value))) => Ok(Trailer::new(key.as_str(), value.as_str().trim())),
            _ => Err(Error::from_kind(EK::TrailerFormatError(s.to_owned())))
        }
    }
}


/// Helper type for colecting trailers in a linked list
///
/// This collector helps parsing blocks of test such that a block contains only
/// lines of text or trailers. In such a situation, we may need to collect
/// trailers as long as the block of text "looks" like a block of trailers but
/// dump them as soon as the block turns out to be a block of text.
///
/// This collector holds this state and offers the functionality for collecting
/// trailers transparently.
///
enum TrailerCollector<'l> {
    Collecting(&'l mut VecDeque<Trailer>),
    Dumping,
}

impl<'l> TrailerCollector<'l> {
    /// Create a new trailer collector collecting into a target
    ///
    pub fn new(target: &'l mut VecDeque<Trailer>) -> Self {
        TrailerCollector::Collecting(target)
    }

    /// Dump the current and all future trailers pushed to this collector
    ///
    pub fn dumping(self) -> Self {
        if let TrailerCollector::Collecting(target) = self {
            target.clear();
        }
        TrailerCollector::Dumping
    }

    /// Push a new trailer
    ///
    /// The trailer pushed will be either collected or dumped, based on the
    /// current state.
    ///
    pub fn push(self, trailer: Trailer) -> Self {
        match self {
            TrailerCollector::Collecting(target) => {
                target.push_back(trailer);
                TrailerCollector::Collecting(target)
            },
            TrailerCollector::Dumping => self,
        }
    }
}


/// Iterator extracting trailers from a sequence of strings representing lines
///
/// This iterator extracts all trailers from a text provided by the wrapped
/// iterator over the text's lines. Blocks of lines which contain regular lines
/// of text are ignored. Only trailers which are part of a pure block of
/// trailers, delimited by blank lines, are returned by the iterator.
///
pub struct Trailers<I, S>
    where I: Iterator<Item = S>,
          S: AsRef<str>
{
    lines: Lines<I, S>,
    buf: VecDeque<Trailer>,
}

impl<I, S> Trailers<I, S>
    where I: Iterator<Item = S>,
          S: AsRef<str>
{
    pub fn only_dit(self) -> DitTrailers<I, S> {
        DitTrailers(self)
    }
}

impl<I, S> From<I> for Trailers<I, S>
    where I: Iterator<Item = S>,
          S: AsRef<str>
{
    fn from(lines: I) -> Self {
        Trailers {
            lines: Lines::from(lines),
            buf: VecDeque::new(),
        }
    }
}

impl<I, S> Iterator for Trailers<I, S>
    where I: Iterator<Item = S>,
          S: AsRef<str>
{
    type Item = Trailer;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            if let Some(trailer) = self.buf.pop_front() {
                return Some(trailer);
            }

            // refill buffer from next block
            let mut collector = TrailerCollector::new(&mut self.buf);
            let mut at_end = true;

            'refill: for line in &mut self.lines {
                at_end = false;
                collector = match line {
                    Line::Text(_) => collector.dumping(), // block of text
                    Line::Trailer(t) => collector.push(t),
                    Line::Blank => continue 'outer, // end of block
                }
            }

            if at_end {
                return None;
            }
        }
    }

}


/// Iterator extracting DIT trailers from an iterator over trailers
///
pub struct DitTrailers<I, S>(Trailers<I, S>)
    where I: Iterator<Item = S>,
          S: AsRef<str>;

impl<I, S> Iterator for DitTrailers<I, S>
    where I: Iterator<Item = S>,
          S: AsRef<str>
{
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

    // Trailers tests

    #[test]
    fn trailers() {
        let mut trailers = Trailers::from(vec![
            "Foo-bar: bar",
            "",
            "Space: the final frontier.",
            "These are the voyages...",
            "",
            "And then he",
            "said: engage!",
            "",
            "",
            "Signed-off-by: Spock",
            "Dit-status: closed",
            "Multi-line-trailer: multi",
            "  line",
            "  content"
        ].into_iter());

        {
            let (key, _) = trailers.next().expect("Failed to parse trailer1").into();
            assert_eq!(key, TrailerKey("Foo-bar".to_string()));
        }

        {
            let (key, _) = trailers.next().expect("Failed to parse trailer2").into();
            assert_eq!(key, TrailerKey("Signed-off-by".to_string()));
        }

        {
            let (key, _) = trailers.next().expect("Failed to parse trailer3").into();
            assert_eq!(key, TrailerKey("Dit-status".to_string()));
        }

        {
            let (key, value) = trailers.next().expect("Failed to parse trailer4").into();
            assert_eq!(key, TrailerKey("Multi-line-trailer".to_string()));
            assert_eq!(value, TrailerValue::String("multi  line  content".to_string()));
        }

        assert!(!trailers.next().is_some())
    }
}
