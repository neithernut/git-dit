//   git-dit - the distributed issue tracker for git
//   Copyright (C) 2017 Matthias Beyer <mail@beyermatthias.de>
//   Copyright (C) 2017 Julian Ganz <neither@nut.email>
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License version 2 as
//   published by the Free Software Foundation.
//

use std::fmt::Debug;
use std::process::exit;

/// Aborting iterator
///
/// Unwraps items and aborts (calls `exit(1)) if an error value was encountered.
/// It yields the unwrapped values.
///
/// This iterator is intended for uses where it is resonable to abort the
/// program if an error is encountered.
///
pub struct AbortingIter<I, V, E>(I)
    where I: Iterator<Item = Result<V, E>> + Sized;

impl<I, V, E> From<I> for AbortingIter<I, V, E>
    where I: Iterator<Item = Result<V, E>> + Sized
{
    fn from(iter: I) -> Self {
        AbortingIter(iter)
    }
}

impl<I, V, E> Iterator for AbortingIter<I, V, E>
    where I: Iterator<Item = Result<V, E>> + Sized,
          E: Debug
{
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|next|
            next.unwrap_or_else(|e| {
                error!("{:?}", e);
                exit(1)
            })
        )
    }
}


/// Extension trait for convenient creation of `AbortingIter`s
///
pub trait IteratorExt<I, V, E>
    where I: Iterator<Item = Result<V, E>> + Sized
{
    /// Wrap this instance in an aborting iterator
    ///
    fn abort_on_err(self) -> AbortingIter<I, V, E>;
}

impl<I, V, E> IteratorExt<I, V, E> for I
    where I: Iterator<Item = Result<V, E>> + Sized
{
    fn abort_on_err(self) -> AbortingIter<I, V, E> {
        AbortingIter::from(self)
    }
}

