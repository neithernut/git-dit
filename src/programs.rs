// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use std::env::var as env_var;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use git2::Config;

use error::*;
use error::ErrorKind as EK;


/// Representation of variables
///
/// Instances represent variables from various sources.
///
enum Var<'a> {
    Environ(&'a str),
    GitConf(&'a str),
    Default(&'a str),
}

impl<'a> Var<'a> {
    /// Get the value of the variable
    ///
    /// Tries to retrieve the variable from the source.
    ///
    pub fn value(&self, config: &Config) -> Option<String> {
        match self {
            &Var::Environ(name) => env_var(name).ok(),
            &Var::GitConf(name) => config.get_str(name).map(String::from).ok(),
            &Var::Default(value) => Some(value.to_owned()),
        }
    }
}


/// Conveniece function for command assembly
///
/// This assembles a command from a slice of possible sources for the name of
/// the program, or returns an error containing the name provided.
///
fn command(name: &str, prefs: &[Var], config: &Config) -> Result<Command> {
    prefs.into_iter()
         .filter_map(|var| var.value(config))
         .map(Command::new)
         .next()
         .ok_or_else(|| Error::from(EK::ProgramError(name.to_owned())))
}


/// Run an editor editing the file specified by the supplied path
///
/// A handle to the editor will be returned.
///
pub fn run_editor(config: Config, path: &PathBuf) -> Result<Child> {
    // preference order as specified by the `git var` man page
    let prefs = [
        Var::Environ("GIT_EDITOR"),
        Var::GitConf("core.editor"),
        Var::Environ("VISUAL"),
        Var::Environ("EDITOR"),
        Var::Default("vi") // TODO: make settable at compile time
    ];
    command("editor", &prefs, &config)?
        .arg(path.as_os_str())
        .spawn().chain_err(|| EK::WrappedIOError)
}


/// Assemble and execute a pager command
///
/// Returns the handle to a pager, with a piped stdin, to which the caller may
/// write in order to generate paged output.
///
pub fn pager(config: Config) -> Result<Child> {
    // preference order as specified by the `git var` man page
    let prefs = [
        Var::Environ("GIT_PAGER"),
        Var::GitConf("core.pager"),
        Var::Environ("PAGER"),
        Var::Default("less") // TODO: make settable at compile time
    ];
    command("pager", &prefs, &config)
        .and_then(|mut command| {
            command.stdin(Stdio::piped());
            command.spawn().chain_err(|| EK::WrappedIOError)
        })
}

