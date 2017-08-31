//   git-dit - the distributed issue tracker for git
//   Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
//   Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License version 2 as
//   published by the Free Software Foundation.
//

use std::fmt::Display;
use std::io::{self, Result as RResult, Write};


/// Extension trait for convenient writing of lines
///
pub trait LinesExt: Sized {
    /// Write the items returned as lines to a given stream
    ///
    fn write_lines(self, stream: &mut Write) -> RResult<()>;

    /// Write the items returned as lines to stdout
    ///
    fn print_lines(self) -> RResult<()> {
        let mut stream = io::stdout();
        self.write_lines(&mut stream)
    }
}

impl<I, L> LinesExt for I
    where I: IntoIterator<Item = L>,
          L: Display
{
    fn write_lines(self, stream: &mut Write) -> RResult<()>
    {
        for line in self {
            write!(stream, "{}\n", line)?;
        }
        Ok(())
    }
}

