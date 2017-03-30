//   git-dit - the distributed issue tracker for git
//   Copyright (C) 2017 Matthias Beyer <mail@beyermatthias.de>
//   Copyright (C) 2017 Julian Ganz <neither@nut.email>
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License version 2 as
//   published by the Free Software Foundation.
//

use clap::{ArgMatches, Values};
use git2::{Commit, Repository};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

use error::ErrorKind as EK;
use error::*;
use programs::run_editor;
use libgitdit::message::LineIteratorExt;
use libgitdit::message::trailer::Trailer;

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

    /// Get a commit message
    ///
    /// An editor will be spawned for editting the file specified by the path
    /// supplied. After editting, the file will be read back, stripped and
    /// checked for validity. If the comit message is valid, it will be
    /// returned.
    ///
    /// Note: the pathbuf is consumed since we assume that the fill will not be
    ///       used after the commit message is read back.
    ///
    fn get_commit_msg(&self, path: PathBuf) -> Result<Vec<String>>;

    /// Retrieve metadata from command line arguments
    ///
    fn prepare_trailers(&self, matches: &ArgMatches) -> Result<Vec<Trailer>>;
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
        self.path().join("COMMIT_EDITMSG")
    }

    fn get_commit_msg(&self, path: PathBuf) -> Result<Vec<String>> {
        // let the user write the message
        if !run_editor(self.config().chain_err(|| EK::WrappedGitError)?, &path)?
            .wait().chain_err(|| EK::WrappedIOError)?
            .success()
        {
            return Err(Error::from_kind(EK::ChildError));
        }

        // read the message back, check for validity
        let lines : Vec<String> = BufReader::new(File::open(path).chain_err(|| EK::WrappedIOError)?)
            .lines()
            .map(|l| l.unwrap_or_else(|err| {
                // abort on IO errors
                error!("{:?}", err);
                exit(1);
            }))
            .stripped()
            .collect();

        lines.iter().check_message_format().chain_err(|| EK::WrappedGitDitError)?;

        Ok(lines)
    }

    fn prepare_trailers(&self, matches: &ArgMatches) -> Result<Vec<Trailer>> {
        let mut trailers = Vec::new();

        if matches.is_present("signoff") {
            let sig = self.signature().chain_err(|| EK::WrappedGitError)?.to_string();
            trailers.push(Trailer::new("Signed-off-by", sig.as_str()));
        }

        // append misc metadata
        if let Some(metadata) = matches.values_of("metadata") {
            for trailer in metadata.map(Trailer::from_str) {
                trailers.push(trailer?);
            }
        }

        Ok(trailers)
    }
}

