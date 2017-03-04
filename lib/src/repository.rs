// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
use git2::{Commit, Error, Oid, References, Repository};


pub trait RepositoryExt {
    /// Get possible heads of an issue by its oid
    ///
    /// Returns heads from both the local repository and remotes for the issue
    /// provided.
    ///
    fn get_issue_heads(&self, issue: Oid) -> Result<References, Error>;

    /// Find the initial message of an issue
    ///
    /// For a given message of an issue, find the initial message.
    ///
    fn find_tree_init<'a>(&'a self, commit: Commit<'a>) -> Result<Commit, Error>;
}

impl RepositoryExt for Repository {
    fn get_issue_heads(&self, issue: Oid) -> Result<References, Error> {
        let glob = format!("**/dit/{}/head", issue);
        self.references_glob(&glob)
    }

    fn find_tree_init<'a>(&'a self, commit: Commit<'a>) -> Result<Commit, Error> {
        // we start with the commit passed itself
        let mut current: Result<Commit, Error> = Ok(commit);

        // We try to get the issue head for the current commit.
        while let Some(heads) = current.iter()
                     .map(|commit| commit.id())
                     .map(|id| self.get_issue_heads(id))
                     .next() {
            if try!(heads).count() > 0 {
                // The current commit's id appears to be an issue id,
                // as there are heads for it.
                break;
            }

            // No lock, try the next one.
            current = current.unwrap().parent(0)
        }

        current
    }
}

