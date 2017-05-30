// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use first_parent_iter::FirstParentIter;
use git2::{Commit, Oid, Repository, References, ReferenceNames};
use repository::RepositoryExt;

use error::*;
use error::ErrorKind as EK;

/// Iterator for transforming the names of head references to issues
///
/// This iterator wrapps a `ReferenceNames` iterator and returns issues
/// associated to the head references returned by the wrapped iterator.
///
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


/// Iterator iterating over messages of an issue
///
/// This iterator returns the first parent of a commit or message successively
/// until an initial issue message is encountered, inclusively.
///
pub struct IssueMessagesIter<'r> {
    inner: FirstParentIter<'r>,
    repo: &'r Repository,
}

impl<'r> IssueMessagesIter<'r> {
    pub fn new<'a>(commit: Commit<'a>, repo: &'a Repository) -> IssueMessagesIter<'a> {
        IssueMessagesIter {
            inner: FirstParentIter::new(commit),
            repo: repo,
        }
    }
}

impl<'r> Iterator for IssueMessagesIter<'r> {
    type Item = <FirstParentIter<'r> as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next();

        // if this was the initial message, we fuse the underlying iterator
        if next.as_ref()
               .map(Commit::id)
               .map(|id| self.repo.get_issue_heads(id))
               .and_then(Result::ok)
               .map(|refs| refs.count() > 0)
               .unwrap_or(false) {
            self.inner.fuse_now();
        }

        next
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
pub trait WithoutComments<I, S>
    where I: Iterator<Item = S> + Sized,
          S: AsRef<str>
{
    fn without_comments(self) -> WithoutCommentsIter<I, S>;
}

/// Iterator type to be returned from WithoutComments::without_comments.
pub struct WithoutCommentsIter<I, S>(I)
    where I: Iterator<Item = S> + Sized,
          S: AsRef<str>;

impl<I, S> WithoutComments<I, S> for I
    where I: Iterator<Item = S> + Sized,
          S: AsRef<str>
{
    fn without_comments(self) -> WithoutCommentsIter<I, S> {
        WithoutCommentsIter(self)
    }
}

impl<I, S> Iterator for WithoutCommentsIter<I, S>
    where I: Iterator<Item = S> + Sized,
          S: AsRef<str>
{
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.0.next() {
            // we do not trim whitespace here, because of code blocks in the message which might
            // have a "#" at the beginning
            if !next.as_ref().starts_with("#") {
                return Some(next)
            }
        }
        None
    }
}

