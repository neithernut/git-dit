//   git-dit - the distributed issue tracker for git
//   Copyright (C) 2017 Matthias Beyer <mail@beyermatthias.de>
//   Copyright (C) 2017 Julian Ganz <neither@nut.email>
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License version 2 as
//   published by the Free Software Foundation.
//

use git2::Reference;


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

