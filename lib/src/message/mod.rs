// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

//! Message handling utilities
//!
//! This module provides utilities for handling and processing individual
//! messages' texts. It may be used in order to prepare a message for
//! committing or analyzing messages which are already committed.
//!
//! The main interface provided by this module are the `LineIteratorExt` trait
//! and the `CommitExt` trait. While the former provides git-dit specific
//! utility operations on lines of text, the latter provides functions for
//! retrieving lines of text as well as other data from commits conveniently.
//!

use error::*;
use error::ErrorKind as EK;
use git2::Commit;
use std;

pub mod block;
pub mod line_processor;
pub mod metadata;

use self::line_processor::{Quoted, StripWhiteSpaceRightIter, WithoutCommentsIter};


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

    /// Create an iterator for quoting lines
    ///
    /// The iterator returned will prepend a `>` and, in the case of non-empty
    /// lines, a space, to each item.
    ///
    fn quoted(self) -> Quoted<Self::Iter, S>;

    /// Create an iterator over categorized blocks
    ///
    /// The iterator returned by this function provides a line-block oriented
    /// view.
    ///
    fn line_blocks(self) -> block::Blocks<Self::Iter, S>;

    /// Create an iterator for extracting trailers
    ///
    /// Ths iterator returned will only yield trailers in the message. Strings
    /// resembling trailers which co-exist with regular text-lines in a block of
    /// non-blank lines will be ignored (e.g. not returned).
    ///
    fn trailers(self) -> block::Trailers<Self::Iter, S>;

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
        StripWhiteSpaceRightIter::from(WithoutCommentsIter::from(self))
    }

    fn quoted(self) -> Quoted<Self::Iter, S> {
        Quoted::from(self)
    }

    fn line_blocks(self) -> block::Blocks<Self::Iter, S> {
        block::Blocks::from(self)
    }

    fn trailers(self) -> block::Trailers<Self::Iter, S> {
        self.into()
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
pub type BodyLines = std::iter::Skip<std::vec::IntoIter<String>>;


/// Message trait
///
/// This extension gives a more convenient access to message functionality via
/// `git2::Commit`.
///
pub trait Message {
    /// Get the commit message as a sequence of lines
    ///
    /// If the commit has no message, an empty message will be simulated.
    ///
    fn message_lines(&self) -> std::vec::IntoIter<String>;

    /// Get the commit message's body as a sequence of lines
    ///
    fn body_lines(&self) -> BodyLines;

    /// Get the commit message's body as a sequence of paragraphs and blocks of trailers
    ///
    fn body_blocks(&self) -> block::Blocks<BodyLines, String>;

    /// Get an iterator over all the trailers in the commit message's body
    ///
    fn trailers(&self) -> block::Trailers<BodyLines, String>;

    /// Get a suitable subject for a reply
    ///
    /// The subject returned will start with "Re: ".
    ///
    fn reply_subject(&mut self) -> Option<String>;
}

impl<'c> Message for Commit<'c> {
    fn message_lines(&self) -> std::vec::IntoIter<String> {
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

    fn body_blocks(&self) -> block::Blocks<BodyLines, String> {
        self.body_lines().line_blocks()
    }

    fn trailers(&self) -> block::Trailers<BodyLines, String> {
        self.body_lines().trailers()
    }

    fn reply_subject(&mut self) -> Option<String> {
        self.summary().map(|s| {
            if s.starts_with("Re: ") {
                s.to_owned()
            } else {
                format!("Re: {}", s)
            }
        })
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    // LineIteratorExt tests

    #[test]
    fn empty_message_format_check() {
        let vec : Vec<&str> = Vec::new();
        assert!(vec.into_iter().check_message_format().is_err());
    }

    #[test]
    fn empty_message_format_check2() {
        assert!(vec![""].into_iter().check_message_format().is_err());
    }

    #[test]
    fn oneline_message_format_check() {
        vec!["Foo bar"].into_iter().check_message_format().unwrap();
    }

    #[test]
    fn malformed_message_format_check() {
        assert!(vec!["Foo bar", "Baz"].into_iter().check_message_format().is_err());
    }

    #[test]
    fn multiline_message_format_check() {
        vec!["Foo bar", "", "Baz"].into_iter().check_message_format().unwrap();
    }
}
