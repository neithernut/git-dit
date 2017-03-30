// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

use std::ops::Add;

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
                false => String::from("> ").add(line),
            }
        })
    }
}


