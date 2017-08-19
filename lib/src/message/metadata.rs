// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

//! Metadata spcification
//!
//! This module provides a type for convenient metadata specification as well as
//! well as specifications for some dit metadata tags.
//!

use std::borrow::Borrow;
use std::iter::FromIterator;

use trailer::accumulation::{AccumulationPolicy, SingleAccumulator, ValueAccumulator};


/// Metadata specification
///
/// Use instances of this type for specifying the names and accumulation rules
/// of pieces of metadata.
///
#[derive(Clone)]
pub struct MetadataSpecification<'k> {
    pub key: &'k str,
    pub accumulation: AccumulationPolicy,
}

impl<'k> MetadataSpecification<'k> {
    /// Create a SingleAccumulator from the specification
    ///
    pub fn single_accumulator(&self) -> SingleAccumulator {
        SingleAccumulator::new(self.key.to_string(), self.accumulation.clone())
    }
}


/// Metadata specification for an issue's type
///
pub const ISSUE_TYPE_SPEC: MetadataSpecification = MetadataSpecification {
    key: "Dit-type",
    accumulation: AccumulationPolicy::Latest,
};

/// Metadata specification for an issue's status
///
pub const ISSUE_STATUS_SPEC: MetadataSpecification = MetadataSpecification {
    key: "Dit-status",
    accumulation: AccumulationPolicy::Latest,
};


/// Construct an accumulation map from a set of MetadataSpecifications
///
/// This trait enables construction of maps from collections of
/// `MetadataSpecification` instances. Use this trait if you want to construct
/// a map-like `Accumulator` (e.g. a `HashMap` or a `BTreeMap`) from a set of
/// specifications in a convenient way.
///
pub trait ToMap {
    /// Construct an accumulation map
    ///
    fn into_map<M>(self) -> M
        where M: FromIterator<(String, ValueAccumulator)>;
}

impl<'s, I, J> ToMap for I
    where I: IntoIterator<Item = J>,
          J: Borrow<MetadataSpecification<'s>>
{
    fn into_map<M>(self) -> M
        where M: FromIterator<(String, ValueAccumulator)>
    {
        self.into_iter()
            .map(|spec| {
                let s = spec.borrow();
                (s.key.to_string(), ValueAccumulator::from(s.accumulation.clone()))
            })
            .collect()
    }
}

