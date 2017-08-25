//   git-dit - the distributed issue tracker for git
//   Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
//   Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License version 2 as
//   published by the Free Software Foundation.
//

use std::fmt::Display;
use std::io::{Result as RResult, Write};

pub trait WriteExt {
    fn consume_lines<I, L>(&mut self, lines: I) -> RResult<()>
        where I: IntoIterator<Item = L>,
              L: Display;
}

impl<W> WriteExt for W
    where W: Write
{
    fn consume_lines<I, L>(&mut self, lines: I) -> RResult<()>
        where I: IntoIterator<Item = L>,
              L: Display
    {
        for line in lines {
            write!(self, "{}\n", line)?;
        }
        Ok(())
    }
}
