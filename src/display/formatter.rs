//   git-dit - the distributed issue tracker for git
//   Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
//   Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License version 2 as
//   published by the Free Software Foundation.
//

//! Generic formatting utilities
//!

use std::borrow::Borrow;
use std::marker::PhantomData;

/// Formatting tokens
///
/// This type represents generic formatting tokens which may be used for
/// formatting items `I` of some sort into a sequence of lines.
/// Formatting is done via an iterator by expanding `Expandable` tokens to
/// `Text` and `LineEnd` tokens. The latter two will be coposed to lines.
///
pub enum FormattingToken<T, I>
    where T: TokenExpander<Item = I> + Sized
{
    Expandable(T, PhantomData<I>),
    Text(String),
    LineEnd,
}

// NOTE: we could also implement some formatting trait for `FormattingToken`,
//       e.g. `Display` or `ToString`, but the compiler won't let us because of
//       the implementation for `From<T>`.
impl<T, I> From<String> for FormattingToken<T, I>
    where T: TokenExpander<Item = I> + Sized
{
    fn from(text: String) -> Self {
        FormattingToken::Text(text)
    }
}

impl<'a, T, I> From<&'a str> for FormattingToken<T, I>
    where T: TokenExpander<Item = I> + Sized
{
    fn from(text: &str) -> Self {
        FormattingToken::Text(text.to_string())
    }
}

impl<T, I> From<T> for FormattingToken<T, I>
    where T: TokenExpander<Item = I> + Sized
{
    fn from(expander: T) -> Self {
        FormattingToken::Expandable(expander, PhantomData)
    }
}


/// Token expander
///
/// This type is used for expanding tokens.
/// Implementors of concrete formatting facilities will implement this trait for
/// the type of items to be formatted.
///
pub trait TokenExpander: Sized {
    type Item;
    type Error;

    fn expand_token(&self, item: &Self::Item) -> Result<Vec<FormattingToken<Self, Self::Item>>, Self::Error>;
}


/// Helper type for storing eihter a type or a Borrow implementation
///
enum BorrowHelper<T, B>
    where T: Sized,
          B: Borrow<T>
{
    Value(T),
    Borrowed(B),
}

impl<T, B> Borrow<T> for BorrowHelper<T, B>
    where T: Sized,
          B: Borrow<T>
{
    fn borrow(&self) -> &T {
        match self {
            &BorrowHelper::Value(ref val) => &val,
            &BorrowHelper::Borrowed(ref val) => val.borrow(),
        }
    }
}

