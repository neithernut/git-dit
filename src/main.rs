//   git-dit - the distributed issue tracker for git
//   Copyright (C) 2016 Matthias Beyer <mail@beyermatthias.de>
//   Copyright (C) 2016 Julian Ganz <neither@nut.email>
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License version 2 as
//   published by the Free Software Foundation.
//

#[macro_use] extern crate log;
#[macro_use] extern crate clap;
#[macro_use] extern crate error_chain;
extern crate git2;
extern crate libgitdit;

mod error;

use clap::App;
use git2::Repository;

use error::ErrorKind as EK;
use error::*;


/// Open the DIT repo
///
/// Opens the DIT repo corresponding to the current one honouring the user
/// configuration.
///
fn open_dit_repo() -> Result<Repository> {
    // TODO: access the config and maybe return another repo instead
    Repository::open_from_env().chain_err(|| EK::WrappedGitError)
}


fn main() {
    let yaml    = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();
    println!("Hello, world!");
}
