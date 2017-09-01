// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

//! Line processing utilities
//!


/// Quotation wrapper for iterators over strings
///
/// This iterator wrapps an iterator over lines as string-like items. It
/// returns the lines prefixed with a quotation.
///
#[derive(Debug)]
pub struct Quoted<I, S>(I)
    where I: Iterator<Item = S>,
          S: AsRef<str>;

impl<I, S> From<I> for Quoted<I, S>
    where I: Iterator<Item = S>,
          S: AsRef<str>
{
    fn from(lines: I) -> Self {
        Quoted(lines)
    }
}

impl<I, S> Iterator for Quoted<I, S>
    where I: Iterator<Item = S>,
          S: AsRef<str>
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|l| {
            let line = l.as_ref();
            match line.is_empty() {
                true  => String::from(">"),
                false => format!("> {}", line),
            }
        })
    }
}


/// An Iterator type which iterates over String objects, used to strip
/// whitespace from an iterator over String.
///
pub struct StripWhiteSpaceLeftIter<I, S>(I)
    where I: Iterator<Item = S> + Sized,
          S: AsRef<str>;

impl<I, S> From<I> for StripWhiteSpaceLeftIter<I, S>
    where I: Iterator<Item = S>,
          S: AsRef<str>
{
    fn from(lines: I) -> Self {
        StripWhiteSpaceLeftIter(lines)
    }
}

impl<'a, I, S> Iterator for StripWhiteSpaceLeftIter<I, S>
    where I: Iterator<Item = S> + Sized,
          S: AsRef<str>
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|s| String::from(s.as_ref().trim_left()))
    }
}


/// An Iterator type which iterates over String objects, used to strip
/// whitespace from an iterator over String.
///
pub struct StripWhiteSpaceRightIter<I, S>(I)
    where I: Iterator<Item = S> + Sized,
          S: AsRef<str>;

impl<I, S> From<I> for StripWhiteSpaceRightIter<I, S>
    where I: Iterator<Item = S>,
          S: AsRef<str>
{
    fn from(lines: I) -> Self {
        StripWhiteSpaceRightIter(lines)
    }
}

impl<'a, I, S> Iterator for StripWhiteSpaceRightIter<I, S>
    where I: Iterator<Item = S> + Sized,
          S: AsRef<str>
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|s| String::from(s.as_ref().trim_right()))
    }
}


/// An iterator type for removing comment lines
///
/// Given an iterator over the lines of a message in the form of strings, this
/// iterator will remove all lines starting with a "#".
///
pub struct WithoutCommentsIter<I, S>(I)
    where I: Iterator<Item = S> + Sized,
          S: AsRef<str>;

impl<I, S> From<I> for WithoutCommentsIter<I, S>
    where I: Iterator<Item = S>,
          S: AsRef<str>
{
    fn from(lines: I) -> Self {
        WithoutCommentsIter(lines)
    }
}

impl<I, S> Iterator for WithoutCommentsIter<I, S>
    where I: Iterator<Item = S> + Sized,
          S: AsRef<str>
{
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.0.next() {
            // we do not trim whitespace here, because of code blocks in the message which might
            // have a "#" at the beginning
            if !next.as_ref().starts_with("#") {
                return Some(next)
            }
        }
        None
    }
}


/// Iterator adapter for removing blank lines from the end of a sequence
///
/// This iterator wraps an iterator over lines and forwards all lines from the
/// wrapped iterator except trailing blank lines.
///
pub struct TrailingBlankTrimmer<I, S>
    where I: Iterator<Item = S> + Sized,
          S: AsRef<str> + Default
{
    inner: ::std::iter::Peekable<I>,
    // counter for blanks
    blanks: usize,
}

impl<I, J, S> From<I> for TrailingBlankTrimmer<J, S>
    where I: IntoIterator<Item = S, IntoIter = J>,
          J: Iterator<Item = S>,
          S: AsRef<str> + Default
{
    fn from(lines: I) -> Self {
        TrailingBlankTrimmer { inner: lines.into_iter().peekable(), blanks: 0 }
    }
}

impl<I, S> Iterator for TrailingBlankTrimmer<I, S>
    where I: Iterator<Item = S>,
          S: AsRef<str> + Default
{
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        if self.blanks > 0 {
            // If we recorded any blank lines, we return them.
            self.blanks = self.blanks - 1;
            return Some(S::default());
        }

        // Record and consume blank lines
        while self
            .inner
            .peek()
            .map(AsRef::as_ref)
            .map(str::is_empty)
            .unwrap_or_else(|| {
                // We reached the end of input and don't want to return any more
                // lines.
                self.blanks = 0;
                false
            })
        {
            self.blanks = self.blanks + 1;
            self.inner.next();
        }

        if self.blanks > 0 {
            // If we recorded any blank lines, we return them.
            self.blanks = self.blanks - 1;
            Some(S::default())
        } else {
            self.inner.next()
        }
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_lines() {
        let mut lines = Quoted::from(vec!["foo", "bar", "", "baz"].into_iter());
        assert_eq!(lines.next().expect("Premature end of input"), "> foo");
        assert_eq!(lines.next().expect("Premature end of input"), "> bar");
        assert_eq!(lines.next().expect("Premature end of input"), ">");
        assert_eq!(lines.next().expect("Premature end of input"), "> baz");
        assert!(!lines.next().is_some());
    }

    #[test]
    fn left_stripped_lines() {
        let mut lines = StripWhiteSpaceLeftIter::from(vec!["foo  ", "  bar", "  ", ""].into_iter());
        assert_eq!(lines.next().expect("Premature end of input"), "foo  ");
        assert_eq!(lines.next().expect("Premature end of input"), "bar");
        assert_eq!(lines.next().expect("Premature end of input"), "");
        assert_eq!(lines.next().expect("Premature end of input"), "");
        assert!(!lines.next().is_some());
    }

    #[test]
    fn right_stripped_lines() {
        let mut lines = StripWhiteSpaceRightIter::from(vec!["foo  ", "  bar", "  ", ""].into_iter());
        assert_eq!(lines.next().expect("Premature end of input"), "foo");
        assert_eq!(lines.next().expect("Premature end of input"), "  bar");
        assert_eq!(lines.next().expect("Premature end of input"), "");
        assert_eq!(lines.next().expect("Premature end of input"), "");
        assert!(!lines.next().is_some());
    }

    #[test]
    fn lines_without_comments() {
        let mut lines = WithoutCommentsIter::from(vec!["foo", "# bar", "#", ""].into_iter());
        assert_eq!(lines.next().expect("Premature end of input"), "foo");
        assert_eq!(lines.next().expect("Premature end of input"), "");
        assert!(!lines.next().is_some());
    }

    #[test]
    fn trailing_blank_trimmer() {
        let mut lines = TrailingBlankTrimmer::from(vec!["", "foo", "bar", "", "baz", "", ""]);
        assert_eq!(lines.next().expect("Premature end of input"), "");
        assert_eq!(lines.next().expect("Premature end of input"), "foo");
        assert_eq!(lines.next().expect("Premature end of input"), "bar");
        assert_eq!(lines.next().expect("Premature end of input"), "");
        assert_eq!(lines.next().expect("Premature end of input"), "baz");
        assert!(!lines.next().is_some());
    }
}
