//   git-dit - the distributed issue tracker for git
//   Copyright (C) 2017 Matthias Beyer <mail@beyermatthias.de>
//   Copyright (C) 2017 Julian Ganz <neither@nut.email>
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License version 2 as
//   published by the Free Software Foundation.
//

use git2::{self, Cred};
use std::io::{self, Write};
use std::result::Result as RResult;
use std::str;

use error::LoggableError;


/// Get credentials from the user
///
#[allow(unused)]
fn get_creds(url: &str, username: Option<&str>, types: git2::CredentialType) -> RResult<Cred, git2::Error> {
    // TODO: implement other authentication methods
    if types.contains(git2::SSH_KEY) {
        if let Some(user) = username {
            return Cred::ssh_key_from_agent(user);
        }
    }
    Cred::default()
}


/// Print sideband progress
///
fn print_sideband(data: &[u8]) -> bool {
    let mut stderr = io::stderr();
    // We don't consider output errors critical for sideband data.
    match str::from_utf8(data) {
        Ok(string) => write!(stderr, "remote: {}", string)
                        .and_then(|_| stderr.flush())
                        .ok().unwrap(),
        Err(e) => e.log()
    }
    true
}


/// Print new and deleted messages
///
fn print_tip_updates(refname: &str, old: git2::Oid, new: git2::Oid) -> bool {
    match (old.is_zero(), new.is_zero()) {
        (false, false) => println!("[changed]:  {}", refname),
        (true,  false) => println!("[new]:      {}", refname),
        (false, true ) => println!("[deleted]:  {}", refname),
        _ => {}
    }
    true
}


/// Print info regarding pushed refs
///
fn print_push_ref_updates(refname: &str, failmsg: Option<&str>) -> RResult<(), git2::Error> {
    match failmsg {
        None      => println!("[updated]:  {}", refname),
        Some(msg) => println!("[error]:    {} ({})", refname, msg),
    };
    Ok(())
}


/// Callbacks to use for fetches and pushes
///
pub fn callbacks() -> git2::RemoteCallbacks<'static> {
    let mut retval = git2::RemoteCallbacks::new();
    retval.credentials(get_creds);
    retval.sideband_progress(print_sideband);
    retval.update_tips(print_tip_updates);
    retval.push_update_reference(print_push_ref_updates);
    retval
}

