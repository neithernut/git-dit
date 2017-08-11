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
use std::error::Error;
use std::result::Result as RResult;


/// Basic logger
///
/// This logger will log to stderr
///
pub struct Logger {
    level: log::LogLevel,
}

impl Logger {
    /// Initialize the basic logger
    ///
    /// Instantiate a basic logger and make it the main logger.
    ///
    pub fn init(level: log::LogLevel) -> RResult<(), log::SetLoggerError> {
        log::set_logger(|max_level| {
            max_level.set(level.to_log_level_filter());
            Box::from(Logger { level: level })
        })
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::LogMetadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::LogRecord) {
        if self.enabled(record.metadata()) {
            writeln!(stderr(), "{}", record.args()).ok();
        }
    }
}


/// Convenience trait for logging error types
///
/// Logs all layers of an error using the `error!` macro.
///
pub trait LoggableError {
    fn log(&self);
}

impl<E> LoggableError for E
    where E: Error
{
    fn log(&self) {
        let mut current = Some(self as &Error);
        while let Some(err) = current {
            error!("{}", err);
            current = err.cause();
        }
    }
}

