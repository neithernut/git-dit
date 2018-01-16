//   git-dit - the distributed issue tracker for git
//   Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
//   Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License version 2 as
//   published by the Free Software Foundation.
//

//! Message formatting facility
//!

use chrono::format::strftime::StrftimeItems;
use git2::{Commit, Oid};
use libgitdit::Message;
use libgitdit::message::block::Block;
use libgitdit::trailer::spec::TrailerSpec;
use colored::{ColoredString, Colorize};

use error::*;
use super::formatter::{TokenExpander, FormattingToken, LineTokens};

/// Tokens for formatting messages
///
#[derive(Clone)]
pub enum MessageFmtToken<'a> {
    Id(usize),
    Subject,
    Author,
    AuthorName,
    AuthorEMail,
    Date(StrftimeItems<'a>),
    Body,
    BodyText,
    Trailers,
    Trailer(TrailerSpec<'a>),
    IfId(Oid, Vec<FormattingToken<MessageFmtToken<'a>, Commit<'a>>>),
}

impl<'a,> TokenExpander for MessageFmtToken<'a> {
    type Item = Commit<'a>;
    type Error = Error;

    fn expand_token(&self, message: &Self::Item) -> Result<Vec<FormattingToken<Self, Self::Item>>> {
        Ok(match self {
            &MessageFmtToken::Id(ref len) => tokenvec![format!("{0:.1$}", message.id(), len).red().to_string()],
            &MessageFmtToken::Subject => tokenvec![message
                .as_object()
                .clone()
                .into_commit()
                .ok()
                .and_then(|mut m| {
                    m.summary().map(Colorize::yellow).map(|cs| cs.to_string())
                })
                .unwrap_or_default()],
            &MessageFmtToken::Author => tokenvec![message
                .author()
                .to_string()
                .green()
                .to_string()],
            &MessageFmtToken::AuthorName => tokenvec![message
                .author()
                .name()
                .unwrap_or_default()
                .green()
                .to_string()],
            &MessageFmtToken::AuthorEMail => tokenvec![message
                .author()
                .email()
                .unwrap_or_default()
                .green()
                .to_string()],
            &MessageFmtToken::Date(ref format) => {
                use chrono::{FixedOffset, TimeZone};

                let gtime = message.time();
                tokenvec![FixedOffset::east(gtime.offset_minutes()*60)
                    .timestamp(gtime.seconds(), 0)
                    .format_with_items(format.clone())
                    .to_string()
                    .green()
                    .to_string()]
            }
            &MessageFmtToken::Body => message
                .body_lines()
                .line_tokens()
                .collect(),
            &MessageFmtToken::BodyText => message
                .body_blocks()
                .flat_map(|block| match block {
                    Block::Text(mut lines) => {
                        // add a blank line after each paragraph
                        lines.push(String::new());
                        lines
                    },
                    _ => vec![],
                })
                .line_tokens()
                .collect(),
            &MessageFmtToken::Trailers => message
                .trailers()
                .line_tokens()
                .collect(),
            &MessageFmtToken::Trailer(ref spec) => message
                .trailers()
                .filter(|trailer| trailer.key.as_ref() == spec.key)
                .line_tokens()
                .collect(),
            &MessageFmtToken::IfId(ref id, ref tokens) => if *id == message.id() {
                tokens.clone()
            } else {
                Vec::new()
            },
        })
    }
}

