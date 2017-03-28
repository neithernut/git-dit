//   git-dit - the distributed issue tracker for git
//   Copyright (C) 2017 Matthias Beyer <mail@beyermatthias.de>
//   Copyright (C) 2017 Julian Ganz <neither@nut.email>
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License version 2 as
//   published by the Free Software Foundation.
//

use clap::Values;
use git2::{Commit, Repository};
use std::path::PathBuf;

use error::ErrorKind as EK;
use error::*;

/// Open the DIT repo
///
/// Opens the DIT repo corresponding to the current one honouring the user
/// configuration.
///
pub fn open_dit_repo() -> Result<Repository> {
    // TODO: access the config and maybe return another repo instead
    Repository::open_from_env().chain_err(|| EK::WrappedGitError)
}


/// Utility trait for some repository-specific functionality
///
pub trait RepositoryUtil<'r> {
    /// Get a commit from a rev
    ///
    /// This function returns a commit for a rev-string.
    ///
    fn value_to_commit(&'r self, rev: &str) -> Result<Commit<'r>>;

    /// Get a vector of commits from values
    ///
    /// This function transforms values to a vector.
    ///
    fn values_to_hashes(&'r self, values: Values) -> Result<Vec<Commit<'r>>>;

    /// Get the path to the file usually used to edit comit messages
    fn commitmsg_edit_path(&self) -> PathBuf;
}

impl<'r> RepositoryUtil<'r> for Repository {
    fn value_to_commit(&'r self, rev: &str) -> Result<Commit<'r>> {
        self.revparse_single(rev)
            .and_then(|oid| self.find_commit(oid.id()))
            .chain_err(|| EK::WrappedGitError)
    }

    fn values_to_hashes(&'r self, values: Values) -> Result<Vec<Commit<'r>>> {
        let mut retval = Vec::new();
        for commit in values.map(|string| self.value_to_commit(string)) {
            retval.push(try!(commit));
        }
        Ok(retval)
    }

    fn commitmsg_edit_path(&self) -> PathBuf {
        self.path().with_file_name("COMMIT_EDITMSG")
    }
}

