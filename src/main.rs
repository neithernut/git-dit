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
mod editor;

use clap::{App, Values};
use git2::{Commit, Oid, Repository};
use libgitdit::message::LineIteratorExt;
use libgitdit::repository::RepositoryExt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::process::Command;

use error::ErrorKind as EK;
use error::*;


/// Convenience macro for early returns in subcommands
///
/// This macro is similar to the `try!` macro. It evaluates the expression
/// passed. If the result the expression yields is ok, it will be unwrapped.
/// Else the error will be printed using the `error!` macro and abort the
/// function, returning `1`.
///
/// Note: using this macro in clauses usually doesn't make sense, since it
///       aborts the function by returning a numeric value.
///
macro_rules! try_or_1 {
    ($expr: expr) => {
        match $expr {
            Ok(v) => v,
            Err(e)   => {error!("{:?}", e); return 1},
        }
    };
}


/// Open the DIT repo
///
/// Opens the DIT repo corresponding to the current one honouring the user
/// configuration.
///
fn open_dit_repo() -> Result<Repository> {
    // TODO: access the config and maybe return another repo instead
    Repository::open_from_env().chain_err(|| EK::WrappedGitError)
}


/// Get a vector of commits from values
///
/// This function transforms values to a vector.
///
fn values_to_hashes<'repo>(repo: &'repo Repository, values: Values) -> Result<Vec<Commit<'repo>>> {
    let mut retval = Vec::new();
    for commit in values.map(|string| repo.revparse_single(string))
                        .map(|oid| repo.find_commit(try!(oid).id())) {
        retval.push(try!(commit));
    }
    Ok(retval)
}


/// check-message subcommand implementation
///
fn check_message(matches: &clap::ArgMatches) -> i32 {
    let reader: Box<Read> = match matches.value_of("filename") {
        Some(filename)  => Box::from(try_or_1!(File::open(filename))),
        None            => Box::from(io::stdin()),
    };
    BufReader::new(reader).lines()
                          .map(|l| l.unwrap_or_else(|err| {
                              // abort on IO errors
                              error!("{:?}", err);
                              std::process::exit(1);
                          }))
                          .skip_while(|l| l.is_empty())
                          .stripped()
                          .check_message_format()
                          .map(|_| 0)
                          .unwrap_or_else(|err| {error!("{:?}", err); 1})
}


/// create-message subcommand implementation
///
fn create_message(repo: &Repository, matches: &clap::ArgMatches) -> i32 {
    let issue = match matches.value_of("issue") {
        Some(i) => Some(try_or_1!(Oid::from_str(i))),
        None    => None,
    };
    let sig = try_or_1!(repo.signature());

    // Note: The list of parents must live long enough to back the references we
    //       supply to `libgitdit::repository::RepositoryExt::create_message()`.
    let parents = match matches.values_of("parents")
                               .map(|p| values_to_hashes(repo, p)) {
        Some(hashes) => try_or_1!(hashes),
        _            => Vec::new(),
    };
    let parent_refs : Vec<&Commit> = parents.iter().map(|command| command).collect();

    // use the first parent's tree if availible
    let tree = match parents.first() {
        Some(commit) => try_or_1!(commit.tree()),
        _            => try_or_1!(repo.empty_tree()),
    };

    // read all from stdin
    let mut message = String::new();
    try_or_1!(io::stdin().read_to_string(&mut message));

    println!("{}", try_or_1!(repo.create_message(issue.as_ref(), &sig, &sig, &message, &tree, &parent_refs)));
    0
}


/// find-tree-init-hash subcommand implementation
///
fn find_tree_init_hash(repo: &Repository, matches: &clap::ArgMatches) -> i32 {
    // note: commit is always present since it is a required parameter
    repo.revparse_single(matches.value_of("commit").unwrap())
        .and_then(|obj| repo.find_commit(obj.id()))
        .chain_err(|| EK::WrappedGitError)
        .and_then(|commit| repo.find_tree_init(commit).chain_err(|| EK::WrappedGitDitError))
        .map(|commit| {println!("{}", commit.id()); 0})
        .unwrap_or_else(|err| {error!("{}", err); 1})
}


/// find-tree-init-hash subcommand implementation
///
fn get_issue_tree_init_hashes(repo: &Repository, _: &clap::ArgMatches) -> i32 {
    for hash in try_or_1!(repo.get_all_issue_hashes()) {
        println!("{}", try_or_1!(hash));
    }
    0
}


/// Handle unknown subcommands
///
/// Try to invoke an executable matching the name of the subcommand.
///
fn handle_unknown_subcommand(name: &str, matches: &clap::ArgMatches) -> i32 {
    // prepare the command to be invoked
    let mut command = Command::new(format!("git-dit-{}", name));
    if let Some(values) = matches.values_of("") {
         values.fold(&mut command, |c, arg| c.arg(arg));
    }

    // run the command
    command.spawn()
           .and_then(|mut child| child.wait())
           .map(|result| result.code().unwrap_or(1))
           .unwrap_or_else(|err| {error!("{}", err); 1})
}


fn main() {
    let yaml    = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    let repo = match open_dit_repo() {
        Ok(r) => r,
        Err(e) => {error!("{}", e); std::process::exit(1)}
    };

    std::process::exit(match matches.subcommand() {
        ("check-message",               Some(sub_matches)) => check_message(sub_matches),
        ("create-message",              Some(sub_matches)) => create_message(&repo, sub_matches),
        ("find-tree-init-hash",         Some(sub_matches)) => find_tree_init_hash(&repo, sub_matches),
        ("get-issue-tree-init-hashes",  Some(sub_matches)) => get_issue_tree_init_hashes(&repo, sub_matches),
        (name, sub_matches) => {
            let default = clap::ArgMatches::default();
            handle_unknown_subcommand(name, sub_matches.unwrap_or(&default))
        },
    })
}
