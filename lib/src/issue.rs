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
use iter::Messages;


#[derive(PartialEq)]
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

    /// Get the issue ref type assiciated with a reference
    ///
    /// This functio ndetermines the issue ref type and returns th type as well
    /// as the issue id as a bonus. If the type of reference could not be
    /// determined or the ref doesn't appear to belong into the dit context,
    /// this function returns `None`.
    ///
    pub fn of_ref(refname: &str) -> Option<(Oid, IssueRefType)> {
        let mut parts = refname.rsplit('/');

        // The ref type is denominated by the last few elements.
        let preliminary_ref_type = match parts.next() {
            Some("head") => IssueRefType::Head,
            Some(part) => if Self::id_from_str(part).is_some() {
                // The last element might be an id, in which case the second
                // last part should tell us the meaning of the id.
                match parts.next() {
                    Some("leaves") => IssueRefType::Leaf,
                    _ => return None,
                }
            } else {
                return None
            },
            None => return None,
        };

        // The denominating end of the reference is preceeded by an issue id of
        // some sort.
        if let Some(id) = parts.next().and_then(Self::id_from_str) {
            // A dit reference also has to contain a "dit" denominator at some
            // point.
            if parts.any(|part| part == "dit") {
                return Some((id, preliminary_ref_type));
            }
        }

        None
    }

    /// Create an Oid from a full 40-character representation
    ///
    /// If the number of characters is not exactly 40 or the string is not an
    /// Oid-representation, `None` is returned.
    ///
    fn id_from_str(id: &str) -> Option<Oid> {
        if id.len() == 40 {
            Oid::from_str(id).ok()
        } else {
            None
        }
    }
}

impl fmt::Debug for IssueRefType {
    fn fmt(&self, f: &mut fmt::Formatter) -> RResult<(), fmt::Error> {
        f.write_str(match self {
            &IssueRefType::Any   => "Any ref",
            &IssueRefType::Head  => "Head ref",
            &IssueRefType::Leaf  => "Leaf ref",
        })
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
    pub fn initial_message(&self) -> Result<git2::Commit<'r>> {
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

    /// Get references for the issue
    ///
    /// Return all references of a specific type associated with the issue from
    /// both the local and remote repositories.
    ///
    pub fn all_refs(&self, ref_type: IssueRefType) -> Result<References<'r>> {
        let glob = format!("**/dit/{}/{}", self.ref_part(), ref_type.glob_part());
        self.repo
            .references_glob(&glob)
            .chain_err(|| EK::CannotGetReferences(glob))
    }

    /// Get all Messages of the issue
    ///
    /// The sorting of the underlying revwalk will be set to "topological".
    ///
    pub fn messages(&self) -> Result<Messages<'r>> {
        self.terminated_messages()
            .and_then(|mut messages| {
                let glob = format!("**/dit/{}/**", self.ref_part());

                // The iterator will iterate over all the messages in the tree
                // spanned but it will halt at the initial message.
                messages
                    .revwalk
                    .push_glob(glob.as_ref())
                    .chain_err(|| EK::CannotGetReferences(glob))?;

                Ok(messages)
            })
    }

    /// Get Messages of the issue starting from a specific one
    ///
    /// The Messages iterator returned will return all first parents up to and
    /// includingthe initial message of the issue.
    ///
    pub fn messages_from(&self, message: Oid) -> Result<Messages<'r>> {
        self.terminated_messages()
            .and_then(|mut messages| {
                messages
                    .revwalk
                    .push(message)
                    .chain_err(|| EK::CannotConstructRevwalk)?;

                Ok(messages)
            })
    }

    /// Prepare a Messages iterator which will terminate at the initial message
    ///
    pub fn terminated_messages(&self) -> Result<Messages<'r>> {
        Messages::empty(self.repo)
            .and_then(|mut messages| {
                // terminate at this issue's initial message
                messages.terminate_at_initial(self)?;

                // configure the revwalk
                messages.revwalk.simplify_first_parent();
                messages.revwalk.set_sorting(git2::SORT_TOPOLOGICAL);

                Ok(messages)
            })
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
    ) -> Result<Commit<'r>>
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
    pub fn update_head(&self, message: Oid, replace: bool) -> Result<Reference<'r>> {
        let refname = format!("refs/dit/{}/head", self.ref_part());
        let reflogmsg = format!("git-dit: set head reference of {} to {}", self, message);
        self.repo
            .reference(&refname, message, replace, &reflogmsg)
            .chain_err(|| EK::CannotSetReference(refname))
    }

    /// Add a new leaf reference associated with the issue
    ///
    /// Creates a new leaf reference for the message provided in the issue.
    ///
    pub fn add_leaf(&self, message: Oid) -> Result<Reference<'r>> {
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

impl<'r> PartialEq for Issue<'r> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<'r> Eq for Issue<'r> {}




#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::TestingRepo;

    use repository::RepositoryExt;

    // IssueRefType tests

    #[test]
    fn ref_identification() {
        {
            let (id, reftype) = IssueRefType::of_ref("refs/dit/65b56706fdc3501749d008750c61a1f24b888f72/head")
                .expect("Expected valid issue id and ref type");
            assert_eq!(id.to_string(), "65b56706fdc3501749d008750c61a1f24b888f72");
            assert_eq!(reftype, IssueRefType::Head);
        }
        {
            let (id, reftype) = IssueRefType::of_ref("refs/dit/65b56706fdc3501749d008750c61a1f24b888f72/leaves/f6bd121bdc2ba5906e412da19191a2eaf2025755")
                .expect("Expected valid issue id and ref type");
            assert_eq!(id.to_string(), "65b56706fdc3501749d008750c61a1f24b888f72");
            assert_eq!(reftype, IssueRefType::Leaf);
        }

        assert!(IssueRefType::of_ref("refs/dit/65b56706fdc3501749d008750c61a1f24b888f72/foo/f6bd121bdc2ba5906e412da19191a2eaf2025755").is_none());
        assert!(IssueRefType::of_ref("refs/dit/65b56706fdc3501749d008750c61a1f24b888f72/head/foo").is_none());
        assert!(IssueRefType::of_ref("refs/dit/65b56706fdc3501749d008750c61a1f24b888f72/leaves/foo").is_none());
        assert!(IssueRefType::of_ref("refs/dit/foo/leaves/f6bd121bdc2ba5906e412da19191a2eaf2025755").is_none());
        assert!(IssueRefType::of_ref("refs/dit/foo/head").is_none());
        assert!(IssueRefType::of_ref("refs/foo/65b56706fdc3501749d008750c61a1f24b888f72/head").is_none());
        assert!(IssueRefType::of_ref("refs/foo/65b56706fdc3501749d008750c61a1f24b888f72/leaves/f6bd121bdc2ba5906e412da19191a2eaf2025755").is_none());
    }

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
            .local_refs(IssueRefType::Leaf)
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
            .local_refs(IssueRefType::Any)
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
            .messages()
            .expect("Could not create message revwalk iterator");
        assert_eq!(iter1.next().unwrap().unwrap().id(), issue1.id());
        assert!(iter1.next().is_none());

        let mut iter2 = issue2
            .messages()
            .expect("Could not create message revwalk iterator");
        assert_eq!(iter2.next().unwrap().unwrap().id(), message_id);
        assert_eq!(iter2.next().unwrap().unwrap().id(), issue2.id());
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

        assert_eq!(issue.local_head().unwrap().target().unwrap(), issue.id());

        issue
            .update_head(message.id(), true)
            .expect("Could not update head reference");
        assert_eq!(issue.local_head().unwrap().target().unwrap(), message.id());
    }
}

