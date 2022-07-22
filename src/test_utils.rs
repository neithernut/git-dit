// git-dit - the distributed issue tracker for git
// Copyright (C) 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

//! Testing utils
//!
//! This module provides some utility functionality exclusively for testing
//! purposes.
//!

use git2::{self, Repository};
use std::path::PathBuf;
use std::fs;


/// Testing repository
///
/// This type provides a (temporary) testing repository.
///
pub struct TestingRepo {
    repo: Repository,
}

impl TestingRepo {
    /// Create a testing repository
    ///
    /// Create a named testing repository
    ///
    pub fn new(name: &str) -> Self {
        // assemble path
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        assert!(path.is_dir());
        path.push("test");
        path.push(name);

        // create repo
        fs::remove_dir_all(&path).ok();
        let repo = Repository::init_opts(
            path,
            git2::RepositoryInitOptions::new().bare(true).mkdir(true)
        ).expect("Could not open/init repository");
        TestingRepo { repo: repo }
    }

    /// Get a reference of the repo
    ///
    pub fn repo(&mut self) -> &mut Repository {
        &mut self.repo
    }
}

