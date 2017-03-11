// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use git2::{Commit, Oid, References, Repository};

use error::*;
use error::ErrorKind as EK;
use first_parent_iter::FirstParentIter;
use iter::HeadRefsToIssuesIter;

pub trait RepositoryExt {
    /// Get possible heads of an issue by its oid
    ///
    /// Returns heads from both the local repository and remotes for the issue
    /// provided.
    ///
    fn get_issue_heads(&self, issue: Oid) -> Result<References>;

    /// Get leaf references of an issue by its oid
    ///
    /// Returns leaf references from both the local repository and remotes for
    /// the issue provided.
    ///
    fn get_issue_leaves(&self, issue: Oid) -> Result<References>;

    /// Find the initial message of an issue
    ///
    /// For a given message of an issue, find the initial message.
    ///
    fn find_tree_init<'a>(&'a self, commit: Commit<'a>) -> Result<Commit>;

    /// Get issue hashes for a prefix
    ///
    /// This function returns all known issues known to the DIT repo under the
    /// prefix provided (e.g. all issues for which refs exist under
    /// `<prefix>/dit/`). Provide "refs" as the prefix to get only local issues.
    ///
    fn get_issue_hashes(&self, prefix: &str) -> Result<HeadRefsToIssuesIter>;

    /// Get all issue hashes
    ///
    /// This function returns all known issues known to the DIT repo.
    ///
    fn get_all_issue_hashes(&self) -> Result<HeadRefsToIssuesIter>;
}

impl RepositoryExt for Repository {
    fn get_issue_heads(&self, issue: Oid) -> Result<References> {
        let glob = format!("**/dit/{}/head", issue);
        self.references_glob(&glob)
            .chain_err(|| EK::WrappedGitError)
    }

    fn get_issue_leaves(&self, issue: Oid) -> Result<References> {
        let glob = format!("**/dit/{}/leaves/*", issue);
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

    fn get_issue_hashes(&self, prefix: &str) -> Result<HeadRefsToIssuesIter> {
        let glob = format!("{}/dit/**/head", prefix);
        Ok(HeadRefsToIssuesIter::from(try!(self.references_glob(&glob))))
    }

    fn get_all_issue_hashes(&self) -> Result<HeadRefsToIssuesIter> {
        Ok(HeadRefsToIssuesIter::from(try!(self.references_glob("**/dit/**/head"))))
    }
}

