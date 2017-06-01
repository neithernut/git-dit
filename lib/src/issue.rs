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

use git2::{self, Oid, Reference, References};
use std::fmt;
use std::result::Result as RResult;

use error::*;
use error::ErrorKind as EK;


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

    /// Get possible heads of the issue
    ///
    /// Returns the head references from both the local repository and remotes
    /// for this issue.
    ///
    pub fn heads(&self) -> Result<References<'r>> {
        let glob = format!("**/dit/{}/head", self.unique_ref_part());
        self.repo
            .references_glob(&glob)
            .chain_err(|| EK::CannotFindIssueHead(self.id))
    }

    /// Get the local issue head for the issue
    ///
    /// Returns the head reference of the issue from the local repository, if
    /// present.
    ///
    pub fn find_local_head(&self) -> Result<Reference<'r>> {
        let refname = format!("refs/dit/{}/head", self.unique_ref_part());
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
        let glob = format!("**/dit/{}/leaves/*", self.unique_ref_part());
        self.repo
            .references_glob(&glob)
            .chain_err(|| EK::CannotGetReferences(glob))
    }

    /// Get all local references for the issue
    ///
    /// Return all references associated with the issue from the local
    /// repository.
    ///
    pub fn local_refs(&self) -> Result<References<'r>> {
        let glob = format!("refs/dit/{}/**", self.unique_ref_part());
        self.repo
            .references_glob(&glob)
            .chain_err(|| EK::CannotGetReferences(glob))
    }

    /// Get a revwalk for traversing all messages of the issue
    ///
    /// The sorting of the revwalk will be set to "topological".
    ///
    pub fn message_revwalk(&self) -> Result<git2::Revwalk<'r>> {
        let glob = format!("**/dit/{}/**", self.unique_ref_part());
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

    /// Get reference part unique for this issue
    ///
    /// The references associated with an issue reside in paths specific to the
    /// issue. This function returns the part unique for the issue, e.g. the
    /// part after the  `dit/`.
    ///
    fn unique_ref_part(&self) -> String {
        self.id.to_string()
    }
}

impl<'r> fmt::Display for Issue<'r> {
    fn fmt(&self, f: &mut fmt::Formatter) -> RResult<(), fmt::Error> {
        write!(f, "{}", self.id)
    }
}

