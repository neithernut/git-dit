// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

//! Line categorization
//!
//! When processing messages, we may have to differentiate between three types
//! of lines: regular lines of text, trailers and blank lines, which delimit
//! blocks of either text or trailers.
//!
//! This module provides a type for representing the different types of lines as
//! well as an convenience iterator which may be used for categorizing lines
//! provided by an iterator.
//!

use message::trailer::Trailer;
use std::iter::Peekable;
use std::str::FromStr;


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

impl<S: AsRef<str>> From<S> for Line {
    fn from(line: S) -> Self {
        let trimmed = line.as_ref().trim_right();
        if trimmed.is_empty() {
            return Line::Blank;
        }

        match Trailer::from_str(trimmed) {
            Ok(trailer) => Line::Trailer(trailer),
            _ => Line::Text(String::from(trimmed)),
        }
    }
}


/// Categorized lines from a sequence of strings
///
/// This iterator wraps an iterator over string-like items. It translates the
/// strings pulled from the wrapped iterator into categorized lines. Multiline
/// trailers will be converted in single `Line::Trailer` items.
///
#[derive(Debug)]
pub struct Lines<I, S>(Peekable<I>)
    where I: Iterator<Item = S>,
          S: AsRef<str>;

impl<I, S> From<I> for Lines<I, S>
    where I: Iterator<Item = S>,
          S: AsRef<str>
{
    fn from(lines: I) -> Self {
        Lines(lines.peekable())
    }
}

impl<I, S> Iterator for Lines<I, S>
    where I: Iterator<Item = S>,
          S: AsRef<str>
{
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next().as_ref().map(Line::from) {
            Some(Line::Trailer(mut trailer)) => {
                // accumulate potential multiline trailer
                // TODO: also respect other whitespace
                while self.0.peek().map_or(false, |l| l.as_ref().starts_with(" ")) {
                    // we have to consume the line we peeked at
                    trailer.value.append(self.0.next().unwrap().as_ref());
                }

                Some(Line::Trailer(trailer))
            },
            next => next,
        }
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    // Line tests

    #[test]
    fn text_line() {
        match Line::from("Just a line of text") {
            Line::Text(_) => (),
            _ => panic!("Line of text misinterpreted"),
        }
    }

    #[test]
    fn trailer_line() {
        match Line::from("Foo-bar: baz") {
            Line::Trailer(_) => (),
            _ => panic!("Trailer misinterpreted"),
        }
    }

    #[test]
    fn blank_line() {
        match Line::from("") {
            Line::Blank => (),
            _ => panic!("Blank line misinterpreted"),
        }
    }

    // Lines tests

    #[test]
    fn lines_test() {
        let mut lines = Lines::from(vec![
            "Mary had a little lamb",
            "Foo-bar: baz",
            "More text",
            "",
            "Multi: line",
            "  trailer"
        ].into_iter());

        match lines.next() {
            Some(Line::Text(_)) => (),
            Some(_) => panic!("Expected line of text"),
            None => panic!("premature end of input"),
        }

        match lines.next() {
            Some(Line::Trailer(_)) => (),
            Some(_) => panic!("Expected trailer"),
            None => panic!("premature end of input"),
        }

        match lines.next() {
            Some(Line::Text(_)) => (),
            Some(_) => panic!("Expected line of text"),
            None => panic!("premature end of input"),
        }

        match lines.next() {
            Some(Line::Blank) => (),
            Some(_) => panic!("Expected blank line"),
            None => panic!("premature end of input"),
        }

        match lines.next() {
            Some(Line::Trailer(_)) => (),
            Some(_) => panic!("Expected trailer"),
            None => panic!("premature end of input"),
        }

        match lines.next() {
            None => (),
            Some(_) => panic!("Expected end of input")
        }
    }
}
