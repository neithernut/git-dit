#![recursion_limit = "1024"]
// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

//! The git-dit library
//!
//! This library provides low-level functionality for accessing, creating and
//! manipulating "git-dit" issues and messages. It is implemented on top of the
//! `git2` crate. This librarie's documentation primarily provides information
//! about its API and abstract processing of issues and messages.
//!
//!
//! # Issues
//!
//! Issues are stored in git repositories. The issues availible in a repository
//! may be accessed through the `RepositoryExt` extension trait implementation
//! for `git2::Repository`.
//!
//! An issue is primarily a tree of messages, consisting of at least an initial
//! message. An issue also has a "head reference". The head reference lets the
//! maintainer indicate an "upstream status" of the issue, e.g. by pointing to a
//! message which introduces a textual solution or a state.
//!
//! # Messages
//!
//! Like emails, messages are immutable once released to the public. Each
//! message has an author and a creation date. Additionally, a message may
//! contain arbitrary metadata in the form of git-tags.
//!

#[macro_use] extern crate error_chain;
#[macro_use] extern crate lazy_static;
extern crate git2;
extern crate regex;

pub mod error;
pub mod issue;
pub mod iter;
pub mod message;
pub mod remote;
pub mod repository;

mod first_parent_iter;

