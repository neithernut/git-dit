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
use std::io;
use std::path::PathBuf;
use std::str::FromStr;

use libgitdit::message::LineIteratorExt;
use libgitdit::repository::UniqueIssues;
use libgitdit::trailer::Trailer;
use libgitdit::{Issue, RepositoryExt};

use error::*;
use error::ErrorKind as EK;
use gitext::RemotePriorization;
use system::{Abortable, IteratorExt, LoggableError};

/// Open the DIT repo
///
/// Opens the DIT repo corresponding to the current one honouring the user
/// configuration.
///
pub fn open_dit_repo() -> Repository {
    // TODO: access the config and maybe return another repo instead
    Repository::open_from_env().unwrap_or_abort()
}


/// Utility trait for some repository-specific functionality
///
pub trait RepositoryUtil<'r> {
    /// Get a commit from a rev
    ///
    /// This function returns a commit for a rev-string.
    ///
    fn value_to_commit(&'r self, rev: &str) -> Commit<'r>;

    /// Get a vector of commits from values
    ///
    /// This function transforms values to a vector.
    ///
    fn values_to_commits(&'r self, values: Values) -> Vec<Commit<'r>>;

    /// Get the issue specified on the command line
    ///
    /// This function parses the issue specified via the `"issue"` field.
    ///
    fn cli_issue(&'r self, matches: &ArgMatches) -> Option<Issue<'r>>;

    /// Get the issues specified on the command line
    ///
    /// This function parses the issues specified via the `"issue"` field.
    ///
    fn cli_issues(&'r self, matches: &ArgMatches) -> Option<UniqueIssues<'r>>;

    /// Retrieve the references from the command line
    ///
    fn cli_references(&'r self, matches: &ArgMatches) -> Vec<Commit<'r>>;

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
    fn get_commit_msg(&self, path: PathBuf) -> Vec<String>;

    /// Retrieve metadata from command line arguments
    ///
    fn prepare_trailers(&self, matches: &ArgMatches) -> Vec<Trailer>;

    /// Get the abbreviation length for oids
    ///
    fn abbreviation_length(&self, matches: &ArgMatches) -> usize;

    /// Get remote priorization from the config
    fn remote_priorization(&self) -> RemotePriorization;
}

impl<'r> RepositoryUtil<'r> for Repository {
    fn value_to_commit(&'r self, rev: &str) -> Commit<'r> {
        self.revparse_single(rev)
            .and_then(|oid| self.find_commit(oid.id()))
            .unwrap_or_abort()
    }

    fn values_to_commits(&'r self, values: Values) -> Vec<Commit<'r>> {
        values.map(|string| self.value_to_commit(string)).collect()
    }

    fn commitmsg_edit_path(&self, matches: &ArgMatches) -> PathBuf {
        matches.value_of("tempfile")
               .map(PathBuf::from)
               .unwrap_or_else(|| self.path().join("COMMIT_EDITMSG"))
    }

    fn cli_issue(&'r self, matches: &ArgMatches) -> Option<Issue<'r>> {
        matches.value_of("issue")
               .map(|value| value_to_issue(self, value))
    }

    fn cli_issues(&'r self, matches: &ArgMatches) -> Option<UniqueIssues<'r>> {
        matches
            .values_of("issue")
            .map(|values| values
                .map(|issue| value_to_issue(self, issue))
                .collect()
            )
    }

    fn cli_references(&'r self, matches: &ArgMatches) -> Vec<Commit<'r>> {
        matches
            .values_of("reference")
            .map(|p| self.values_to_commits(p))
            .unwrap_or_default()
    }

    fn get_commit_msg(&self, path: PathBuf) -> Vec<String> {
        use system::programs::run_editor;

        // let the user write the message
        if !run_editor(self.config().unwrap_or_abort(), &path)
            .unwrap_or_abort()
            .wait()
            .unwrap_or_abort()
            .success()
        {
            Error::from_kind(EK::ChildError).log();
            ::std::process::exit(1);
        }

        // read the message back, check for validity
        use io::BufRead;
        let lines : Vec<String> = io::BufReader::new(File::open(path).unwrap_or_abort())
            .lines()
            .abort_on_err()
            .stripped()
            .collect();

        lines
            .iter()
            .check_message_format()
            .unwrap_or_abort();

        lines
    }

    fn prepare_trailers(&self, matches: &ArgMatches) -> Vec<Trailer> {
        let mut trailers = Vec::new();

        if matches.is_present("signoff") {
            let sig = self.signature().unwrap_or_abort().to_string();
            trailers.push(Trailer::new("Signed-off-by", sig.as_str()));
        }

        // append misc metadata
        if let Some(metadata) = matches.values_of("metadata") {
            for trailer in metadata.map(Trailer::from_str) {
                trailers.push(trailer.unwrap_or_abort());
            }
        }

        trailers
    }

    fn abbreviation_length(&self, matches: &ArgMatches) -> usize {
        if !matches.is_present("abbrev") {
            // If the abbreviation option was not used, we can just use the
            // known length of a hash.
            // TODO: have this compile-time at some prominent place
            return 40;
        }

        // TODO: the following _might_ be simplified using the `programs::Var`
        //       enum in the future.

        if let Some(number) = matches.value_of("abbrev") {
            // The abbreviation flas might have been specified with a value.
            return str::parse(number).unwrap_or_abort();
        }

        if let Some(number) = self.config().unwrap_or_abort().get_i32("core.abbrev").ok() {
            // The abbreviation flag might have been specified as a configuration option
            return number as usize;
        }

        // TODO: use a larger number based on the number of objects in the repo
        7
    }

    fn remote_priorization(&self) -> RemotePriorization {
        self.config()
            .unwrap_or_abort()
            .get_str("dit.remote-prios")
            .unwrap_or("*")
            .into()
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


/// Get an issue from a string representation
///
/// This function returns an issue from a string representation.
///
fn value_to_issue<'r>(repo: &'r Repository, value: &str) -> Issue<'r> {
    let id = git2::Oid::from_str(value).unwrap_or_abort();
    repo.find_issue(id).unwrap_or_abort()
}

