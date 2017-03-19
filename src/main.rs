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
mod programs;
mod util;

use clap::App;
use git2::{Commit, Oid, Repository};
use libgitdit::iter::IssueMessagesIter;
use libgitdit::message::{CommitExt, LineIteratorExt};
use libgitdit::repository::RepositoryExt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::process::Command;
use std::process::exit;

use error::ErrorKind as EK;
use error::*;
use util::RepositoryUtil;


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


// Plumbing subcommand implementations

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
                               .map(|p| repo.values_to_hashes(p)) {
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
    repo.value_to_commit(matches.value_of("commit").unwrap())
        .and_then(|commit| repo.find_tree_init(commit).chain_err(|| EK::WrappedGitDitError))
        .map(|commit| {println!("{}", commit.id()); 0})
        .unwrap_or_else(|err| {error!("{}", err); 1})
}


/// get-issue-metadata subcommand implementation
///
fn get_issue_metadata(repo: &Repository, matches: &clap::ArgMatches) -> i32 {
    // note: "head" is always present since it is a required parameter
    let commits = try_or_1!(repo.value_to_commit(matches.value_of("head").unwrap())
                                .map(|commit| IssueMessagesIter::new(commit, repo)));
    for trailer in commits.flat_map(|commit| commit.trailers()) {
        println!("{}", trailer);
    }
    0
}


/// find-tree-init-hash subcommand implementation
///
fn get_issue_tree_init_hashes(repo: &Repository, _: &clap::ArgMatches) -> i32 {
    for hash in try_or_1!(repo.get_all_issue_hashes()) {
        println!("{}", try_or_1!(hash));
    }
    0
}


// Porcelain subcommand implementations

/// show subcommand implementation
fn show(repo: &Repository, sub: &clap::ArgMatches) -> i32 {
    let issue        = sub.value_of("parent").unwrap(); // clap has us here
    let treeinit     = sub.value_of("tree-init-hash").unwrap(); // clap has us here
    let do_abbrev    = sub.is_present("abbrev");
    let only_init    = sub.is_present("initial");
    let show_tree    = sub.is_present("tree");
    let message_tree = sub.is_present("msgtree");
    let verify_gpg   = sub.is_present("verify-gpg");
    let decorate     = sub.is_present("decorate");
    let format       = sub.value_of("format").map(String::from).unwrap_or_else(|| {
        format!("%Cgreen{}\
                %Creset%n\
                %C(yellow)Author:    %an <%ae> (%ai)%Creset%n\
                %C(yellow)Committed: %cn <%ce> (%ci)%Creset%n\
                %n%s%n%n%b%n",
                if do_abbrev { "%h" } else { "%H" })
    });

    if message_tree {
        Oid::from_str(issue)
            .chain_err(|| "Cannot convert issue hash to Oid")
            .and_then(|oid| repo.get_issue_leaves(oid)
                                .chain_err(|| "Cannot find issue leaves"))
            .map(|refs| {
                let def = Command::new("git-log")
                    .arg("--graph")
                    .arg("--topo-order")
                    .arg("--first-parent")
                    .arg(format!("--format=format:{}", format));

                // Builder pattern on steroids
                let command = refs
                    .map(|r| match r {
                        Ok(r) => match r.target() {
                            Some(id) => id,
                            None => {
                                error!("Error: Ref error");
                                exit(42);
                            }
                        },
                        Err(e) => {
                            error!("Error: {:?}", e);
                            exit(42);
                        },
                    })
                    .map(|id| format!("{}", id))
                    .fold(def, |cmd, elem| cmd.arg(elem));

                command
                    .output()
                    .map(|output| {
                        debug!("Exit code of command: {}", output.status);
                        let stdout = String::from_utf8(output.stdout)
                            .map(|outstr| {
                                outstr.lines()
                                    .rev()
                                    .filter(|line| line.contains("|"))
                                    .collect()
                            })
                            .unwrap_or(String::from("UTF8-Error"));

                        println!("{}", stdout);
                        debug!("{:?}", String::from_utf8(output.stderr));

                        0
                    })
                    .unwrap_or_else(|e| {
                        error!("Error: {:?}", e);
                        1
                    })
            })
            .map(|_| 0)
            .unwrap_or_else(|e| {
                error!("Something went wrong: {:?}", e);
                1
            })
    } else {
        if only_init {
            let commit = Oid::from_str(issue)
                .chain_err(|| "Cannot parse commit hash")
                .and_then(|oid| repo.find_commit(oid).chain_err(|| "Cannot find commit"))
                .and_then(|commit| repo.find_tree_init(commit).chain_err(|| "Cannot find tree init"));

            let commit = match commit {
                Ok(c) => c,
                Err(e) => {
                    error!("Error: {:?}", e);
                    return 1;
                }
            };

            let mut cmd = Command::new("git-show");
            if do_abbrev {
                cmd.arg("--abbrev");
            }
            cmd.arg(format!("{}", commit.id())).output(); // or something like this

            0
        } else {
            Oid::from_str(issue)
                .chain_err(|| "Cannot convert issue hash to Oid")
                .and_then(|oid| repo.get_issue_leaves(oid).chain_err(|| "Cannot find issue leaves"))
                .map(|refs| {
                    let mut cdef = Command::new("git-log");

                    if do_abbrev {
                        cdef.arg("--abbrev-commit");
                    }

                    if show_tree {
                        cdef.arg("--graph");
                    }

                    if verify_gpg {
                        cdef.arg("--verify-gpg");
                    }

                    if decorate {
                        cdef.arg("--decorate");
                    }

                    cdef.arg("--topo-order")
                        .arg("--first-parent")
                        .arg(format!("--format=format:{}", format));

                    // Builder pattern on steroids
                    let command = refs
                        .map(|r| match r {
                            Ok(r) => match r.target() {
                                Some(id) => id,
                                None => {
                                    error!("Error: Ref error");
                                    exit(42);
                                }
                            },
                            Err(e) => {
                                error!("Error: {:?}", e);
                                exit(42);
                            },
                        })
                        .map(|id| format!("{}", id))
                        .fold(cdef, |cmd, elem| cmd.arg(elem));

                    command.output() // something like this
                })
                .map(|_| 0)
                .unwrap_or_else(|e| {
                    error!("Something went wrong: {:?}", e);
                    1
                })
        }
    }
}


// Unknown subcommand handler

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

    let repo = match util::open_dit_repo() {
        Ok(r) => r,
        Err(e) => {error!("{}", e); std::process::exit(1)}
    };

    std::process::exit(match matches.subcommand() {
        // Plumbing subcommands
        ("check-message",               Some(sub_matches)) => check_message(sub_matches),
        ("create-message",              Some(sub_matches)) => create_message(&repo, sub_matches),
        ("find-tree-init-hash",         Some(sub_matches)) => find_tree_init_hash(&repo, sub_matches),
        ("get-issue-metadata",          Some(sub_matches)) => get_issue_metadata(&repo, sub_matches),
        ("get-issue-tree-init-hashes",  Some(sub_matches)) => get_issue_tree_init_hashes(&repo, sub_matches),

        // Porcelain subcommands
        ("show",                        Some(sub_matches)) => show(&repo, sub_matches),

        // Unknown subcommands
        (name, sub_matches) => {
            let default = clap::ArgMatches::default();
            handle_unknown_subcommand(name, sub_matches.unwrap_or(&default))
        },
    })
}
