//   git-dit - the distributed issue tracker for git
//   Copyright (C) 2017 Matthias Beyer <mail@beyermatthias.de>
//   Copyright (C) 2017 Julian Ganz <neither@nut.email>
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License version 2 as
//   published by the Free Software Foundation.
//

use libgitdit::Issue;
use libgitdit::trailer::filter::{TrailerFilter, ValueMatcher};
use libgitdit::trailer::{TrailerValue, spec};
use regex::{Regex, Match};
use std::str::FromStr;

use error::*;
use error::ErrorKind as EK;
use gitext::{RemotePriorization, ReferrencesExt};
use system::{Abortable, IteratorExt};


/// Representation of non-trailer metadata
///
enum NonTrailer {
    ReporterName,
    ReporterEMail,
}

impl NonTrailer {
    /// Retrieve the value for a given issue
    ///
    pub fn for_issue(&self, issue: &Issue) -> Result<TrailerValue> {
        match self {
            &NonTrailer::ReporterName => {
                let initial = issue.initial_message()?;
                let value = initial
                    .author()
                    .name()
                    .map(TrailerValue::from_slice)
                    .unwrap_or_default();
                Ok(value)
            },
            &NonTrailer::ReporterEMail => {
                let initial = issue.initial_message()?;
                let value = initial
                    .author()
                    .email()
                    .map(TrailerValue::from_slice)
                    .unwrap_or_default();
                Ok(value)
            },
        }
    }
}


/// Filter specification
///
/// This type represents a filter rule for a single piece of metadata.
///
pub struct FilterSpec {
    /// Metadata to filter
    key: String,
    /// Matcher for the value
    matcher: ValueMatcher,
    /// Indicator whether the filter shall be negated or not
    negated: bool,
}

impl FilterSpec {
    fn into_trailer<'a>(self, spec: spec::TrailerSpec<'a>) -> (TrailerFilter<'a>, bool) {
        (TrailerFilter::new(spec, self.matcher), self.negated)
    }

    fn into_nontrailer<'a>(self, spec: NonTrailer) -> (NonTrailer, ValueMatcher, bool) {
        (spec, self.matcher, self.negated)
    }
}

impl FromStr for FilterSpec {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            // regex for parsing a trailer spec
            static ref RE: Regex = Regex::new(r"^(!)?([[:alnum:]-]+)((=|~)(.*))?$").unwrap();
        }

        let parts = RE
            .captures(s)
            .ok_or_else(|| Error::from_kind(EK::MalformedFilterSpec(s.to_owned())))?;

        let key = parts
            .get(2)
            .as_ref()
            .map(Match::as_str)
            .ok_or_else(|| Error::from_kind(EK::MalformedFilterSpec(s.to_owned())))?;

        let matcher = if parts.get(3).is_some() {
            let op = parts
                .get(4)
                .as_ref()
                .map(Match::as_str)
                .ok_or_else(|| Error::from_kind(EK::MalformedFilterSpec(s.to_owned())))?;

            let value = parts
                .get(5)
                .as_ref()
                .map(Match::as_str)
                .ok_or_else(|| Error::from_kind(EK::MalformedFilterSpec(s.to_owned())))?;

            match op {
                "=" => ValueMatcher::Equals(TrailerValue::from_slice(value)),
                "~" => ValueMatcher::Contains(value.to_string()),
                _   => return Err(Error::from_kind(EK::MalformedFilterSpec(s.to_owned()))),
            }
        } else {
            ValueMatcher::Any
        };

        Ok(FilterSpec {
            key: key.to_string(),
            matcher: matcher,
            negated: parts.get(1).is_some()
        })
    }
}


/// Metadata filter
///
pub struct MetadataFilter<'a> {
    prios: &'a RemotePriorization,
    nontrailers: Vec<(NonTrailer, ValueMatcher, bool)>,
    trailers: Vec<(TrailerFilter<'a>, bool)>,
}

impl<'a> MetadataFilter<'a> {
    /// Create a new metadata filter
    ///
    pub fn new<I>(prios: &'a RemotePriorization, spec: I) -> Result<Self>
        where I: IntoIterator<Item = FilterSpec>
    {
        let mut nontrailers = Vec::new();
        let mut trailers = Vec::new();

        for s in spec.into_iter() {
            match s.key.as_ref() {
                "status"            => trailers.push(s.into_trailer(spec::ISSUE_STATUS_SPEC.clone())),
                "type"              => trailers.push(s.into_trailer(spec::ISSUE_TYPE_SPEC.clone())),
                "reporter-name"     => nontrailers.push(s.into_nontrailer(NonTrailer::ReporterName)),
                "reporter-email"    => nontrailers.push(s.into_nontrailer(NonTrailer::ReporterEMail)),
                _                   => return Err(Error::from_kind(EK::UnknownMetadataKey(s.key.to_string()))),
            }
        }

        Ok(MetadataFilter { prios: prios, nontrailers: nontrailers, trailers: trailers })
    }

    /// Create an empty metadata filter
    ///
    /// The filter will not filter out any issues.
    ///
    pub fn empty(prios: &'a RemotePriorization) -> Self {
        MetadataFilter {
            prios: prios,
            nontrailers: Vec::new(),
            trailers: Vec::new(),
        }
    }

    /// Filter an issue
    ///
    pub fn filter(&self, issue: &Issue) -> bool {
        // NOTE: if we ever add the filters crate as a dependency, this method
        //       may be transferred to an implementatio nof the Filter trait
        use git2::ObjectType;
        use libgitdit::iter::MessagesExt;
        use std::collections::HashMap;

        // Check non-trailer metadata first
        for metadata in self.nontrailers.iter() {
            let value = metadata.0.for_issue(issue).unwrap_or_abort();
            if !metadata.1.matches(&value) ^ metadata.2 {
                return false;
            }
        }

        // Filtering may be expensive, so it makes sense to return early if the
        // filter is empty.
        if self.trailers.is_empty() {
            return true;
        }

        // Get the head reference
        let head = issue
            .heads()
            .abort_on_err()
            .select_ref(self.prios)
            .map(|head| head.peel(ObjectType::Commit).unwrap_or_abort().id());

        // Accumulate all the metadata we care about
        let acc: HashMap<_, _> = head
            .into_iter()
            .flat_map(|head| issue.messages_from(head).abort_on_err())
            .accumulate_trailers(self.trailers.iter().map(|i| i.0.spec()));

        // Compute whether all constraints are met
        self.trailers
            .iter()
            .all(|spec| spec.0.matches(&acc) ^ spec.1)
    }
}

