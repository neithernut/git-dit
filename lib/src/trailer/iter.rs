// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

//! Trailer related iterators
//!
//! This module offers some iterators for convenient processing of `Trailer`s.
//!

use super::{Trailer, TrailerKey, TrailerValue};


/// Iterator assembling trailers from key-value pairs
///
/// This iterator wraps an iterator returning key-value pairs. The pairs
/// returned by the wrapped iterator are assembled to `Trailer`s.
///
pub struct PairsToTrailers<K, I>
    where K: Into<TrailerKey>,
          I: Iterator<Item = (K, TrailerValue)>
{
    inner: I
}

impl<K, I, J> From<J> for PairsToTrailers<K, I>
    where K: Into<TrailerKey>,
          I: Iterator<Item = (K, TrailerValue)>,
          J: IntoIterator<Item = (K, TrailerValue), IntoIter = I>
{
    fn from(iter: J) -> Self {
        PairsToTrailers { inner: iter.into_iter() }
    }
}

impl<K, I> Iterator for PairsToTrailers<K, I>
    where K: Into<TrailerKey>,
          I: Iterator<Item = (K, TrailerValue)>
{
    type Item = Trailer;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(k, v)| Trailer { key: k.into(), value: v })
    }
}


/// Iterator extracting DIT trailers from an iterator over trailers
///
pub struct DitTrailers<I>(I)
    where I: Iterator<Item = Trailer>;

impl<I> From<I> for DitTrailers<I>
    where I: Iterator<Item = Trailer>
{
    fn from(inner: I) -> Self {
        DitTrailers(inner)
    }
}

impl<I> Iterator for DitTrailers<I>
    where I: Iterator<Item = Trailer>
{
    type Item = Trailer;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next() {
                None => return None,
                Some(trailer) => {
                    if trailer.key.0.starts_with("Dit") {
                        return Some(trailer);
                    } else {
                        continue;
                    }

                }
            }
        }
    }

}

