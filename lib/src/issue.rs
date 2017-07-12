// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

//! Issues
//!
//! This module provides the `Issue` type and related functionality.
//!

use git2::{self, Commit, Oid, Reference, References};
use std::fmt;
use std::result::Result as RResult;

use error::*;
use error::ErrorKind as EK;


pub enum IssueRefType {
    Any,
    Head,
    Leaf,
}

impl IssueRefType {
    /// Get the part of a glob specific to the type
    ///
    pub fn glob_part(&self) -> &'static str {
        match *self {
            IssueRefType::Any   => "**",
            IssueRefType::Head  => "head",
            IssueRefType::Leaf  => "leaves/*",
        }
    }
}


/// Issue handle
///
/// Instances of this type represent single issues. Issues reside in
/// repositories and are uniquely identified by an id.
///
pub struct Issue<'r> {
    repo: &'r git2::Repository,
    id: Oid,
}

impl<'r> Issue<'r> {
    /// Create a new handle for an issue with a given id
    ///
    pub fn new(repo: &'r git2::Repository, id: Oid) -> Self {
        Issue { repo: repo, id: id }
    }

    /// Get the issue's id
    ///
    pub fn id(&self) -> Oid {
        self.id
    }

    /// Get the issue's initial message
    ///
    pub fn initial_message(&self) -> Result<git2::Commit> {
        self.repo.find_commit(self.id).chain_err(|| EK::CannotGetCommit)
    }

    /// Get possible heads of the issue
    ///
    /// Returns the head references from both the local repository and remotes
    /// for this issue.
    ///
    pub fn heads(&self) -> Result<References<'r>> {
        let glob = format!("**/dit/{}/head", self.ref_part());
        self.repo
            .references_glob(&glob)
            .chain_err(|| EK::CannotFindIssueHead(self.id))
    }

    /// Get the local issue head for the issue
    ///
    /// Returns the head reference of the issue from the local repository, if
    /// present.
    ///
    pub fn local_head(&self) -> Result<Reference<'r>> {
        let refname = format!("refs/dit/{}/head", self.ref_part());
        self.repo
            .find_reference(&refname)
            .chain_err(|| EK::CannotFindIssueHead(self.id))
    }

    /// Get the leaf references for the issue
    ///
    /// Returns the leaf references for the issue from both the local repository
    /// and remotes.
    ///
    pub fn issue_leaves(&self) -> Result<References<'r>> {
        let glob = format!("**/dit/{}/leaves/*", self.ref_part());
        self.repo
            .references_glob(&glob)
            .chain_err(|| EK::CannotGetReferences(glob))
    }

    /// Get local references for the issue
    ///
    /// Return all references of a specific type associated with the issue from
    /// the local repository.
    ///
    pub fn local_refs(&self, ref_type: IssueRefType) -> Result<References<'r>> {
        let glob = format!("refs/dit/{}/{}", self.ref_part(), ref_type.glob_part());
        self.repo
            .references_glob(&glob)
            .chain_err(|| EK::CannotGetReferences(glob))
    }

    /// Get remote references for the issue
    ///
    /// Return all references of a specific type associated with the issue from
    /// all remote repositories.
    ///
    pub fn remote_refs(&self, ref_type: IssueRefType) -> Result<References<'r>> {
        let glob = format!("refs/remotes/*/dit/{}/{}", self.ref_part(), ref_type.glob_part());
        self.repo
            .references_glob(&glob)
            .chain_err(|| EK::CannotGetReferences(glob))
    }

    /// Get a revwalk for traversing all messages of the issue
    ///
    /// The sorting of the revwalk will be set to "topological".
    ///
    pub fn message_revwalk(&self) -> Result<git2::Revwalk<'r>> {
        let glob = format!("**/dit/{}/**", self.ref_part());
        self.repo
            .revwalk()
            .and_then(|mut revwalk| {
                // The iterator will iterate over all the messages in the tree
                // spanned but it will halt at the initial message.
                revwalk.push_glob(glob.as_ref())?;
                let _ = self.repo
                    .find_commit(self.id)
                    .and_then(|commit| commit.parent_id(0))
                    .ok() // the initial message having no parent is not unusual
                    .map(|parent| revwalk.hide(parent))
                    .unwrap_or(Ok(()))?;

                // configure the revwalk
                revwalk.simplify_first_parent();
                revwalk.set_sorting(git2::SORT_TOPOLOGICAL);

                Ok(revwalk)
            })
            .chain_err(|| EK::CannotGetReferences(glob))
    }

    /// Add a new message to the issue
    ///
    /// Adds a new message to the issue. Also create a leaf reference for the
    /// new message. Returns the message.
    ///
    pub fn add_message<'a, A, I, J>(&self,
                                    author: &git2::Signature,
                                    committer: &git2::Signature,
                                    message: A,
                                    tree: &git2::Tree,
                                    parents: I
    ) -> Result<Commit>
        where A: AsRef<str>,
              I: IntoIterator<Item = &'a Commit<'a>, IntoIter = J>,
              J: Iterator<Item = &'a Commit<'a>>
    {
        let parent_vec : Vec<&Commit> = parents.into_iter().collect();

        self.repo
            .commit(None, author, committer, message.as_ref(), tree, &parent_vec)
            .and_then(|id| self.repo.find_commit(id))
            .chain_err(|| EK::CannotCreateMessage)
            .and_then(|message| self.add_leaf(message.id()).map(|_| message))
    }

    /// Update the local head reference of the issue
    ///
    /// Updates the local head reference of the issue to the provided message.
    ///
    /// # Warnings
    ///
    /// The function will update the reference even if it would not be an
    /// fast-forward update.
    ///
    pub fn update_head(&self, message: Oid) -> Result<Reference> {
        let refname = format!("refs/dit/{}/head", self.ref_part());
        let reflogmsg = format!("git-dit: set head reference of {} to {}", self, message);
        self.repo
            .reference(&refname, message, true, &reflogmsg)
            .chain_err(|| EK::CannotSetReference(refname))
    }

    /// Add a new leaf reference associated with the issue
    ///
    /// Creates a new leaf reference for the message provided in the issue.
    ///
    pub fn add_leaf(&self, message: Oid) -> Result<Reference> {
        let refname = format!("refs/dit/{}/leaves/{}", self.ref_part(), message);
        let reflogmsg = format!("git-dit: new leaf for {}: {}", self, message);
        self.repo
            .reference(&refname, message, false, &reflogmsg)
            .chain_err(|| EK::CannotSetReference(refname))
    }

    /// Get reference part for this issue
    ///
    /// The references associated with an issue reside in paths specific to the
    /// issue. This function returns the part unique for the issue, e.g. the
    /// part after the  `dit/`.
    ///
    pub fn ref_part(&self) -> String {
        self.id.to_string()
    }
}

impl<'r> fmt::Display for Issue<'r> {
    fn fmt(&self, f: &mut fmt::Formatter) -> RResult<(), fmt::Error> {
        write!(f, "{}", self.id)
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::TestingRepo;

    use repository::RepositoryExt;

    // Issue tests

    #[test]
    fn issue_leaves() {
        let mut testing_repo = TestingRepo::new("issue_leaves");
        let repo = testing_repo.repo();

        let sig = git2::Signature::now("Foo Bar", "foo.bar@example.com")
            .expect("Could not create signature");
        let empty_tree = repo
            .empty_tree()
            .expect("Could not create empty tree");

        {
            // messages we're not supposed to see
            let issue = repo
                .create_issue(&sig, &sig, "Test message 1", &empty_tree, vec![])
                .expect("Could not create issue");
            let initial_message = issue
                .initial_message()
                .expect("Could not retrieve initial message");
            issue.add_message(&sig, &sig, "Test message 2", &empty_tree, vec![&initial_message])
                .expect("Could not add message");
        }

        let issue = repo
            .create_issue(&sig, &sig, "Test message 3", &empty_tree, vec![])
            .expect("Could not create issue");
        let initial_message = issue
            .initial_message()
            .expect("Could not retrieve initial message");
        let message = issue
            .add_message(&sig, &sig, "Test message 4", &empty_tree, vec![&initial_message])
            .expect("Could not add message");

        let mut leaves = issue
            .issue_leaves()
            .expect("Could not retrieve issue leaves");
        let leaf = leaves
            .next()
            .expect("Could not find leaf reference")
            .expect("Could not retrieve leaf reference")
            .target()
            .expect("Could not determine the target of the leaf reference");
        assert_eq!(leaf, message.id());
        assert!(leaves.next().is_none());
    }

    #[test]
    fn local_refs() {
        let mut testing_repo = TestingRepo::new("local_refs");
        let repo = testing_repo.repo();

        let sig = git2::Signature::now("Foo Bar", "foo.bar@example.com")
            .expect("Could not create signature");
        let empty_tree = repo
            .empty_tree()
            .expect("Could not create empty tree");

        {
            // messages we're not supposed to see
            let issue = repo
                .create_issue(&sig, &sig, "Test message 1", &empty_tree, vec![])
                .expect("Could not create issue");
            let initial_message = issue
                .initial_message()
                .expect("Could not retrieve initial message");
            issue.add_message(&sig, &sig, "Test message 3", &empty_tree, vec![&initial_message])
                .expect("Could not add message");
        }

        let issue = repo
            .create_issue(&sig, &sig, "Test message 2", &empty_tree, vec![])
            .expect("Could not create issue");
        let initial_message = issue
            .initial_message()
            .expect("Could not retrieve initial message");
        let message = issue
            .add_message(&sig, &sig, "Test message 3", &empty_tree, vec![&initial_message])
            .expect("Could not add message");

        let mut ids = vec![issue.id(), message.id()];
        ids.sort();
        let mut ref_ids: Vec<Oid> = issue
            .local_refs()
            .expect("Could not retrieve local refs")
            .map(|reference| reference.unwrap().target().unwrap())
            .collect();
        ref_ids.sort();
        assert_eq!(ref_ids, ids);
    }

    #[test]
    fn message_revwalk() {
        let mut testing_repo = TestingRepo::new("message_revwalk");
        let repo = testing_repo.repo();

        let sig = git2::Signature::now("Foo Bar", "foo.bar@example.com")
            .expect("Could not create signature");
        let empty_tree = repo
            .empty_tree()
            .expect("Could not create empty tree");

        let issue1 = repo
            .create_issue(&sig, &sig, "Test message 1", &empty_tree, vec![])
            .expect("Could not create issue");
        let initial_message1 = issue1
            .initial_message()
            .expect("Could not retrieve initial message");

        let issue2 = repo
            .create_issue(&sig, &sig, "Test message 2", &empty_tree, vec![&initial_message1])
            .expect("Could not create issue");
        let initial_message2 = issue2
            .initial_message()
            .expect("Could not retrieve initial message");
        let message = issue2
            .add_message(&sig, &sig, "Test message 3", &empty_tree, vec![&initial_message2])
            .expect("Could not add message");
        let message_id = message.id();

        let mut iter1 = issue1
            .message_revwalk()
            .expect("Could not create message revwalk iterator");
        assert_eq!(iter1.next().unwrap().unwrap(), issue1.id());
        assert!(iter1.next().is_none());

        let mut iter2 = issue2
            .message_revwalk()
            .expect("Could not create message revwalk iterator");
        assert_eq!(iter2.next().unwrap().unwrap(), message_id);
        assert_eq!(iter2.next().unwrap().unwrap(), issue2.id());
        assert!(iter2.next().is_none());
    }

    #[test]
    fn update_head() {
        let mut testing_repo = TestingRepo::new("update_head");
        let repo = testing_repo.repo();

        let sig = git2::Signature::now("Foo Bar", "foo.bar@example.com")
            .expect("Could not create signature");
        let empty_tree = repo
            .empty_tree()
            .expect("Could not create empty tree");

        let issue = repo
            .create_issue(&sig, &sig, "Test message 2", &empty_tree, vec![])
            .expect("Could not create issue");
        let initial_message = issue
            .initial_message()
            .expect("Could not retrieve initial message");
        let message = issue
            .add_message(&sig, &sig, "Test message 3", &empty_tree, vec![&initial_message])
            .expect("Could not add message");

        assert_eq!(issue.find_local_head().unwrap().target().unwrap(), issue.id());

        issue
            .update_head(message.id())
            .expect("Could not update head reference");
        assert_eq!(issue.find_local_head().unwrap().target().unwrap(), message.id());
    }
}

