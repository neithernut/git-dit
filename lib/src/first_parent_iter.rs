// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
use git2::Commit;


/// Iterator for following the chain of first parents of iterators
///
pub struct FirstParentIter<'repo> {
    current: Option<Commit<'repo>>,
}

impl<'repo> FirstParentIter<'repo> {
    pub fn new(commit: Commit) -> FirstParentIter {
        FirstParentIter { current: Some(commit) }
    }
}

impl<'repo> Iterator for FirstParentIter<'repo> {
    type Item = Commit<'repo>;

    fn next(&mut self) -> Option<Commit<'repo>> {
        self.current.take().map(|c| { self.current = c.parent(0).ok(); c })
    }
}

