// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use git2::{Oid, References, ReferenceNames};

use error::*;
use error::ErrorKind as EK;

pub struct HeadRefsToIssuesIter<'r>(ReferenceNames<'r>);

impl<'r> Iterator for HeadRefsToIssuesIter<'r> {
    type Item = Result<Oid>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|r_name|  {
                r_name
                    .chain_err(|| EK::WrappedGitError)
                    .and_then(|name| if name.ends_with("/head") {
                        name.rsplitn(3, "/")
                            .nth(1)
                            .ok_or_else(|| {
                                Error::from_kind(EK::MalFormedHeadReference(name.to_string()))
                            })
                            .and_then(|hash| {
                                Oid::from_str(hash)
                                    .chain_err(|| EK::OidFormatError(name.to_string()))
                            })
                    } else {
                        Err(Error::from_kind(EK::MalFormedHeadReference(name.to_string())))
                    })
            })
    }
}

impl<'r> From<References<'r>> for HeadRefsToIssuesIter<'r> {
    fn from(r: References<'r>) -> HeadRefsToIssuesIter<'r> {
        HeadRefsToIssuesIter(r.names())
    }
}

impl<'r> From<ReferenceNames<'r>> for HeadRefsToIssuesIter<'r> {
    fn from(r: ReferenceNames<'r>) -> HeadRefsToIssuesIter<'r> {
        HeadRefsToIssuesIter(r)
    }
}

