//   git-dit - the distributed issue tracker for git
//   Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
//   Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License version 2 as
//   published by the Free Software Foundation.
//

use log;
use io::{stderr, Write};
use std::result::Result as RResult;


/// Basic logger
///
/// This logger will log to stderr
///
pub struct Logger;

impl Logger {
    /// Initialize the basic logger
    ///
    /// Instantiate a basic logger and make it the main logger.
    ///
    pub fn init(level: log::Level) -> RResult<(), log::SetLoggerError> {
        log::set_logger(&Logger)?;
        log::set_max_level(level.to_level_filter());
        Ok(())
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= ::log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            writeln!(stderr(), "{}", record.args()).ok();
        }
    }

    fn flush(&self) {
        /* implementation not needed as we do not cache */
    }
}

