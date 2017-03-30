// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use error::*;
use error::ErrorKind as EK;
use git2::Commit;
use iter::{StripWhiteSpace, StripWhiteSpaceRightIter};
use iter::{WithoutComments, WithoutCommentsIter};
use std::iter::Skip;
use std::str;
use std::vec;

pub mod line;
pub mod trailer;
pub mod quoted;


/// Special iterator extension for messages
///
/// This iterator extension provides some special functionality for issue and
/// commit messages. It is intended for use on iterators over the lines of a
/// message.
///
pub trait LineIteratorExt<S>
    where S: AsRef<str>
{
    type Iter : Iterator<Item = S>;

    /// Check whether the formatting of a message is valid
    ///
    /// This function checks whether a message has a subject line and whether
    /// that subject line is followed by an empty line. The message should
    /// already be stripped of comments and trailing whitespace.
    ///
    fn check_message_format(self) -> Result<()>;

    /// Create a whitespace and comment stripping iterator
    ///
    /// This function creates an iterator suitable for stripping parts of a
    /// message which should not be stored, e.g. commetents and trailing
    /// whitespace.
    ///
    /// Note that the iterator does not (yet) strip blank lines at the beginning
    /// or end of a message.
    ///
    fn stripped(self) -> StripWhiteSpaceRightIter<WithoutCommentsIter<Self::Iter, S>, S>;

    /// Create an iterator for categorizing lines
    ///
    /// The iterator returned by this function will return categorized lines.
    ///
    fn categorized_lines(self) -> line::Lines<Self::Iter, S>;

    /// Create an iterator for extracting trailers
    ///
    /// Ths iterator returned will only yield trailers in the message. Strings
    /// resembling trailers which co-exist with regular text-lines in a block of
    /// non-blank lines will be ignored (e.g. not returned).
    ///
    fn trailers(self) -> trailer::Trailers<Self::Iter, S>;

    /// Accumulate the lines into a single string
    ///
    fn collect_string(self) -> String;
}

impl<L, S> LineIteratorExt<S> for L
    where L: Iterator<Item = S>,
          S: AsRef<str>
{
    type Iter = L;

    fn check_message_format(mut self) -> Result<()> {
        if try!(self.next().ok_or(Error::from_kind(EK::EmptyMessage))).as_ref().is_empty() {
            return Err(Error::from_kind(EK::EmptySubject))
        }

        if !self.next().map(|line| line.as_ref().is_empty()).unwrap_or(true) {
            return Err(Error::from_kind(EK::MalformedMessage));
        }

        Ok(())
    }

    fn stripped(self) -> StripWhiteSpaceRightIter<WithoutCommentsIter<Self::Iter, S>, S> {
        self.without_comments().strip_whitespace_right()
    }

    fn categorized_lines(self) -> line::Lines<Self::Iter, S> {
        line::Lines::from(self)
    }

    fn trailers(self) -> trailer::Trailers<Self::Iter, S> {
        trailer::Trailers::from(self)
    }

    fn collect_string(self) -> String {
        self.fold(String::new(), |mut res, line| {
            res.push_str(line.as_ref());
            res.push('\n');
            res
        })
    }
}


/// Type representing the lines composing the body part of a commit message
///
pub type BodyLines = Skip<vec::IntoIter<String>>;


/// Extension for commit
///
/// This extension gives a more convenient access to message functionality via
/// `git2::Commit`.
///
pub trait CommitExt {
    /// Get the commit message as a sequence of lines
    ///
    /// If the commit has no message, an empty message will be simulated.
    ///
    fn message_lines(&self) -> vec::IntoIter<String>;

    /// Get the commit message's body as a sequence of lines
    ///
    fn body_lines(&self) -> BodyLines;

    /// Get the commit message's body as a sequence of categorized lines
    ///
    fn categorized_body(&self) -> line::Lines<BodyLines, String>;

    /// Get an iterator over all the trailers in the commit message's body
    ///
    fn trailers(&self) -> trailer::Trailers<BodyLines, String>;
}

impl<'c> CommitExt for Commit<'c> {
    fn message_lines(&self) -> vec::IntoIter<String> {
        let lines : Vec<String> = self.message()
                                      .unwrap_or("")
                                      .lines()
                                      .map(String::from)
                                      .collect();
        lines.into_iter()
    }

    fn body_lines(&self) -> BodyLines {
        self.message_lines().skip(2)
    }

    fn categorized_body(&self) -> line::Lines<BodyLines, String> {
        self.body_lines().categorized_lines()
    }

    fn trailers(&self) -> trailer::Trailers<BodyLines, String> {
        self.body_lines().trailers()
    }
}

