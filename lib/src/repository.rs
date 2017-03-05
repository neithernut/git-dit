// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use error::ErrorKind as EK;
use error::*;

use git2::{Commit, Oid, References, Repository};
use first_parent_iter::FirstParentIter;


pub trait RepositoryExt {
    /// Get possible heads of an issue by its oid
    ///
    /// Returns heads from both the local repository and remotes for the issue
    /// provided.
    ///
    fn get_issue_heads(&self, issue: Oid) -> Result<References>;

    /// Find the initial message of an issue
    ///
    /// For a given message of an issue, find the initial message.
    ///
    fn find_tree_init<'a>(&'a self, commit: Commit<'a>) -> Result<Commit>;
}

impl RepositoryExt for Repository {
    fn get_issue_heads(&self, issue: Oid) -> Result<References> {
        let glob = format!("**/dit/{}/head", issue);
        self.references_glob(&glob)
            .chain_err(|| EK::WrappedGitError)
    }

    fn find_tree_init<'a>(&'a self, commit: Commit<'a>) -> Result<Commit> {
        // follow the chain of first parents towards an initial message for
        // which a head exists
        let cid = commit.id();
        for c in FirstParentIter::new(commit) {
            let head = try!(self
                            .get_issue_heads(c.id())
                            .chain_err(|| EK::CannotFindIssueHead(c.id())));

            if head.count() > 0 {
                return Ok(c);
            }
        }

        Err(Error::from_kind(EK::NoTreeInitFound(cid)))
    }
}

