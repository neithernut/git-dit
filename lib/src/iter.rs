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

use git2::{self, Repository};
use std::collections::HashMap;

use issue;
use repository::RepositoryExt;

use error::*;
use error::ErrorKind as EK;

/// Iterator for transforming the names of head references to issues
///
/// This iterator wrapps a `ReferenceNames` iterator and returns issues
/// associated to the head references returned by the wrapped iterator.
///
pub struct HeadRefsToIssuesIter<'r>
{
    inner: git2::References<'r>,
    repo: &'r Repository
}

impl<'r> HeadRefsToIssuesIter<'r>
{
    pub fn new(repo: &'r Repository, inner: git2::References<'r>) -> Self {
        HeadRefsToIssuesIter { inner: inner, repo: repo }
    }
}

impl<'r> Iterator for HeadRefsToIssuesIter<'r>
{
    type Item = Result<issue::Issue<'r>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|reference| {
                reference
                    .chain_err(|| EK::CannotGetReference)
                    .and_then(|r| self.repo.issue_by_head_ref(&r))
            })
    }
}


/// Messages iter
///
/// Use this iterator if you intend to iterate over messages rather than `Oid`s
/// via a `Revwalk`.
///
pub struct Messages<'r> {
    pub revwalk: git2::Revwalk<'r>,
    repo: &'r Repository,
}

impl<'r> Messages<'r> {
    /// Create a new Messages itrator from a revwalk for a given repo
    ///
    pub fn new<'a>(repo: &'a Repository, revwalk: git2::Revwalk<'a>) -> Messages<'a> {
        Messages { revwalk: revwalk, repo: repo }
    }

    /// Create a new messages iter from an unconfigured revwalk
    ///
    pub fn empty<'a>(repo: &'a Repository) -> Result<Messages<'a>> {
        repo.revwalk()
            .map(|revwalk| Self::new(repo, revwalk))
            .chain_err(|| EK::CannotConstructRevwalk)
    }

    /// Create an IssueMessagesIter from this instance
    ///
    pub fn until_any_initial(self) -> IssueMessagesIter<'r> {
        self.into()
    }

    /// Terminate this iterator at the given issue's initial message
    ///
    /// This method hides the initial message's parents. It is somewhat more
    /// performant than creating an `IssueMessagesIter`. However, the issue has
    /// to be known in advance.
    ///
    pub fn terminate_at_initial(&mut self, issue: &issue::Issue) -> Result<()> {
        for parent in issue.initial_message()?.parent_ids() {
            self.revwalk.hide(parent)?;
        }
        Ok(())
    }
}

impl<'r> Iterator for Messages<'r> {
    type Item = Result<git2::Commit<'r>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.revwalk
            .next()
            .map(|item| item
                .and_then(|id| self.repo.find_commit(id))
                .chain_err(|| EK::CannotGetCommit)
            )
    }
}


/// Iterator iterating over messages of an issue
///
/// This iterator returns the first parent of a commit or message successively
/// until an initial issue message is encountered, inclusively.
///
pub struct IssueMessagesIter<'r>(Messages<'r>);

impl<'r> IssueMessagesIter<'r> {
    /// Fuse the iterator is the id refers to an issue
    ///
    fn fuse_if_initial(&mut self, id: git2::Oid) {
        if self.0.repo.find_issue(id).is_ok() {
            self.0.revwalk.reset();
        }
    }
}

impl<'r> From<Messages<'r>> for IssueMessagesIter<'r> {
    fn from(messages: Messages<'r>) -> Self {
        IssueMessagesIter(messages)
    }
}

impl<'r> Iterator for IssueMessagesIter<'r> {
    type Item = Result<git2::Commit<'r>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|item| {
                if let Ok(ref commit) = item {
                    self.fuse_if_initial(commit.id());
                }
                item
            })
    }
}


/// Iterator over references referring to any of a number of commits
///
/// This iterator wraps a `git2::Revwalk`. It will iterate over the commits
/// provided by the wrapped iterator. If one of those commits is referred to
/// by any of the whatched references, that references will be returned.
///
/// Only "watched" references are returned, e.g. they need to be supplied
/// through the `watch_ref()` function. Each reference will only be returned
/// once.
///
pub struct RefsReferringTo<'r> {
    refs: HashMap<git2::Oid, Vec<git2::Reference<'r>>>,
    inner: git2::Revwalk<'r>,
    current_refs: Vec<git2::Reference<'r>>,
}

impl<'r> RefsReferringTo<'r> {
    /// Create a new iterator iterating over the messages supplied
    ///
    pub fn new(messages: git2::Revwalk<'r>) -> Self
    {
        Self { refs: HashMap::new(), inner: messages, current_refs: Vec::new() }
    }

    /// Start watching a reference
    ///
    /// A watched reference may be returned by the iterator.
    ///
    pub fn watch_ref(&mut self, reference: git2::Reference<'r>) -> Result<()> {
        let id = reference
            .peel(git2::ObjectType::Any)
            .chain_err(|| EK::CannotGetCommitForRev(reference.name().unwrap_or_default().to_string()))?
            .id();
        self.refs.entry(id).or_insert_with(Vec::new).push(reference);
        Ok(())
    }

    /// Start watching a number of references
    ///
    pub fn watch_refs<I>(&mut self, references: I) -> Result<()>
        where I: IntoIterator<Item = git2::Reference<'r>>
    {
        for reference in references.into_iter() {
            self.watch_ref(reference)?;
        }
        Ok(())
    }
}

impl<'r> Iterator for RefsReferringTo<'r> {
    type Item = Result<git2::Reference<'r>>;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            if let Some(reference) = self.current_refs.pop() {
                // get one of the references for the current commit
                return Some(Ok(reference));
            }

            // Refills may be rather expensive. Let's check whether we have any
            // refs left, first.
            if self.refs.is_empty() {
                return None;
            }

            // refill the stash of references for the next commit
            'refill: for item in &mut self.inner {
                match item.chain_err(|| EK::CannotGetCommit) {
                    Ok(id) => if let Some(new_refs) = self.refs.remove(&id) {
                        // NOTE: should new_refs be empty, we just loop once
                        //       more through the 'outer loop
                        self.current_refs = new_refs;
                        continue 'outer;
                    },
                    Err(err) => return Some(Err(err)),
                }
            }

            // We depleted the inner iterator.
            return None;
        }
    }
}


/// Iterator for deleting references
///
/// This iterator wraps an iterator over references. All of the references
/// returned by the wrapped iterator are deleted. The `ReferenceDeletingIter`
/// itself returns (only) the errors encountered. Sucessful deletions are not
/// reported, e.g. no items will be returned.
///
/// Use this iterator if you want to remove references from a repository but
/// also want to delegate the decision what to do if an error is encountered.
///
pub struct ReferenceDeletingIter<'r, I>
    where I: Iterator<Item = git2::Reference<'r>>
{
    inner: I
}

impl<'r, I> ReferenceDeletingIter<'r, I>
    where I: Iterator<Item = git2::Reference<'r>>
{
    /// Delete, ignoring errors
    ///
    /// Delete all references returned by the wrapped iterator, ignoring all
    /// errors.
    ///
    pub fn delete_ignoring(self) {
        for _ in self {}
    }
}

impl<'r, I, J> From<J> for ReferenceDeletingIter<'r, I>
    where I: Iterator<Item = git2::Reference<'r>>,
          J: IntoIterator<Item = git2::Reference<'r>, IntoIter = I>
{
    fn from(items: J) -> Self {
        ReferenceDeletingIter { inner: items.into_iter() }
    }
}

impl<'r, I> Iterator for ReferenceDeletingIter<'r, I>
    where I: Iterator<Item = git2::Reference<'r>>
{
    type Item = Error;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .by_ref()
            .filter_map(|mut r| r
                .delete()
                .chain_err(|| EK::CannotDeleteReference(r.name().unwrap_or_default().to_string()))
                .err()
            )
            .next()
    }
}

