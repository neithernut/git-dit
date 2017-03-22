// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use message::line::{Line, Lines};
use std::collections::VecDeque;
use std::fmt::{self, Display, Formatter};

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

impl Display for TrailerKey {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
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

    pub fn from_slice(slice: &str) -> TrailerValue {
        use std::str::FromStr;

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

impl Display for TrailerValue {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            TrailerValue::Int(i)        => write!(f, "{}", i),
            TrailerValue::String(ref s) => write!(f, "{}", s),
        }
    }
}

/// The combination of a TrailerKey and a TrailerValue
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

impl Display for Trailer {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}: {}", self.key, self.value)
    }
}


/// Helper type for colecting trailers in a linked list
///
enum TrailerCollector<'l> {
    Collecting(&'l mut VecDeque<Trailer>),
    Dumping,
}

impl<'l> TrailerCollector<'l> {
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

            'refill: for line in self.lines.next() {
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

