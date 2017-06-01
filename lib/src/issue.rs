// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

//! Issues
//!
//! This module provides the `Issue` type and related functionality.
//!

use git2::{self, Oid};


/// Issue handle
///
/// Instances of this type represent single issues. Issues reside in
/// repositories and are uniquely identified by an id.
///
pub struct Issue<'r> {
    repo: &'r git2::Repository,
    id: Oid,
}

impl<'r> Issue<'r> {
    /// Create a new handle for an issue with a given id
    ///
    pub fn new(repo: &'r git2::Repository, id: Oid) -> Self {
        Issue { repo: repo, id: id }
    }

    /// Get the issue's id
    ///
    pub fn id(&self) -> Oid {
        self.id
    }
}

