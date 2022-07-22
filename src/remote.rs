// git-dit - the distributed issue tracker for git
// Copyright (C) 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

//! Module providing extension trait for remotes
//!

use git2::Remote;

use issue::Issue;


/// Extension trait for remotes
///
pub trait RemoteExt {
    /// Get the refspec for a specific issue for this remote
    ///
    /// A refspec will only be returned if the remote has a (valid) name.
    ///
    fn issue_refspec(&self, issue: Issue) -> Option<String>;

    /// Get the refspec for all issue for this remote
    ///
    /// A refspec will only be returned if the remote has a (valid) name.
    ///
    fn all_issues_refspec(&self) -> Option<String>;
}

impl<'r> RemoteExt for Remote<'r> {
    fn issue_refspec(&self, issue: Issue) -> Option<String> {
        self.name()
            .map(|name| format!("+refs/dit/{1}/*:refs/remotes/{0}/dit/{1}/*", name, issue.ref_part()))
    }

    fn all_issues_refspec(&self) -> Option<String> {
        self.name()
            .map(|name| format!("+refs/dit/*:refs/remotes/{0}/dit/*", name))
    }
}

