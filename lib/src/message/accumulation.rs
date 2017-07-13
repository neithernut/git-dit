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

use message::trailer::TrailerValue;

/// Policy for accumulating trailers
///
/// These enum values represent accumulation policies for trailers, e.g. how
/// trailer values are accumulated.
///
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

