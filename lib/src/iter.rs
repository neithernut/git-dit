// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use git2::{Oid, References, ReferenceNames};

use error::*;
use error::ErrorKind as EK;

pub struct HeadRefsToIssuesIter<'r>(ReferenceNames<'r>);

impl<'r> Iterator for HeadRefsToIssuesIter<'r> {
    type Item = Result<Oid>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|r_name|  {
                r_name
                    .chain_err(|| EK::WrappedGitError)
                    .and_then(|name| if name.ends_with("/head") {
                        name.rsplitn(3, "/")
                            .nth(1)
                            .ok_or_else(|| {
                                Error::from_kind(EK::MalFormedHeadReference(name.to_string()))
                            })
                            .and_then(|hash| {
                                Oid::from_str(hash)
                                    .chain_err(|| EK::OidFormatError(name.to_string()))
                            })
                    } else {
                        Err(Error::from_kind(EK::MalFormedHeadReference(name.to_string())))
                    })
            })
    }
}

impl<'r> From<References<'r>> for HeadRefsToIssuesIter<'r> {
    fn from(r: References<'r>) -> HeadRefsToIssuesIter<'r> {
        HeadRefsToIssuesIter(r.names())
    }
}

impl<'r> From<ReferenceNames<'r>> for HeadRefsToIssuesIter<'r> {
    fn from(r: ReferenceNames<'r>) -> HeadRefsToIssuesIter<'r> {
        HeadRefsToIssuesIter(r)
    }
}

/// A trait to strip whitespace from a thing that consists of several strings, for example the
/// `std::str::Lines` iterator.
pub trait StripWhiteSpace<I, S>
    where I: Iterator<Item = S> + Sized,
          S: AsRef<str>
{
    fn strip_whitespace_left(self)  -> StripWhiteSpaceLeftIter<I, S>;
    fn strip_whitespace_right(self) -> StripWhiteSpaceRightIter<I, S>;
}

/// Implement the StripWhiteSpace extension trait for all things where we can iterate over String
/// objects.
/// This implements StripWhiteSpace<String> for type String automatically, apparently.
impl<I, S> StripWhiteSpace<I, S> for I
    where I: Iterator<Item = S> + Sized,
          S: AsRef<str>
{
    fn strip_whitespace_left(self) -> StripWhiteSpaceLeftIter<I, S> {
        StripWhiteSpaceLeftIter(self)
    }
    fn strip_whitespace_right(self) -> StripWhiteSpaceRightIter<I, S> {
        StripWhiteSpaceRightIter(self)
    }
}

/// A Iterator type which iterates over String objects, used to strip whitespace from an iterator
/// over String.
pub struct StripWhiteSpaceLeftIter<I, S>(I)
    where I: Iterator<Item = S> + Sized,
          S: AsRef<str>;

impl<'a, I, S> Iterator for StripWhiteSpaceLeftIter<I, S>
    where I: Iterator<Item = S> + Sized,
          S: AsRef<str>
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|s| String::from(s.as_ref().trim_left()))
    }
}

/// A Iterator type which iterates over String objects, used to strip whitespace from an iterator
/// over String.
pub struct StripWhiteSpaceRightIter<I, S>(I)
    where I: Iterator<Item = S> + Sized,
          S: AsRef<str>;

impl<'a, I, S> Iterator for StripWhiteSpaceRightIter<I, S>
    where I: Iterator<Item = S> + Sized,
          S: AsRef<str>
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|s| String::from(s.as_ref().trim_right()))
    }
}

/// Extension trait for everything that iterates over Strings, to remove comment lines
/// (Lines starting with "#")
pub trait WithoutComments<I>
    where I: Iterator<Item = String> + Sized
{
    fn without_comments(self) -> WithoutCommentsIter<I>;
}

/// Iterator type to be returned from WithoutComments::without_comments.
pub struct WithoutCommentsIter<I>(I)
    where I: Iterator<Item = String> + Sized;

impl<I> WithoutComments<I> for I
    where I: Iterator<Item = String> + Sized
{
    fn without_comments(self) -> WithoutCommentsIter<I> {
        WithoutCommentsIter(self)
    }
}

impl<I> Iterator for WithoutCommentsIter<I>
    where I: Iterator<Item = String> + Sized
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.0.next() {
            // we do not trim whitespace here, because of code blocks in the message which might
            // have a "#" at the beginning
            if !next.starts_with("#") {
                return Some(next)
            }
        }
        None
    }
}

