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
use std::borrow::Borrow;
use std::collections::HashMap;
use std::iter::FromIterator;

use issue;
use repository::RepositoryExt;
use trailer::{accumulation, spec};

use error::*;
use error::Kind as EK;

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
    type Item = Result<issue::Issue<'r>, git2::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|reference| {
                reference
                    .wrap_with_kind(EK::CannotGetReference)
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
    pub(crate) revwalk: git2::Revwalk<'r>,
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
    pub fn empty<'a>(repo: &'a Repository) -> Result<Messages<'a>, git2::Error> {
        repo.revwalk()
            .map(|revwalk| Self::new(repo, revwalk))
            .wrap_with_kind(EK::CannotConstructRevwalk)
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
    pub fn terminate_at_initial(&mut self, issue: &issue::Issue) -> Result<(), git2::Error> {
        for parent in issue.initial_message()?.parent_ids() {
            self.revwalk.hide(parent)?;
        }
        Ok(())
    }
}

impl<'r> Iterator for Messages<'r> {
    type Item = Result<git2::Commit<'r>, git2::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.revwalk
            .next()
            .map(|item| item
                .and_then(|id| self.repo.find_commit(id))
                .wrap_with_kind(EK::CannotGetCommit)
            )
    }
}


/// Messages iterator extension trait
///
/// This trait provides some convenience functionality for iterators over
/// `Message`s which does not need to be part of `Messages` or another iterator.
///
pub trait MessagesExt {
    type Output:
        accumulation::MultiAccumulator +
        FromIterator<(String, accumulation::ValueAccumulator)>;

    /// Accumulate trailers according to the specification provided
    ///
    /// This function accumulates all specified trailers from the messages
    /// returned by the iterator.
    ///
    fn accumulate_trailers<'a, I, J>(self, specs: I) -> Self::Output
        where I: IntoIterator<Item = J>,
              J: Borrow<spec::TrailerSpec<'a>>;
}

impl<'a, I> MessagesExt for I
    where I: Iterator<Item = git2::Commit<'a>>
{
    type Output = HashMap<String, accumulation::ValueAccumulator>;

    fn accumulate_trailers<'b, J, K>(self, specs: J) -> Self::Output
        where J: IntoIterator<Item = K>,
              K: Borrow<spec::TrailerSpec<'b>>
    {
        use message::Message;
        use trailer::accumulation::Accumulator;
        use trailer::spec::ToMap;

        let mut accumulator = specs.into_map();
        accumulator.process_all(self.flat_map(|message| message.trailers()));
        accumulator
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
    type Item = Result<git2::Commit<'r>, git2::Error>;

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

    /// Push a starting point for the iteration
    ///
    /// The message will be pushed onto the underlying `Revwalk` used for
    /// iterating over messages.
    ///
    pub fn push(&mut self, message: git2::Oid) -> Result<(), git2::Error> {
        self.inner.push(message).wrap_with_kind(EK::CannotConstructRevwalk)
    }

    /// Start watching a reference
    ///
    /// A watched reference may be returned by the iterator.
    ///
    pub fn watch_ref(&mut self, reference: git2::Reference<'r>) -> Result<(), git2::Error> {
        let id = reference
            .peel(git2::ObjectType::Any)
            .wrap_with(|| EK::CannotGetCommitForRev(reference.name().unwrap_or_default().to_string()))?
            .id();
        self.refs.entry(id).or_insert_with(Vec::new).push(reference);
        Ok(())
    }

    /// Start watching a number of references
    ///
    pub fn watch_refs<I>(&mut self, references: I) -> Result<(), git2::Error>
        where I: IntoIterator<Item = git2::Reference<'r>>
    {
        for reference in references.into_iter() {
            self.watch_ref(reference)?;
        }
        Ok(())
    }
}

impl<'r> Iterator for RefsReferringTo<'r> {
    type Item = Result<git2::Reference<'r>, git2::Error>;

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
            for item in &mut self.inner {
                match item.wrap_with_kind(EK::CannotGetCommit) {
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


/// Implementation of Extend for RefsReferringTo
///
/// The references supplied will be returned by the extended `RefsReferringTo`
/// iterator.
///
impl<'r> Extend<git2::Reference<'r>> for RefsReferringTo<'r> {
    fn extend<I>(&mut self, references: I)
        where I: IntoIterator<Item = git2::Reference<'r>>
    {
        self.current_refs.extend(references);
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
    type Item = Error<git2::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .by_ref()
            .filter_map(|mut r| r
                .delete()
                .wrap_with(|| EK::CannotDeleteReference(r.name().unwrap_or_default().to_string()))
                .err()
            )
            .next()
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::TestingRepo;

    use repository::RepositoryExt;

    // RefsReferringTo tests

    #[test]
    fn referred_refs() {
        let mut testing_repo = TestingRepo::new("referred_refs");
        let repo = testing_repo.repo();

        let sig = git2::Signature::now("Foo Bar", "foo.bar@example.com")
            .expect("Could not create signature");
        let empty_tree = repo
            .empty_tree()
            .expect("Could not create empty tree");
        let empty_parents: Vec<&git2::Commit> = vec![];

        let mut commits = repo.revwalk().expect("Could not create revwalk");
        let mut refs_to_watch = Vec::new();
        let mut refs_to_report = Vec::new();

        {
            let commit = repo
                .commit(None, &sig, &sig, "Test message 1", &empty_tree, &empty_parents)
                .expect("Could not create commit");
            let refa = repo
                .reference("refs/test/1a", commit, false, "create test ref 1a")
                .expect("Could not create reference");
            let refb = repo
                .reference("refs/test/1b", commit, false, "create test ref 1b")
                .expect("Could not create reference");
            commits.push(commit).expect("Could not push commit onto revwalk");
            refs_to_report.push(refa.name().expect("Could not retrieve name").to_string());
            refs_to_report.push(refb.name().expect("Could not retrieve name").to_string());
            refs_to_watch.push(refa);
            refs_to_watch.push(refb);
        }

        {
            let commit = repo
                .commit(None, &sig, &sig, "Test message 2", &empty_tree, &empty_parents)
                .expect("Could not create commit");
            let refa = repo
                .reference("refs/test/2a", commit, false, "create test ref 2a")
                .expect("Could not create reference");
            repo.reference("refs/test/2b", commit, false, "create test ref 2b")
                .expect("Could not create reference");
            commits.push(commit).expect("Could not push commit onto revwalk");
            refs_to_report.push(refa.name().expect("Could not retrieve name").to_string());
            refs_to_watch.push(refa);
        }

        {
            let commit = repo
                .commit(None, &sig, &sig, "Test message 3", &empty_tree, &empty_parents)
                .expect("Could not create commit");
            repo.reference("refs/test/3a", commit, false, "create test ref 3a")
                .expect("Could not create reference");
            repo.reference("refs/test/3b", commit, false, "create test ref 3b")
                .expect("Could not create reference");
            commits.push(commit).expect("Could not push commit onto revwalk");
        }

        {
            let commit = repo
                .commit(None, &sig, &sig, "Test message 4", &empty_tree, &empty_parents)
                .expect("Could not create commit");
            let refa = repo
                .reference("refs/test/4a", commit, false, "create test ref 4a")
                .expect("Could not create reference");
            let refb = repo
                .reference("refs/test/4b", commit, false, "create test ref 4b")
                .expect("Could not create reference");
            refs_to_watch.push(refa);
            refs_to_watch.push(refb);
        }

        let mut referred = RefsReferringTo::new(commits);
        referred.watch_refs(refs_to_watch).expect("Could not watch refs");

        let mut reported: Vec<_> = referred
            .map(|item| item
                .expect("Error during iterating over refs")
                .name()
                .expect("Could not retrieve name")
                .to_string()
            )
            .collect();
        reported.sort();
        refs_to_report.sort();
        assert_eq!(reported, refs_to_report);
    }
}

