// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

//! Trailer based filtering
//!
//! This module provides utilities for trailer based filtering of issues.
//!
//! # Performance note
//!
//! These utilities are purely designed for filtering.
//! For searching issues in repositories of huge projects, an index should be
//! more performant.
//!

use std::borrow::Borrow;

use trailer::TrailerValue;
use trailer::accumulation::ValueAccumulator;
use trailer::spec::TrailerSpec;


/// Type for matching TrailerValues
///
pub enum ValueMatcher {
    Any,
    Equals(TrailerValue),
    Contains(String),
}

impl ValueMatcher {
    /// Check whether the value supplied matches the matcher
    ///
    pub fn matches(&self, value: &TrailerValue) -> bool
    {
        match self {
            &ValueMatcher::Any             => true,
            &ValueMatcher::Equals(ref v)   => value == v,
            &ValueMatcher::Contains(ref s) => value.to_string().contains(s),
        }
    }

    /// Check whether any of the value supplied matches the matcher
    ///
    pub fn matches_any<I, V>(&self, values: I) -> bool
        where I: IntoIterator<Item = V>,
              V: Borrow<TrailerValue>
    {
        values.into_iter().any(|v| self.matches(v.borrow()))
    }
}


/// Trailer based filter
///
pub struct TrailerFilter<'a> {
    trailer: TrailerSpec<'a>,
    matcher: ValueMatcher,
}

impl<'a> TrailerFilter<'a> {
    /// Create a new trailer filter
    ///
    pub fn new(trailer: TrailerSpec<'a>, matcher: ValueMatcher) -> Self {
        Self { trailer: trailer, matcher: matcher }
    }

    /// Check whether an issue matches the filter
    ///
    /// Rather than the issue itself, this method takes a metadata map which
    /// was previously retrieved via accumulation.
    /// The function returns true if the issue matches the filter, e.g. it
    /// should be displayed or considered for an operation.
    ///
    pub fn matches<'b>(&self, accumulator: &::std::collections::HashMap<String, ValueAccumulator>) -> bool {
        let values = accumulator
            .get(self.trailer.key)
            .cloned()
            .unwrap_or_default();
        self.matcher.matches_any(values)
    }

    /// Retrieve the spec associated with this filter
    ///
    pub fn spec(&self) -> &TrailerSpec<'a> {
        &self.trailer
    }
}

