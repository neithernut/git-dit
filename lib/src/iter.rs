// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

//! Utility iterators
//!
//! This module provides various iterators.
//!

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
                    .chain_err(|| EK::ReferenceNameError)
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

