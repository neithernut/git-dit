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
use iter::{StripWhiteSpace, StripWhiteSpaceRightIter};
use iter::{WithoutComments, WithoutCommentsIter};

pub mod line;
pub mod trailer;


/// Special iterator extension for messages
///
/// This iterator extension provides some special functionality for issue and
/// commit messages. It is intended for use on iterators over the lines of a
/// message.
///
pub trait LineIteratorExt {
    type Iter : Iterator<Item = String>;

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
    fn stripped(self) -> StripWhiteSpaceRightIter<WithoutCommentsIter<Self::Iter>>;

    /// Create an iterator for categorizing lines
    ///
    /// The iterator returned by this function will return categorized lines.
    ///
    fn categorized_lines(self) -> line::Lines<Self::Iter, String>;

    /// Create an iterator for extracting trailers
    ///
    /// Ths iterator returned will only yield trailers in the message. Strings
    /// resembling trailers which co-exist with regular text-lines in a block of
    /// non-blank lines will be ignored (e.g. not returned).
    ///
    fn trailers(self) -> trailer::Trailers<Self::Iter, String>;
}

impl<L> LineIteratorExt for L
    where L: Iterator<Item = String>
{
    type Iter = L;

    fn check_message_format(mut self) -> Result<()> {
        if try!(self.next().ok_or(Error::from_kind(EK::EmptyMessage))).is_empty() {
            return Err(Error::from_kind(EK::EmptySubject))
        }

        if !self.next().map(|line| line.is_empty()).unwrap_or(true) {
            return Err(Error::from_kind(EK::MalformedMessage));
        }

        Ok(())
    }

    fn stripped(self) -> StripWhiteSpaceRightIter<WithoutCommentsIter<Self::Iter>> {
        self.without_comments().strip_whitespace_right()
    }

    fn categorized_lines(self) -> line::Lines<Self::Iter, String> {
        line::Lines::from(self)
    }

    fn trailers(self) -> trailer::Trailers<Self::Iter, String> {
        trailer::Trailers::from(self)
    }
}

