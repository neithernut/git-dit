//   git-dit - the distributed issue tracker for git
//   Copyright (C) 2016 Matthias Beyer <mail@beyermatthias.de>
//   Copyright (C) 2016 Julian Ganz <neither@nut.email>
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License version 2 as
//   published by the Free Software Foundation.
//

#[macro_use] extern crate clap;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate is_match;
#[macro_use] extern crate log;
extern crate chrono;
extern crate git2;
extern crate libgitdit;

mod abort;
mod callbacks;
mod error;
mod logger;
mod msgtree;
mod programs;
mod util;
mod write;

use chrono::{FixedOffset, TimeZone};
use clap::App;
use git2::{Commit, PushOptions, Repository};
use libgitdit::message::{LineIteratorExt, Trailer};
use libgitdit::{Message, RemoteExt, RepositoryExt};
use log::LogLevel;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::Command;
use std::str::FromStr;

use abort::{Abortable, IteratorExt};
use error::*;
use error::ErrorKind as EK;
use msgtree::{IntoTreeGraph, TreeGraphElem, TreeGraphElemLine};
use util::{RepositoryUtil, message_from_args};
use write::WriteExt;


// Plumbing subcommand implementations

/// check-message subcommand implementation
///
fn check_message(matches: &clap::ArgMatches) {
    let reader: Box<Read> = match matches.value_of("filename") {
        Some(filename)  => Box::from(File::open(filename).unwrap_or_abort()),
        None            => Box::from(io::stdin()),
    };
    BufReader::new(reader).lines()
                          .abort_on_err()
                          .skip_while(|l| l.is_empty())
                          .stripped()
                          .check_message_format()
                          .unwrap_or_abort();
}


/// create-message subcommand implementation
///
fn create_message(repo: &Repository, matches: &clap::ArgMatches) {
    let issue = match matches.value_of("issue") {
        Some(i) => Some(repo.value_to_issue(i).unwrap_or_abort()),
        None    => None,
    };
    let sig = repo.signature().unwrap_or_abort();

    // Note: The list of parents must live long enough to back the references we
    //       supply to `libgitdit::repository::RepositoryExt::create_message()`.
    let parents = matches.values_of("parents")
                         .map(|p| repo.values_to_hashes(p))
                         .map(Abortable::unwrap_or_abort)
                         .unwrap_or_default();
    let parent_refs = parents.iter().map(|command| command);

    // use the first parent's tree if availible
    let tree = match parents.first() {
        Some(commit) => commit.tree().unwrap_or_abort(),
        _            => repo.empty_tree().unwrap_or_abort(),
    };

    // read all from stdin
    let mut message = String::new();
    io::stdin().read_to_string(&mut message).unwrap_or_abort();
    let id = match issue {
        Some(i) => i.add_message(&sig, &sig, message, &tree, parent_refs)
                    .unwrap_or_abort()
                    .id(),
        None => repo.create_issue(&sig, &sig, message, &tree, parent_refs)
                    .unwrap_or_abort()
                    .id(),
    };

    println!("{}", id);
}


/// find-tree-init-hash subcommand implementation
///
fn find_tree_init_hash(repo: &Repository, matches: &clap::ArgMatches) {
    // note: commit is always present since it is a required parameter
    let commit = repo
        .value_to_commit(matches.value_of("commit").unwrap())
        .and_then(|commit| {
            repo.issue_with_message(&commit)
                .chain_err(|| EK::WrappedGitDitError)
        })
        .unwrap_or_abort();

     println!("{}", commit.id());
}


/// get-issue-metadata subcommand implementation
///
fn get_issue_metadata(repo: &Repository, matches: &clap::ArgMatches) {
    // note: "head" is always present since it is a required parameter
    let head = repo
        .value_to_commit(matches.value_of("head").unwrap())
        .unwrap_or_abort();
    let commits = repo.issue_messages_iter(head).abort_on_err();
    for trailer in commits.flat_map(|commit| commit.trailers()) {
        println!("{}", trailer);
    }
}


/// find-tree-init-hash subcommand implementation
///
fn get_issue_tree_init_hashes(repo: &Repository, _: &clap::ArgMatches) {
    io::stdout().consume_lines(repo.issues().abort_on_err()).unwrap_or_abort();
}


// Porcelain subcommand implementations

/// fetch subcommand implementation
///
fn fetch_impl(repo: &Repository, matches: &clap::ArgMatches) {
    // note: "remote" is always present since it is a required parameter
    let mut remote = repo
        .find_remote(matches.value_of("remote").unwrap())
        .unwrap_or_abort();

    // accumulate the refspecs to fetch
    let refspecs : Vec<String> = if let Some(issues) = matches.values_of("issue") {
        // fetch a specific list of issues
        let iter = issues.map(|issue| repo.value_to_issue(issue)).abort_on_err();
        if matches.is_present("known") {
            iter.chain(repo.issues().abort_on_err())
                .filter_map(|issue| remote.issue_refspec(issue))
                .collect()
        } else {
            iter.filter_map(|issue| remote.issue_refspec(issue))
                .collect()
        }
    } else {
        vec![remote.all_issues_refspec().unwrap()]
    };

    // set the options for the fetch
    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.prune(if matches.is_present("prune") {
        git2::FetchPrune::On
    } else {
        git2::FetchPrune::Unspecified
    });
    fetch_options.remote_callbacks(callbacks::callbacks());

    let refspec_refs : Vec<&str> = refspecs.iter().map(String::as_str).collect();
    remote.fetch(refspec_refs.as_ref(), Some(&mut fetch_options), None)
          .unwrap_or_abort();
}


/// list subcommand implementation
///
fn list_impl(repo: &Repository, matches: &clap::ArgMatches) {
    // get initial commits
    let mut commits : Vec<Commit> = repo.issues()
        .abort_on_err()
        .map(|issue| repo.find_commit(issue.id()))
        .abort_on_err()
        .collect();

    // descending order, maybe limited to some number specified by the user
    commits.sort_by(|a, b| b.time().cmp(&a.time()));
    if let Some(number) = matches.value_of("n") {
        // TODO: better error reporting?
        commits.truncate(str::parse(number).unwrap_or_abort());
    }

    let id_len = repo.abbreviation_length(matches).unwrap_or_abort();

    // spawn a pager
    let mut pager = programs::pager(repo.config().unwrap_or_abort())
        .unwrap_or_abort();

    {
        let mut stream = pager.stdin.as_mut().unwrap();
        let long = matches.is_present("long");
        for mut commit in commits {
            let id = commit.id();
            let time = {
                let gtime = commit.time();
                FixedOffset::east(gtime.offset_minutes()*60).timestamp(gtime.seconds(), 0)
            };
            if long {
                write!(stream, "Issue:  {}\nAuthor: {}\nDate:   {}\n\n", id, commit.author(), time.to_rfc3339())
                    .unwrap_or_abort();
                stream.consume_lines(commit.message_lines()).unwrap_or_abort();
                write!(stream, "\n\n").unwrap_or_abort();
            } else {
                writeln!(stream, "{0:.1$} ({2}) {3}", id, id_len, time.format("%c"), commit.summary().unwrap_or(""))
                    .unwrap_or_abort();
            }
        }
    }

    // don't trash the shell by exitting with a child still printing to it
    let result = pager.wait().unwrap_or_abort();
    if !result.success() {
        std::process::exit(result.code().unwrap_or(1));
    }
}


/// new subcommand implementation
///
fn new_impl(repo: &Repository, matches: &clap::ArgMatches) {
    let sig = repo.signature().unwrap_or_abort();

    // get the message, either from the command line argument or an editor
    let message = if let Some(m) = message_from_args(matches) {
        // the message was supplied via the command line
        m.into_iter()
         .chain(repo.prepare_trailers(matches)
                    .unwrap_or_abort()
                    .into_iter()
                    .map(|t| t.to_string()))
         .collect()
    } else {
        // we need an editor

        // get the path where we want to edit the message
        let path = repo.commitmsg_edit_path(matches);

        { // write
            let mut file = File::create(path.as_path()).unwrap_or_abort();
            file.consume_lines(repo.prepare_trailers(matches).unwrap_or_abort()).unwrap_or_abort();
            file.flush().unwrap_or_abort();
        }

        repo.get_commit_msg(path).unwrap_or_abort()
    }.into_iter().collect_string();

    // commit the message
    let tree = repo.empty_tree().unwrap_or_abort();
    let id = repo
        .create_issue(&sig, &sig, message.trim(), &tree, Vec::new())
        .unwrap_or_abort();
    println!("[dit][new] {}", id);
}


/// push subcommand implementation
///
fn push_impl(repo: &Repository, matches: &clap::ArgMatches) {
    // note: "remote" is always present since it is a required parameter
    let mut remote = repo.find_remote(matches.value_of("remote").unwrap()).unwrap_or_abort();

    // accumulate the refspecs to push
    let refspecs : Vec<String> = if let Some(issues) = matches.values_of("issue") {
        // push a specific list of issues
        issues.map(|issue| repo.value_to_issue(issue))
              .abort_on_err()
              .map(|issue| issue.local_refs())
              .abort_on_err()
              .flat_map(git2::References::names)
              .abort_on_err()
              .map(String::from)
              .collect()
    } else {
        repo.issues_with_prefix("refs")
            .abort_on_err()
            .map(|issue| issue.local_refs())
            .abort_on_err()
            .flat_map(git2::References::names)
            .abort_on_err()
            .map(String::from)
            .collect()
    };

    // set the options for the push
    let mut fetch_options = PushOptions::new();
    fetch_options.remote_callbacks(callbacks::callbacks());

    let refspec_refs : Vec<&str> = refspecs.iter().map(String::as_str).collect();
    remote.push(refspec_refs.as_ref(), Some(&mut fetch_options))
          .unwrap_or_abort();
}


/// reply subcommand implementation
///
fn reply_impl(repo: &Repository, matches: &clap::ArgMatches) {
    let sig = repo.signature().unwrap_or_abort();

    // NOTE: We want to do a lot of stuff early, because we want to report
    //       errors before a user spent time writing a commit message in her
    //       editor. This means that we have a lot of bindings which may not
    //       be neccessary otherwise, resulting in data lying around.

    // the unwrap is safe since `parent` is a required value
    // and get all the info from it that we might need
    let mut parent = repo
        .value_to_commit(matches.value_of("parent").unwrap())
        .unwrap_or_abort();

    // extract the subject and tree from the parent
    let subject = parent.reply_subject();
    let tree = parent.tree().unwrap_or_abort();

    // figure out to what issue we reply
    let issue = repo.issue_with_message(&parent).unwrap_or_abort();

    // get the references specified on the command line
    let references = repo.cli_references(matches).unwrap_or_abort();

    // get the message, either from the command line argument or an editor
    let message = if let Some(m) = message_from_args(matches) {
        // the message was supplied via the command line
        if matches.is_present("quote") {
            warn!("Message will only quoted if an editor is used.");
        }

        m.into_iter()
         .chain(repo.prepare_trailers(matches)
                    .unwrap_or_abort()
                    .into_iter()
                    .map(|t| t.to_string()))
         .collect()
    } else {
        // we need an editor

        // get the path where we want to edit the message
        let path = repo.commitmsg_edit_path(matches);

        { // write
            let mut file = File::create(path.as_path()).unwrap_or_abort();
            if let Some(s) = subject {
                write!(&mut file, "{}\n\n", s).unwrap_or_abort();
            }

            if matches.is_present("quote") {
                file.consume_lines(parent.body_lines().quoted())
                    .unwrap_or_abort();
                write!(&mut file, "\n").unwrap_or_abort();
            }

            file.consume_lines(repo.prepare_trailers(matches).unwrap_or_abort())
                .unwrap_or_abort();
            file.flush().unwrap_or_abort();
        }

        repo.get_commit_msg(path).unwrap_or_abort()
    }.into_iter().collect_string();

    // construct a vector holding all parents
    let parent_refs = Some(&parent).into_iter().chain(references.iter());

    // finally, create the message
    issue.add_message(&sig, &sig, message.trim(), &tree, parent_refs)
         .unwrap_or_abort();
}

/// show subcommand implementation
///
fn show_impl(repo: &Repository, matches: &clap::ArgMatches) {
    let id_len = repo.abbreviation_length(matches).unwrap_or_abort();

    // translate commit to lines representing the commit
    let commit_lines = |mut commit: Commit| -> Vec<String> {
        // the function is this ugly to comply to the old bash interface
        if matches.is_present("msgtree") {
            // With the "tree" option, we only display subjects in a short
            // format

            // NOTE: the commit is borrowed mutable in order to get the subject
            let subject = commit.summary().unwrap_or("").to_owned();
            vec![format!("{0:.1$} {2}: {3}", commit.id(), id_len, commit.author(), subject)]
        } else {
            let mut id = commit.id().to_string();
            id.truncate(id_len);
            // Regular "long" format
            vec![
                id,
                commit.author().to_string(),
                String::new()
            ].into_iter()
                .chain(commit.message_lines())
                .chain(vec![String::new()].into_iter())
                .collect()
        }
    };

    // first, get us an iterator over all the commits
    let issue = repo.cli_issue(matches).unwrap_or_abort();
    let mut commits : Vec<(TreeGraphElemLine, Commit)> =
        if matches.is_present("initial") {
            vec![(
                TreeGraphElemLine::empty(),
                issue.initial_message().unwrap_or_abort()
            )]
        } else {
            issue
                .message_revwalk()
                .abort_on_err()
                .map(|oid| repo.find_commit(oid))
                .abort_on_err()
                .into_tree_graph()
                .collect()
        };

    // Decide on the order in which the messages will be printed.
    if matches.is_present("tree") {
        // We want the commits in chronological order
        commits.reverse();
        for commit in commits.iter_mut() {
            commit.0.reverse_marks();
        }
    };

    // Transform the simple graph element line into an iterator over lines to
    // print via multiple steps.
    let graph = commits
        .into_iter()
        // expand the graph element lines for each message
        .map(|commit| {
            let mut elems = commit.0;
            // offset the commit from the graph elements by adding an empty one
            // in between
            elems.append(TreeGraphElem::Empty);
            (elems.commit_iterator(), commit.1)
        })
        // expand the message to a series of lines
        .flat_map(|commit| commit.0.zip(commit_lines(commit.1)))
        // combine each line of graph elements and message
        .map(|line| format!("{} {}", line.0, line.1));

    // spawn a pager and write the graph
    let mut pager = programs::pager(repo.config().unwrap_or_abort())
        .unwrap_or_abort();
    pager.stdin.as_mut().unwrap().consume_lines(graph).unwrap_or_abort();

    // don't trash the shell by exitting with a child still printing to it
    let result = pager.wait().unwrap_or_abort();
    if !result.success() {
        std::process::exit(result.code().unwrap_or(1));
    }
}

/// tag subcommand implementation
///
fn tag_impl(repo: &Repository, matches: &clap::ArgMatches) {
    // get the head for the issue to tag
    let mut issue_head = repo
        .cli_issue(matches)
        .unwrap_or_abort()
        .find_local_head()
        .unwrap_or_abort();
    let mut head_commit = issue_head
        .peel(git2::ObjectType::Commit)
        .unwrap_or_abort()
        .into_commit()
        .ok()
        .unwrap();

    if matches.is_present("list") {
        // we only list the metadata
        let trailers = repo.issue_messages_iter(head_commit)
            .abort_on_err()
            .flat_map(|c| c.trailers());
        io::stdout().consume_lines(trailers).unwrap_or_abort();
        return;
    }

    // we produce a commit with status and references

    // get references and trailers for the new commit
    let references = repo.cli_references(matches).unwrap_or_abort();
    let trailers : Vec<Trailer> = matches.values_of("set-status")
                                         .into_iter()
                                         .flat_map(|values| values)
                                         .map(Trailer::from_str)
                                         .abort_on_err()
                                         .collect();
    if references.is_empty() && trailers.is_empty() {
        warn!("No commit was created because no reference or tags were supplied.");
        return;
    }

    // construct the message
    let sig = repo.signature().unwrap_or_abort();
    let message = [head_commit.reply_subject().unwrap_or_default(), String::new()]
        .to_vec()
        .into_iter()
        .chain(trailers.into_iter().map(|t| t.to_string()))
        .collect_string();
    let tree = repo.empty_tree().unwrap_or_abort();
    let parent_refs : Vec<&Commit> = Some(&head_commit).into_iter().chain(references.iter()).collect();
    let new = repo
        .commit(None, &sig, &sig, message.trim(), &tree, &parent_refs)
        .unwrap_or_abort();

    // update the head reference
    issue_head.set_target(new, "Issue head updated by git-dit-tag")
              .unwrap_or_abort();
}


// Unknown subcommand handler

/// Handle unknown subcommands
///
/// Try to invoke an executable matching the name of the subcommand.
///
fn handle_unknown_subcommand(name: &str, matches: &clap::ArgMatches) {
    // prepare the command to be invoked
    let mut command = Command::new(format!("git-dit-{}", name));
    if let Some(values) = matches.values_of("") {
         values.fold(&mut command, |c, arg| c.arg(arg));
    }

    // run the command
    let result = command
        .spawn()
        .and_then(|mut child| child.wait())
        .unwrap_or_abort();
    if !result.success() {
        std::process::exit(result.code().unwrap_or(1));
    }
}


fn main() {
    let yaml    = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Err(err) = logger::Logger::init(LogLevel::Warn) {
        writeln!(io::stderr(), "Could not initialize logger: {}", err).ok();
    }

    let repo = util::open_dit_repo().unwrap_or_abort();

    match matches.subcommand() {
        // Plumbing subcommands
        ("check-message",               Some(sub_matches)) => check_message(sub_matches),
        ("create-message",              Some(sub_matches)) => create_message(&repo, sub_matches),
        ("find-tree-init-hash",         Some(sub_matches)) => find_tree_init_hash(&repo, sub_matches),
        ("get-issue-metadata",          Some(sub_matches)) => get_issue_metadata(&repo, sub_matches),
        ("get-issue-tree-init-hashes",  Some(sub_matches)) => get_issue_tree_init_hashes(&repo, sub_matches),
        // Porcelain subcommands
        ("fetch",   Some(sub_matches)) => fetch_impl(&repo, sub_matches),
        ("list",    Some(sub_matches)) => list_impl(&repo, sub_matches),
        ("new",     Some(sub_matches)) => new_impl(&repo, sub_matches),
        ("push",    Some(sub_matches)) => push_impl(&repo, sub_matches),
        ("reply",   Some(sub_matches)) => reply_impl(&repo, sub_matches),
        ("show",    Some(sub_matches)) => show_impl(&repo, sub_matches),
        ("tag",     Some(sub_matches)) => tag_impl(&repo, sub_matches),
        // Unknown subcommands
        ("", _) => {
            writeln!(io::stderr(), "{}", matches.usage()).ok();
            std::process::exit(1);
        },
        (name, sub_matches) => {
            let default = clap::ArgMatches::default();
            handle_unknown_subcommand(name, sub_matches.unwrap_or(&default))
        },
    }
}
