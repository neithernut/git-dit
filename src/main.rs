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

mod abort;
mod error;
mod programs;
mod util;
mod write;

use clap::App;
use git2::{Commit, Oid, Repository};
use libgitdit::iter::IssueMessagesIter;
use libgitdit::message::{CommitExt, LineIteratorExt};
use libgitdit::repository::RepositoryExt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::Command;

use abort::IteratorExt;
use error::ErrorKind as EK;
use error::*;
use util::{RepositoryUtil, message_from_args};
use write::WriteExt;


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

/// check-message subcommand implementation
///
fn check_message(matches: &clap::ArgMatches) -> i32 {
    let reader: Box<Read> = match matches.value_of("filename") {
        Some(filename)  => Box::from(try_or_1!(File::open(filename))),
        None            => Box::from(io::stdin()),
    };
    BufReader::new(reader).lines()
                          .abort_on_err()
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
        .and_then(|commit| repo.find_tree_init(&commit).chain_err(|| EK::WrappedGitDitError))
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

/// new subcommand implementation
///
fn new_impl(repo: &Repository, matches: &clap::ArgMatches) -> i32 {
    let sig = try_or_1!(repo.signature());

    // get the message, either from the command line argument or an editor
    let message = if let Some(m) = message_from_args(matches) {
        // the message was supplied via the command line
        m.into_iter()
         .chain(try_or_1!(repo.prepare_trailers(matches))
                              .into_iter()
                              .map(|t| t.to_string()))
         .collect()
    } else {
        // we need an editor

        // get the path where we want to edit the message
        let path = repo.commitmsg_edit_path(matches);

        { // write
            let mut file = try_or_1!(File::create(path.as_path()));
            try_or_1!(file.consume_lines(try_or_1!(repo.prepare_trailers(matches))));
            try_or_1!(file.flush());
        }

        try_or_1!(repo.get_commit_msg(path))
    }.into_iter().collect_string();

    // commit the message
    let tree = try_or_1!(repo.empty_tree());
    let parent_refs = Vec::new();
    println!("[dit][new] {}", try_or_1!(repo.create_message(None, &sig, &sig, message.trim(), &tree, &parent_refs)));
    0
}


/// reply subcommand implementation
///
fn reply_impl(repo: &Repository, matches: &clap::ArgMatches) -> i32 {
    let sig = try_or_1!(repo.signature());

    // NOTE: We want to do a lot of stuff early, because we want to report
    //       errors before a user spent time writing a commit message in her
    //       editor. This means that we have a lot of bindings which may not
    //       be neccessary otherwise, resulting in data lying around.

    // the unwrap is safe since `parent` is a required value
    // and get all the info from it that we might need
    let mut parent = try_or_1!(repo.value_to_commit(matches.value_of("parent").unwrap()));

    // extract the subject and tree from the parent
    let subject = parent.summary().map(|s| {
        if s.starts_with("Re: ") {
            s.to_owned()
        } else {
            format!("Re: {}", s)
        }
    });
    let tree = try_or_1!(parent.tree());

    // figure out to what issue we reply
    let issue = try_or_1!(repo.find_tree_init(&parent)).id();

    // get the references specified on the command line
    let references = match matches.values_of("reference")
                               .map(|p| repo.values_to_hashes(p)) {
        Some(hashes) => try_or_1!(hashes),
        _            => Vec::new(),
    };

    // get the message, either from the command line argument or an editor
    let message = if let Some(m) = message_from_args(matches) {
        // the message was supplied via the command line
        if matches.is_present("quote") {
            warn!("Message will only quoted if an editor is used.");
        }

        m.into_iter()
         .chain(try_or_1!(repo.prepare_trailers(matches))
                              .into_iter()
                              .map(|t| t.to_string()))
         .collect()
    } else {
        // we need an editor

        // get the path where we want to edit the message
        let path = repo.commitmsg_edit_path(matches);

        { // write
            let mut file = try_or_1!(File::create(path.as_path()));
            if let Some(s) = subject {
                try_or_1!(write!(&mut file, "{}\n\n", s));
            }

            if matches.is_present("quote") {
                try_or_1!(file.consume_lines(parent.body_lines().quoted()));
                try_or_1!(write!(&mut file, "\n"));
            }

            try_or_1!(file.consume_lines(try_or_1!(repo.prepare_trailers(matches))));
            try_or_1!(file.flush());
        }

        try_or_1!(repo.get_commit_msg(path))
    }.into_iter().collect_string();

    // construct a vector holding all parents
    let parent_refs : Vec<&Commit> = Some(&parent).into_iter().chain(references.iter()).collect();

    // finally, create the message
    try_or_1!(repo.create_message(Some(&issue), &sig, &sig, message.trim(), &tree, &parent_refs));
    0
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
        ("new",     Some(sub_matches)) => new_impl(&repo, sub_matches),
        ("reply",   Some(sub_matches)) => reply_impl(&repo, sub_matches),
        // Unknown subcommands
        (name, sub_matches) => {
            let default = clap::ArgMatches::default();
            handle_unknown_subcommand(name, sub_matches.unwrap_or(&default))
        },
    })
}
