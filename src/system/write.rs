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
use std::process::Child;

use atty;

use error::*;
use error::ErrorKind as EK;


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

    /// Pipe lines to a child process
    ///
    /// If stdout is a TTY, pipe the lines to the child process and wait until
    /// it is closed. Otherwise, just print them to stdout.
    ///
    /// Returns `0` on success and a non-null return code if an error occured
    /// in the child.
    ///
    /// # Note
    ///
    /// The `child` provided must provide an `stdin` field which is not `None`,
    /// e.g. it must accept data via standart input. Otherwise, this function
    /// panics.
    ///
    fn pipe_lines(self, mut child: Child) -> Result<i32> {
        if atty::is(atty::Stream::Stdout) {
            // NOTE: this unwrap is ok via the requirements on `child`.
            self.write_lines(child.stdin.as_mut().unwrap())
                .chain_err(|| Error::from(EK::WrappedIOError))?;

            child
                .wait()
                .chain_err(|| Error::from(EK::ChildError))
                .map(|result| result.code().unwrap_or(1))
        } else {
            self.print_lines()
                .chain_err(|| Error::from(EK::WrappedIOError))
                .map(|_| 0)
        }
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

