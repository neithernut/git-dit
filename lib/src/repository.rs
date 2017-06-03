// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

//! Repository related utilities
//!
//! This module provides the `RepositoryExt` extension trait which provides
//! issue handling utilities for repositories.
//!

use git2::{self, Commit, Oid, Repository, Signature, Tree};

use issue::Issue;
use error::*;
use error::ErrorKind as EK;
use first_parent_iter::FirstParentIter;
use iter::HeadRefsToIssuesIter;


/// Extension trait for Repositories
///
/// This trait is intended as an extension for repositories. It introduces
/// utility functions for dealing with issues, e.g. for retrieving references
/// for issues, creating messages and finding the initial message of an issue.
///
pub trait RepositoryExt {
    /// Retrieve an issue
    ///
    /// Returns the issue with a given id.
    ///
    fn find_issue(&self, id: Oid) -> Result<Issue>;

    /// Retrieve an issue by its head ref
    ///
    /// Returns the issue associated with a head reference.
    ///
    fn issue_by_head_ref(&self, head_ref: &git2::Reference) -> Result<Issue>;

    /// Find the issue with a given message in it
    ///
    /// Returns the issue containing the message provided
    ///
    fn issue_with_message<'a>(&'a self, message: &Commit<'a>) -> Result<Issue>;

    /// Get issue hashes for a prefix
    ///
    /// This function returns all known issues known to the DIT repo under the
    /// prefix provided (e.g. all issues for which refs exist under
    /// `<prefix>/dit/`). Provide "refs" as the prefix to get only local issues.
    ///
    fn issues_with_prefix(&self, prefix: &str) -> Result<HeadRefsToIssuesIter>;

    /// Get all issue hashes
    ///
    /// This function returns all known issues known to the DIT repo.
    ///
    fn issues(&self) -> Result<HeadRefsToIssuesIter>;

    /// Create a new message
    ///
    /// This function creates a new issue message as well as an appropriate
    /// reference. The oid of the new message will be returned.
    /// The message will be part of the issue supplied by the caller. If no
    /// issue is provided, a new issue will be initiated with the message.
    /// In this case, the oid returned is also the oid of the new issue.
    ///
    fn create_message(&self,
                      issue: Option<&Oid>,
                      author: &Signature,
                      committer: &Signature,
                      message: &str,
                      tree: &Tree,
                      parents: &[&Commit]
                     ) -> Result<Oid>;

    /// Get an empty tree
    ///
    /// This function returns an empty tree.
    ///
    fn empty_tree(&self) -> Result<Tree>;
}

impl RepositoryExt for Repository {
    fn find_issue(&self, id: Oid) -> Result<Issue> {
        let retval = Issue::new(self, id);

        // make sure the id refers to an issue by checking whether an associated
        // head reference exists
        if retval.heads()?.next().is_some() {
            Ok(retval)
        } else {
            Err(Error::from_kind(EK::CannotFindIssueHead(id)))
        }
    }

    fn issue_by_head_ref(&self, head_ref: &git2::Reference) -> Result<Issue> {
        let name = head_ref.name();
        name.and_then(|name| if name.ends_with("/head") {
                Some(name)
            } else {
                None
            })
            .and_then(|name| name.rsplitn(3, "/").nth(1))
            .ok_or_else(|| {
                let n = name.unwrap_or_default().to_owned();
                Error::from_kind(EK::MalFormedHeadReference(n))
            })
            .and_then(|hash| {
               Oid::from_str(hash)
                   .chain_err(|| EK::OidFormatError(hash.to_string()))
            })
            .map(|id| Issue::new(self, id))
    }

    fn issue_with_message<'a>(&'a self, message: &Commit<'a>) -> Result<Issue> {
        // follow the chain of first parents towards an initial message for
        // which a head exists
        let cid = message.id();
        // NOTE: The following is this ugly because `Clone` is not implemented
        //       for `git2::Commit`. We take a reference because consuming the
        //       commit doesn't make sense for this function, semantically.
        for c in FirstParentIter::new(message.as_object().clone().into_commit().ok().unwrap()) {
            let issue = self.find_issue(c.id());
            if issue.is_ok() {
                return issue
            }
        }

        Err(Error::from_kind(EK::NoTreeInitFound(cid)))
    }

    fn issues_with_prefix(&self, prefix: &str) -> Result<HeadRefsToIssuesIter> {
        let glob = format!("{}/dit/**/head", prefix);
        self.references_glob(&glob)
            .chain_err(|| EK::CannotGetReferences(glob))
            .map(|refs| HeadRefsToIssuesIter::new(self, refs))
    }

    fn issues(&self) -> Result<HeadRefsToIssuesIter> {
        let glob = "**/dit/**/head";
        self.references_glob(glob)
            .chain_err(|| EK::CannotGetReferences(glob.to_owned()))
            .map(|refs| HeadRefsToIssuesIter::new(self, refs))
    }

    fn create_message(&self,
                      issue: Option<&Oid>,
                      author: &Signature,
                      committer: &Signature,
                      message: &str,
                      tree: &Tree,
                      parents: &[&Commit]
                     ) -> Result<Oid> {
        // commit message
        let msg_id = try!(self.commit(None, author, committer, message, tree, parents));

        // make an apropriate reference
        let refname =  match issue {
            Some(hash)  => format!("refs/dit/{}/leaves/{}", hash, msg_id),
            _           => format!("refs/dit/{}/head", msg_id),
        };
        let reflogmsg = format!("new dit message: {}", msg_id);
        try!(self.reference(&refname, msg_id, false, &reflogmsg));

        Ok(msg_id)
    }

    fn empty_tree(&self) -> Result<Tree> {
        self.treebuilder(None)
            .and_then(|treebuilder| treebuilder.write())
            .and_then(|oid| self.find_tree(oid))
            .chain_err(|| EK::CannotBuildTree)
    }
}

