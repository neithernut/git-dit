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


/// Adaption iterator for transforming lines to tokens
///
/// This adapter wraps an iterator over "lines" and transforms each item from
/// the wrapped iterator to a sequence of `Text` and `LineEnd` tokens.
///
pub struct LinesToTokens<I, J, T, K>
    where I: Iterator<Item = J>,
          J: ToString
{
    inner: I,
    eol: bool,
    dummy1: PhantomData<T>,
    dummy2: PhantomData<K>,
}

impl<I, J, T, K> From<I> for LinesToTokens<I, J, T, K>
    where I: Iterator<Item = J>,
          J: ToString,
          T: TokenExpander<Item = K> + Sized
{
    fn from(iter: I) -> Self {
        Self {
            inner: iter,
            eol: false,
            dummy1: PhantomData,
            dummy2: PhantomData,
        }
    }
}

impl<I, J, T, K> Iterator for LinesToTokens<I, J, T, K>
    where I: Iterator<Item = J>,
          J: ToString,
          T: TokenExpander<Item = K> + Sized
{
    type Item = FormattingToken<T, K>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.eol {
            self.eol = false;
            Some(FormattingToken::LineEnd)
        } else {
            let retval = self
                .inner
                .next()
                .map(|line| line.to_string())
                .map(FormattingToken::from);
            // only return an `EndLine` from the next call if we return text in
            // this one
            self.eol = retval.is_some();
            retval
        }
    }
}


/// Convenience iterator for transforming lines to sequences of tokens
///
pub trait LineTokens<I, J>
    where I: Iterator<Item = J>,
          J: ToString
{
    /// Create a LinesToTokens from this iterator
    fn line_tokens<T, K>(self) -> LinesToTokens<I, J, T, K>
        where T: TokenExpander<Item = K> + Sized;
}

impl<A, I, J> LineTokens<I, J> for A
    where A: IntoIterator<Item = J, IntoIter = I>,
          I: Iterator<Item = J>,
          J: ToString
{
    /// Create a LinesToTokens from this iterator
    fn line_tokens<T, K>(self) -> LinesToTokens<I, J, T, K>
        where T: TokenExpander<Item = K> + Sized
    {
        self.into_iter().into()
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


/// Adapter iterator for transforming formatting tokens to lines of text
///
/// This adapter wraps an iterator over formatting tokens, bundled with an item
/// to format.
/// All `Expandable` tokens returned by the wrapped iterator will be expanded
/// recursively.
/// The iterator will construct lines from the resulting "simple" tokens, which
/// will be returned as items.
///
pub struct FormattedLines<I, J, T, K, B>
    where I: Iterator<Item = J>, // Underlying token iterator
          J: Borrow<FormattingToken<T, K>>, // Tokens returned by the iterator
          T: TokenExpander<Item = K>, // The specific token expander
          // K: Item to format
          B: Borrow<K> // Reference of item to format
{
    inner: I,
    item: B,
    tokenstack: Vec<BorrowHelper<FormattingToken<T, K>, J>>,
}

impl<I, J, T, K, B> FormattedLines<I, J, T, K, B>
    where I: Iterator<Item = J>,
          J: Borrow<FormattingToken<T, K>>,
          T: TokenExpander<Item = K>,
          B: Borrow<K>
{
    pub fn new<A>(inner: A, item: B) -> Self
        where A: IntoIterator<Item = J, IntoIter = I>
    {
        FormattedLines {
            inner: inner.into_iter(),
            tokenstack: Vec::new(),
            item: item
        }
    }
}

impl<I, J, T, K, B> Iterator for FormattedLines<I, J, T, K, B>
    where I: Iterator<Item = J>,
          J: Borrow<FormattingToken<T, K>>,
          T: TokenExpander<Item = K>,
          B: Borrow<K>
{
    type Item = Result<String, T::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // Implementers' note: in case you did not guess by now, this adapter
        // is implemented as a stack machine. As described in the user
        // implementation, "simple" tokens are used for composing lines.
        // `Expandable` tokens are expanded to new tokens via the build-in
        // expansion mechanism.
        // Since those tokens may contain line-ends, we have have to buffer
        // them. Since they may contain other `Expandable` tokens, we cannot
        // simply store the returned list but put them on the `tokenstack`.
        // From that stack, we pop an item and process it, which may result in
        // more expanded items on the stack.
        let mut line: Option<String> = None;
        loop {
            // Retrieve a token. Check the stack first before consulting the
            // inner iterator.
            let token = match self
                .tokenstack
                .pop()
                .or_else(|| self.inner.next().map(BorrowHelper::Borrowed))
            {
                Some(val)   => val,
                None        => break,
            };

            match token.borrow() {
                &FormattingToken::Expandable(ref expander, _) => match expander
                    .expand_token(self.item.borrow())
                {
                    Ok(mut tokens) => {
                        // Put the tokens on the stack, in reverse order since we are popping
                        // items rather than iterating over them.
                        tokens.reverse();
                        self.tokenstack
                            .extend(tokens.into_iter().map(BorrowHelper::Value));
                    },
                    Err(err) => return Some(Err(err)),
                },
                &FormattingToken::Text(ref text) => {
                    // Expand the "simple" token.
                    line = Some(line.take().unwrap_or_default() + text);
                },
                &FormattingToken::LineEnd => {
                    // Make sure we return a line one each line end, even if
                    // it's empty.
                    if line.is_none() {
                        line = Some(String::new())
                    }
                    // Return the line
                    break
                },
            }
        }
        line.map(|l| Ok(l))
    }
}


/// Convenience trait for creating a FormattedLines
///
/// Users of formatting facilities will most probably use this trait for
/// performing the formatting.
///
pub trait LineFormatter<I, J, T, K, B>
    where I: Iterator<Item = J>, // Underlying token iterator
          J: Borrow<FormattingToken<T, K>>, // Tokens returned by the iterator
          T: TokenExpander<Item = K>, // The specific token expander
          B: Borrow<K> // Reference of item to format
{
    fn formatted_lines(self, item: B) -> FormattedLines<I, J, T, K, B>;
}

impl<A, I, J, T, K, B> LineFormatter<I, J, T, K, B> for A
    where A: IntoIterator<Item = J, IntoIter = I>,
          I: Iterator<Item = J>,
          J: Borrow<FormattingToken<T, K>>,
          T: TokenExpander<Item = K>,
          B: Borrow<K>
{
    fn formatted_lines(self, item: B) -> FormattedLines<I, J, T, K, B> {
        FormattedLines::new(self, item)
    }
}

