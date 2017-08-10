// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

//! Private utilities
//!
//! This module provides utilities private to this library.
//!

use std::result::Result as RResult;


/// Trait for pre-accumulating results
pub trait ResultIterExt<I, E> : Sized {
    fn collect_result(self) -> RResult<Vec<I>, E> {
        let mut res = Vec::new();
        self.collect_result_into(&mut res)?;
        Ok(res)
    }

    fn collect_result_into(self, target: &mut Vec<I>) -> RResult<(), E>;
}

impl<I, E, J> ResultIterExt<I, E> for J
    where J: Iterator<Item = RResult<I, E>>
{
    fn collect_result_into(self, target: &mut Vec<I>) -> RResult<(), E> {
        for item in self {
            target.push(item?);
        }
        Ok(())
    }
}

