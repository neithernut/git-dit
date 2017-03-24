// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use std::process::Command;
use std::env::var as env_var;
use std::slice;

use git2::Config;

use error::*;
use error::ErrorKind as EK;


/// Command enum for internal representation of different commands
///
enum CommandSpec { Editor, Pager }

impl CommandSpec {
    /// The canonical name of the program
    ///
    pub fn name(&self) -> &str {
        match self {
            &CommandSpec::Editor => "editor",
            &CommandSpec::Pager  => "pager",
        }
    }

    /// Get the name of the git config containing the program name or path
    ///
    pub fn config_name(&self) -> &str {
        match self {
            &CommandSpec::Editor => "core.editor",
            &CommandSpec::Pager  => "core.pager",
        }
    }

    /// Get the names of the environment variables which may contain the program
    /// name
    ///
    pub fn env_var_names(&self) -> slice::Iter<'static, &'static str> {
        match self {
            &CommandSpec::Editor => {static X : &'static [&'static str] = &["GIT_EDITOR", "EDITOR"]; X},
            &CommandSpec::Pager  => {static X : &'static [&'static str] = &["GIT_PAGER",  "PAGER" ]; X},
        }.into_iter()
    }
}


/// Build a `Command` from the config provided
///
/// A bare command will be returned containing
///
fn program(config: Config, command: CommandSpec) -> Result<Command> {
    config
        .get_entry(command.config_name())
        .chain_err(|| EK::ConfigError(command.config_name().to_owned()))?
        .value()
        .map(String::from)
        .or_else(|| command.env_var_names().map(env_var).filter_map(|res| res.ok()).next())
        .map(Command::new)
        .ok_or_else(|| EK::ProgramError(command.name().to_owned()).into())
}


pub fn editor(config: Config) -> Result<Command> {
    program(config, CommandSpec::Editor)
}

pub fn pager(config: Config) -> Result<Command> {
    program(config, CommandSpec::Pager)
}

