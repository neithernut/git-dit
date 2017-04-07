//   git-dit - the distributed issue tracker for git
//   Copyright (C) 2017 Matthias Beyer <mail@beyermatthias.de>
//   Copyright (C) 2017 Julian Ganz <neither@nut.email>
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License version 2 as
//   published by the Free Software Foundation.
//

use git2::{self, Cred, CredentialType, Error};
use std::result::Result as RResult;


/// Get credentials from the user
///
fn get_creds(url: &str, username: Option<&str>, types: CredentialType) -> RResult<Cred, Error> {
    // TODO: implement other authentication methods
    if types.contains(git2::SSH_KEY) {
        if let Some(user) = username {
            return Cred::ssh_key_from_agent(user);
        }
    }
    Cred::default()
}


/// Callbacks to use for fetches and pushes
///
pub fn callbacks() -> git2::RemoteCallbacks<'static> {
    let mut retval = git2::RemoteCallbacks::new();
    retval.credentials(get_creds);
    retval
}

