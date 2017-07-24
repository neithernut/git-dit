// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

//! Line block categorization
//!
//! When processing messages, we may have to differentiate between blocks of
//! text-lines (paragraphs) and blocks of trailers.
//!
//! This module provides a type for representing the different types of blocks as
//! well as an iterator for extracting the blocks from a sequence of lines.
//!

use message::trailer::Trailer;


/// A block of lines
///
/// We differentiate between paragraphs and blocks of trailers.
///
pub enum Block {
    Text(Vec<String>),
    Trailer(Vec<Trailer>),
}


// TODO: iterator

