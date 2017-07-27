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
pub struct RemotePriorization(Vec<String>);

impl RemotePriorization {
    /// Query the priority for a remote
    ///
    /// If the remote's name is not found, the lowest possible priority is
    /// returned.
    ///
    pub fn priority_for_remote(&self, remote: &str) -> usize {
        self.0
            .iter()
            .position(|item| *item == remote)
            .map(|pos| pos + 1)
            .unwrap_or(usize::max_value())
    }

    /// Query the priority of a reference
    ///
    /// This function returns the priority of the remote assiciated with a
    /// reference. If the reference does not appear to be a remote, the highest
    /// possible priority is returned.
    ///
    pub fn priority_for_ref(&self, reference: &Reference) -> usize {
        reference
            .remote()
            .map(|name| self.priority_for_remote(name))
            .unwrap_or(0)
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
    fn select_ref(self, prios: RemotePriorization) -> Option<Reference<'r>>;
}

impl<'r, I> ReferrencesExt<'r> for I
    where I: IntoIterator<Item = Reference<'r>>,
{
    fn select_ref(self, prios: RemotePriorization) -> Option<Reference<'r>> {
        self.into_iter()
            .min_by_key(|reference| prios.priority_for_ref(reference.borrow()))
    }
}
