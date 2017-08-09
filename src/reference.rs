//   git-dit - the distributed issue tracker for git
//   Copyright (C) 2017 Matthias Beyer <mail@beyermatthias.de>
//   Copyright (C) 2017 Julian Ganz <neither@nut.email>
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License version 2 as
//   published by the Free Software Foundation.
//

use git2::Reference;
use std::borrow::Borrow;


/// Extension trait for references
///
pub trait ReferrenceExt {
    /// Get the name of the remote associated with the reference
    ///
    /// If this reference is a remote trackign ref, the name of the remote will
    /// be returned. If the reference is not associated with any remote, the
    /// function will return `None`.
    ///
    fn remote(&self) -> Option<&str>;
}

impl<'r> ReferrenceExt for Reference<'r> {
    fn remote(&self) -> Option<&str> {
        if let Some(name) = self.name() {
            let mut name_parts = name.split('/');

            if !is_match!(name_parts.next(), Some("refs")) {
                return None
            }
            if !is_match!(name_parts.next(), Some("remotes")) {
                return None
            }
            name_parts.next()
        } else {
            None
        }
    }
}


/// Expression of priorization of remotes
///
/// Use this type for querying the priority of a remote, represented as a
/// numerical value. A lower numerical value indicates a higher priority.
///
/// The special name `*` in the priority list matches any remote name.
///
pub struct RemotePriorization(Vec<String>);

impl RemotePriorization {
    /// Query the priority for a remote
    ///
    /// If the remote's name is not found, `None` is returned.
    ///
    pub fn priority_for_remote(&self, remote: &str) -> Option<usize> {
        self.0
            .iter()
            .position(|item| *item == remote || *item == "*")
            .map(|pos| pos + 1)
    }

    /// Query the priority of a reference
    ///
    /// This function returns the priority of the remote assiciated with a
    /// reference. If the reference does not appear to be a remote, the highest
    /// possible priority is returned.
    ///
    pub fn priority_for_ref(&self, reference: &Reference) -> Option<usize> {
        match reference.remote() {
            Some(remote) => self.priority_for_remote(remote),
            None => Some(0),
        }
    }
}

impl<'a> From<&'a str> for RemotePriorization {
    fn from(list: &'a str) -> Self {
        RemotePriorization(list.split(',').map(String::from).collect())
    }
}


/// Extension trait for iterators over references
///
pub trait ReferrencesExt<'r> {
    /// Select the reference with the highest priority
    ///
    fn select_ref(self, prios: &RemotePriorization) -> Option<Reference<'r>>;
}

impl<'r, I> ReferrencesExt<'r> for I
    where I: IntoIterator<Item = Reference<'r>>,
{
    fn select_ref(self, prios: &RemotePriorization) -> Option<Reference<'r>> {
        self.into_iter()
            .filter_map(|reference| prios
                .priority_for_ref(reference.borrow())
                .map(|prio| (reference, prio))
            )
            .min_by_key(|item| item.1)
            .map(|item| item.0)
    }
}
