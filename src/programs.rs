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

use git2::Config;

use error::*;
use error::ErrorKind as EK;

pub fn editor(config: Config) -> Result<Command> {
    config
        .get_entry("core.editor")
        .chain_err(|| EK::ConfigError("core.editor".to_owned()))?
        .value()
        .map(String::from)
        .or_else(|| env_var("GIT_EDITOR").ok())
        .or_else(|| env_var("EDITOR").ok())
        .map(Command::new)
        .ok_or_else(|| Error::from(EK::ProgramError("editor".to_owned())))
}

