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


/// Policy for accumulating trailers
///
/// These enum values represent accumulation policies for trailers, e.g. how
/// trailer values are accumulated.
///
pub enum AccumulationPolicy {
    Latest,
    List,
}

