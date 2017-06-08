//   git-dit - the distributed issue tracker for git
//   Copyright (C) 2017 Matthias Beyer <mail@beyermatthias.de>
//   Copyright (C) 2017 Julian Ganz <neither@nut.email>
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License version 2 as
//   published by the Free Software Foundation.
//

use clap::{ArgMatches, Values};
use git2::{self, Commit, Repository};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use std::str::FromStr;

use abort::IteratorExt;
use error::ErrorKind as EK;
use error::*;
use programs::run_editor;
use libgitdit::{Issue, RepositoryExt};
use libgitdit::message::LineIteratorExt;
use libgitdit::message::trailer::Trailer;

/// Open the DIT repo
///
/// Opens the DIT repo corresponding to the current one honouring the user
/// configuration.
///
pub fn open_dit_repo() -> Result<Repository> {
    // TODO: access the config and maybe return another repo instead
    Repository::open_from_env().chain_err(|| EK::CannotOpenRepository)
}


/// Utility trait for some repository-specific functionality
///
pub trait RepositoryUtil<'r> {
    /// Get a commit from a rev
    ///
    /// This function returns a commit for a rev-string.
    ///
    fn value_to_commit(&'r self, rev: &str) -> Result<Commit<'r>>;

    /// Get an issue from a string representation
    ///
    /// This function returns an issue from a string representation.
    ///
    fn value_to_issue(&'r self, value: &str) -> Result<Issue<'r>>;

    /// Get a vector of commits from values
    ///
    /// This function transforms values to a vector.
    ///
    fn values_to_hashes(&'r self, values: Values) -> Result<Vec<Commit<'r>>>;

    /// Get the issue specified on the command line
    ///
    /// This function parses the issue specified via the `"issue"` field.
    ///
    fn cli_issue(&'r self, matches: &ArgMatches) -> Result<Issue<'r>>;

    /// Retrieve the references from the command line
    ///
    fn cli_references(&'r self, matches: &ArgMatches) -> Result<Vec<Commit<'r>>>;

    /// Get the path to the file usually used to edit comit messages
    fn commitmsg_edit_path(&self, matches: &ArgMatches) -> PathBuf;

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

    /// Get the abbreviation length for oids
    ///
    fn abbreviation_length(&self, matches: &ArgMatches) -> Result<usize>;
}

impl<'r> RepositoryUtil<'r> for Repository {
    fn value_to_commit(&'r self, rev: &str) -> Result<Commit<'r>> {
        self.revparse_single(rev)
            .and_then(|oid| self.find_commit(oid.id()))
            .chain_err(|| EK::WrappedGitDitError)
    }

    fn value_to_issue(&'r self, value: &str) -> Result<Issue<'r>> {
        git2::Oid::from_str(value)
            .chain_err(|| EK::WrappedParseError)
            .and_then(|id| {
                self.find_issue(id).chain_err(|| EK::WrappedGitDitError)
            })
    }

    fn values_to_hashes(&'r self, values: Values) -> Result<Vec<Commit<'r>>> {
        let mut retval = Vec::new();
        for commit in values.map(|string| self.value_to_commit(string)) {
            retval.push(try!(commit));
        }
        Ok(retval)
    }

    fn commitmsg_edit_path(&self, matches: &ArgMatches) -> PathBuf {
        matches.value_of("tempfile")
               .map(PathBuf::from)
               .unwrap_or_else(|| self.path().join("COMMIT_EDITMSG"))
    }

    fn cli_issue(&'r self, matches: &ArgMatches) -> Result<Issue<'r>> {
        matches.value_of("issue")
               .ok_or_else(|| {
                   Error::from_kind(EK::ParameterMissing("issue".to_owned()))
               })
               .and_then(|value| self.value_to_issue(value))
    }

    fn cli_references(&'r self, matches: &ArgMatches) -> Result<Vec<Commit<'r>>> {
        matches.values_of("reference")
               .map(|p| self.values_to_hashes(p))
               .unwrap_or(Ok(vec![]))
    }

    fn get_commit_msg(&self, path: PathBuf) -> Result<Vec<String>> {
        // let the user write the message
        if !run_editor(self.config().chain_err(|| EK::CannotGetRepositoryConfig)?, &path)?
            .wait().chain_err(|| EK::WrappedIOError)?
            .success()
        {
            return Err(Error::from_kind(EK::ChildError));
        }

        // read the message back, check for validity
        let lines : Vec<String> = BufReader::new(File::open(path).chain_err(|| EK::WrappedIOError)?)
            .lines()
            .abort_on_err()
            .stripped()
            .collect();

        lines.iter()
            .check_message_format()
            .chain_err(|| EK::WrappedGitDitError)?;

        Ok(lines)
    }

    fn prepare_trailers(&self, matches: &ArgMatches) -> Result<Vec<Trailer>> {
        let mut trailers = Vec::new();

        if matches.is_present("signoff") {
            let sig = self.signature().chain_err(|| EK::CannotGetSignature)?.to_string();
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

    fn abbreviation_length(&self, matches: &ArgMatches) -> Result<usize> {
        if !matches.is_present("abbrev") {
            // If the abbreviation option was not used, we can just use the
            // known length of a hash.
            // TODO: have this compile-time at some prominent place
            return Ok(40);
        }

        // TODO: the following _might_ be simplified using the `programs::Var`
        //       enum in the future.

        if let Some(number) = matches.value_of("abbrev") {
            // The abbreviation flas might have been specified with a value.
            return str::parse(number).chain_err(|| EK::WrappedParseError);
        }

        if let Some(number) = self.config().and_then(|c| c.get_i32("core.abbrev")).ok() {
            // The abbreviation flag might have been specified as a configuration option
            return Ok(number as usize);
        }

        // TODO: use a larger number based on the number of objects in the repo
        Ok(7)
    }
}

/// Get the message specified on the command line, as lines
///
/// Retrieve the message specified on the command line. If no paragraph was
/// specified, an empty vector will be returned.
///
pub fn message_from_args(matches: &ArgMatches) -> Option<Vec<String>> {
    matches.values_of("message")
           .map(|ps| ps.map(str::to_owned)
                       .map(|p| (p + "\n").to_owned()) // paragraphs
                       .collect())
}

