// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

//! Metadata extraction
//!
//! While the `trailer` module offers functionality to extract trailers, this
//! module provides functionality for accumulating trailers and forming sets of
//! metadata.
//!

use std::collections;
use std::hash::BuildHasher;

use trailer::{Trailer, TrailerValue};

/// Policy for accumulating trailers
///
/// These enum values represent accumulation policies for trailers, e.g. how
/// trailer values are accumulated.
///
#[derive(Clone)]
pub enum AccumulationPolicy {
    Latest,
    List,
}


/// Accumulation helper for trailer values
///
/// This type encapsulates the task of accumulating trailers in an appropriate
/// data structure.
///
pub enum ValueAccumulator {
    Latest(Option<TrailerValue>),
    List(Vec<TrailerValue>),
}

impl ValueAccumulator {
    /// Process a new trailer value
    ///
    pub fn process(&mut self, new_value: TrailerValue) {
        match self {
            &mut ValueAccumulator::Latest(ref mut value) => if value.is_none() {
                *value = Some(new_value);
            },
            &mut ValueAccumulator::List(ref mut values)  => values.push(new_value),
        }
    }
}

impl From<AccumulationPolicy> for ValueAccumulator {
    fn from(policy: AccumulationPolicy) -> Self {
        match policy {
            AccumulationPolicy::Latest  => ValueAccumulator::Latest(None),
            AccumulationPolicy::List    => ValueAccumulator::List(Vec::new()),
        }
    }
}

impl IntoIterator for ValueAccumulator {
    type Item = TrailerValue;
    type IntoIter = Box<Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            ValueAccumulator::Latest(value) => Box::new(value.into_iter()),
            ValueAccumulator::List(values)  => Box::new(values.into_iter()),
        }
    }
}

impl Default for ValueAccumulator {
    fn default() -> Self {
        ValueAccumulator::Latest(None)
    }
}


/// Accumulation trait for trailers
///
pub trait Accumulator {
    /// Process a new trailer
    ///
    /// Retrieve the trailer's key. If the key matches a registered trailer,
    /// process its value.
    ///
    fn process(&mut self, trailer: Trailer);

    /// Process all trailers provided by some iterator
    ///
    fn process_all<I>(&mut self, iter: I)
        where I: IntoIterator<Item = Trailer>
    {
        for trailer in iter.into_iter() {
            self.process(trailer);
        }
    }
}


/// Trait for accumulators accumulating multiple values
///
/// # Note
///
/// This trait really is a convenience trait for consolidating mapping
/// containers. It only exists because the standart library doesn't provide
/// any matching traits (the `Index` trait is not an option).
///
pub trait MultiAccumulator {
    /// Get the ValueAccumulator associated with a given string
    ///
    fn get(&self, key: &str) -> Option<&ValueAccumulator>;

    /// Get the ValueAccumulator associated with a given string, mutable
    ///
    fn get_mut(&mut self, key: &str) -> Option<&mut ValueAccumulator>;
}

impl<M> Accumulator for M
    where M: MultiAccumulator
{
    fn process(&mut self, trailer: Trailer) {
        let (key, value) = trailer.into();
        self.get_mut(key.as_ref())
            .map(|ref mut acc| acc.process(value));
    }
}

impl<S> MultiAccumulator for collections::HashMap<String, ValueAccumulator, S>
    where S: BuildHasher
{
    fn get(&self, key: &str) -> Option<&ValueAccumulator> {
        collections::HashMap::get(self, key)
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut ValueAccumulator> {
        collections::HashMap::get_mut(self, key)
    }
}

impl MultiAccumulator for collections::BTreeMap<String, ValueAccumulator> {
    fn get(&self, key: &str) -> Option<&ValueAccumulator> {
        collections::BTreeMap::get(self, key)
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut ValueAccumulator> {
        collections::BTreeMap::get_mut(self, key)
    }
}


/// Accumulator for a single piece of metadata
///
/// Use this accumulator if you only want a single item, e.g. the assignee of
/// an issue.
///
pub struct SingleAccumulator {
    key: String,
    acc: ValueAccumulator,
}

impl SingleAccumulator {
    /// Create a new accumulator for trailers with the key specified
    ///
    pub fn new(key: String, policy: AccumulationPolicy) -> Self {
        SingleAccumulator { key: key, acc: ValueAccumulator::from(policy) }
    }

    /// Convert into an iterator over the accumulated values
    ///
    pub fn into_values(self) -> <ValueAccumulator as IntoIterator>::IntoIter {
        self.acc.into_iter()
    }
}

impl Accumulator for SingleAccumulator {
    fn process(&mut self, trailer: Trailer) {
        let (key, value) = trailer.into();
        if *key.as_ref() == self.key {
            self.acc.process(value);
        }
    }
}

impl Into<(String, ValueAccumulator)> for SingleAccumulator {
    fn into(self) -> (String, ValueAccumulator) {
        (self.key, self.acc)
    }
}

impl Into<ValueAccumulator> for SingleAccumulator {
    fn into(self) -> ValueAccumulator {
        self.acc
    }
}


pub struct SingleKeyTrailerAssemblyIterator<I>
    where I: Iterator<Item = TrailerValue>
{
    key: String,
    inner: I,
}

impl<I> SingleKeyTrailerAssemblyIterator<I>
    where I: Iterator<Item = TrailerValue>
{
    fn new(key: String, inner: I) -> Self {
        SingleKeyTrailerAssemblyIterator { key: key, inner: inner }
    }
}

impl<I> Iterator for SingleKeyTrailerAssemblyIterator<I>
    where I: Iterator<Item = TrailerValue>
{
    type Item = (String, TrailerValue);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|value| (self.key.clone(), value))
    }
}

impl IntoIterator for SingleAccumulator {
    type Item = (String, TrailerValue);
    type IntoIter = SingleKeyTrailerAssemblyIterator<<ValueAccumulator as IntoIterator>::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        SingleKeyTrailerAssemblyIterator::new(self.key, self.acc.into_iter())
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use trailer::{Trailer, TrailerValue};

    // ValueAccumulator tests

    #[test]
    fn accumulate_latest() {
        let mut acc = ValueAccumulator::from(AccumulationPolicy::Latest);
        acc.process(TrailerValue::from_slice("foo-bar"));
        acc.process(TrailerValue::from_slice("baz"));

        let mut values = acc.into_iter();
        assert_eq!(values.next().expect("Could not retrieve value").to_string(), "foo-bar");
        assert_eq!(values.next(), None);
    }

    #[test]
    fn accumulate_list() {
        let mut acc = ValueAccumulator::from(AccumulationPolicy::List);
        acc.process(TrailerValue::from_slice("foo-bar"));
        acc.process(TrailerValue::from_slice("baz"));

        let mut values = acc.into_iter();
        assert_eq!(values.next().expect("Could not retrieve value").to_string(), "foo-bar");
        assert_eq!(values.next().expect("Could not retrieve value").to_string(), "baz");
        assert_eq!(values.next(), None);
    }

    // Accumulator tests

    #[test]
    fn btree_map_accumulator() {
        use std::iter::FromIterator;

        let val_accs = vec![
            (String::from("Assignee"), AccumulationPolicy::Latest),
            (String::from("Foo-bar"), AccumulationPolicy::List),
        ]
            .into_iter()
            .map(|(k, v)| (k, ValueAccumulator::from(v)));
        let mut acc = ::std::collections::BTreeMap::from_iter(val_accs);

        acc.process(Trailer::new("Foo-bar", "baz"));
        acc.process(Trailer::new("Assignee", "Foo Bar <foo.bar@example.com>"));
        acc.process(Trailer::new("Status", "Red alert"));
        acc.process(Trailer::new("Foo-bar", "bam"));
        acc.process(Trailer::new("Assignee", "Mee Seeks <meeseeks@rm.com>"));

        {
            let mut vals = acc
                .remove(&String::from("Assignee"))
                .expect("Could not retrieve value from map")
                .into_iter();
            assert_eq!(
                vals.next().expect("Could not retrieve value from iterator").to_string(),
                "Foo Bar <foo.bar@example.com>"
            );
            assert_eq!(vals.next(), None);
        }

        {
            let mut vals = acc
                .remove(&String::from("Foo-bar"))
                .expect("Could not retrieve value from map")
                .into_iter();
            assert_eq!(
                vals.next().expect("Could not retrieve value from iterator").to_string(),
                "baz"
            );
            assert_eq!(
                vals.next().expect("Could not retrieve value from iterator").to_string(),
                "bam"
            );
            assert_eq!(vals.next(), None);
        }

        assert!(acc.is_empty());
    }

    #[test]
    fn single_accumulator_latest() {
        let mut acc = SingleAccumulator::new(String::from("Foo-bar"), AccumulationPolicy::Latest);
        acc.process(Trailer::new("Foo-bar", "baz"));
        acc.process(Trailer::new("Assignee", "Foo Bar <foo.bar@example.com>"));
        acc.process(Trailer::new("Foo-bar", "bam"));
        acc.process(Trailer::new("Assignee", "Mee Seeks <meeseeks@rm.com>"));

        let mut vals = acc.into_iter();
        assert_eq!(vals.next().expect("Could not retrieve value").1.to_string(), "baz");
        assert_eq!(vals.next(), None);
    }

    #[test]
    fn single_accumulator_list() {
        let mut acc = SingleAccumulator::new(String::from("Foo-bar"), AccumulationPolicy::List);
        acc.process(Trailer::new("Foo-bar", "baz"));
        acc.process(Trailer::new("Assignee", "Foo Bar <foo.bar@example.com>"));
        acc.process(Trailer::new("Foo-bar", "bam"));
        acc.process(Trailer::new("Assignee", "Mee Seeks <meeseeks@rm.com>"));

        let mut vals = acc.into_iter();
        assert_eq!(vals.next().expect("Could not retrieve value").1.to_string(), "baz");
        assert_eq!(vals.next().expect("Could not retrieve value").1.to_string(), "bam");
        assert_eq!(vals.next(), None);
    }
}

