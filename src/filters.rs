//   git-dit - the distributed issue tracker for git
//   Copyright (C) 2017 Matthias Beyer <mail@beyermatthias.de>
//   Copyright (C) 2017 Julian Ganz <neither@nut.email>
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License version 2 as
//   published by the Free Software Foundation.
//

use libgitdit::Issue;
use libgitdit::message::accumulation::{Accumulator, ValueAccumulator};
use libgitdit::message::trailer::TrailerValue;
use libgitdit::message::{Message, metadata};

use abort::{Abortable, IteratorExt};
use reference::{self, ReferrencesExt};


/// Filter specification
///
/// This type represents a filter rule for a single piece of metadata.
///
pub struct FilterSpec<'a> {
    /// Metadata to filter
    metadata: metadata::MetadataSpecification<'a>,
    /// Expected value
    value: TrailerValue,
}

impl<'a> FilterSpec<'a> {
    /// Apply the filter rule to a piece of accumulated values
    ///
    /// This function returns true if the filter applies.
    ///
    pub fn apply_to_values(&self, values: ValueAccumulator) -> bool {
        values.into_iter().any(|v| v == self.value)
    }
}


/// Metadata filter
///
pub struct MetadataFilter<'a> {
    prios: &'a reference::RemotePriorization,
    spec: Vec<FilterSpec<'a>>,
}

impl<'a> MetadataFilter<'a> {
    /// Create an empty metadata filter
    ///
    /// The filter will not filter out any issues.
    ///
    pub fn empty(prios: &'a reference::RemotePriorization) -> Self {
        MetadataFilter {
            prios: prios,
            spec: Vec::new(),
        }
    }

    /// Filter an issue
    ///
    pub fn filter(&self, issue: &Issue) -> bool {
        // NOTE: if we ever add the filters crate as a dependency, this method
        //       may be transferred to an implementatio nof the Filter trait
        use git2::ObjectType;
        use libgitdit::message::metadata::ToMap;
        use std::collections::HashMap;

        // Filtering may be expensive, so it makes sense to return early if the
        // filter is empty.
        if self.spec.is_empty() {
            return true;
        }

        // Construct an iterator over trailers
        let trailers = issue
            .heads()
            .abort_on_err()
            .select_ref(self.prios)
            .into_iter()
            .map(|head| head.peel(ObjectType::Commit).unwrap_or_abort().id())
            .flat_map(|head| issue.messages_from(head).abort_on_err())
            .flat_map(|message| message.trailers());

        // Accumulate all the metadata we care about
        let mut acc: HashMap<String, ValueAccumulator> = self
            .spec
            .iter()
            .map(|i| &i.metadata)
            .into_map();
        acc.process_all(trailers);

        // Compute whether all constraints are met
        self.spec
            .iter()
            .all(|spec| {
                acc.remove(&spec.metadata.key.to_owned())
                    .map(|values| spec.apply_to_values(values))
                    .unwrap_or(false)
            })
    }
}

