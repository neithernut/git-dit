//   git-dit - the distributed issue tracker for git
//   Copyright (C) 2017 Matthias Beyer <mail@beyermatthias.de>
//   Copyright (C) 2017 Julian Ganz <neither@nut.email>
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License version 2 as
//   published by the Free Software Foundation.
//

use std::process::exit;

use error::LoggableError;

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
          E: LoggableError
{
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Abortable::unwrap_or_abort)
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

impl<I, V, IE, OE> IteratorExt<I, V, IE> for Result<I, OE>
    where I: Iterator<Item = Result<V, IE>> + Sized,
          OE: LoggableError
{
    fn abort_on_err(self) -> AbortingIter<I, V, IE> {
        AbortingIter::from(self.unwrap_or_abort())
    }
}


/// Extension trait for convenient abortion in case of errors
///
pub trait Abortable<V>
{
    /// Just like a regular unwrap() except it performs proper logging
    ///
    /// Returns the contained value or aborts the program, logging the error.
    ///
    fn unwrap_or_abort(self) -> V;
}

impl<V, E> Abortable<V> for Result<V, E>
    where E: LoggableError
{
    fn unwrap_or_abort(self) -> V {
        self.unwrap_or_else(|e| {
            e.log();
            exit(1)
        })
    }
}

