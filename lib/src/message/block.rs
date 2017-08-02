// git-dit - the distributed issue tracker for git
// Copyright (C) 2016, 2017 Matthias Beyer <mail@beyermatthias.de>
// Copyright (C) 2016, 2017 Julian Ganz <neither@nut.email>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//

//! Line block categorization
//!
//! When processing messages, we may have to differentiate between blocks of
//! text-lines (paragraphs) and blocks of trailers.
//!
//! This module provides a type for representing the different types of blocks as
//! well as an iterator for extracting the blocks from a sequence of lines.
//!

use message::trailer::Trailer;
use std::str::FromStr;


/// A block of lines
///
/// We differentiate between paragraphs and blocks of trailers.
///
pub enum Block {
    Text(Vec<String>),
    Trailer(Vec<Trailer>),
}


/// Iterate over blocks of lines instead of lines
///
/// This iterator wraps an iterator over lines and offers iteration over the
/// blocks found in the sequence of lines. Paragraphs and blocks of text are
/// cleanly separated from another.
///
#[derive(Debug)]
pub struct Blocks<I, S>(I)
    where I: Iterator<Item = S>,
          S: AsRef<str>;

impl<I, S> From<I> for Blocks<I, S>
    where I: Iterator<Item = S>,
          S: AsRef<str>
{
    fn from(iter: I) -> Self {
        Blocks(iter)
    }
}

impl<I, S> Iterator for Blocks<I, S>
    where I: Iterator<Item = S>,
          S: AsRef<str>
{
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        let mut lines = Vec::new();
        let mut trailers: Vec<Trailer> = Vec::new();
        let mut is_trailer = true;

        // get us the next block
        for line in &mut self.0 {
            let trimmed = line.as_ref().trim_right();

            // If we encounter an empty line, we are done. However, we should
            // refrain from reporting empty blocks.
            if trimmed.is_empty() {
                if lines.is_empty() {
                    continue;
                } else {
                    break;
                }
            }

            // Even if we encountered only trailers in the current block, we
            // keep all the lines. We might need them in case the block turns
            // out to be a paragraph.
            lines.push(trimmed.to_string());

            // Parsing trailers is far more expensive than accumulating strings.
            if !is_trailer {
                continue;
            }

            if trimmed.starts_with(" ") {
                // We encountered a part of a multiline trailer.
                if let Some(ref mut trailer) = trailers.last_mut() {
                    trailer.value.append(trimmed);
                } else {
                    // Turns out this is a paragraph with the first line being
                    // indented.
                    is_trailer = false;
                }
            } else if let Ok(trailer) = Trailer::from_str(trimmed) {
                // This looks like a trailer.
                trailers.push(trailer);
            } else {
                // It's just text.
                is_trailer = false;
            }
        }

        // If we did not encounter any lines at all, we must have run out of
        // lines.
        if lines.is_empty() {
            return None;
        }

        if is_trailer {
            Some(Block::Trailer(trailers))
        } else {
            Some(Block::Text(lines))
        }
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    use message::trailer::{TrailerKey, TrailerValue};

    // Blocks test

    #[test]
    fn trailers() {
        let mut blocks = Blocks::from(vec![
            "Foo-bar: bar",
            "",
            "Space: the final frontier.",
            "These are the voyages...",
            "",
            "And then he",
            "said: engage!",
            "",
            "And now",
            "for something completely different.",
            "",
            "",
            "Signed-off-by: Spock",
            "Dit-status: closed",
            "Multi-line-trailer: multi",
            "  line",
            "  content"
        ].into_iter());

        match blocks.next().expect("Failed to retrieve block 1") {
            Block::Trailer(trailers) => {
                let mut iter = trailers.iter();

                let trailer = iter.next().expect("Failed to parse trailer 1");
                assert_eq!(trailer.key, TrailerKey::from("Foo-bar".to_string()));
                assert!(iter.next().is_none());
            },
            _ => panic!("Wrong type for block 1")
        }

        match blocks.next().expect("Failed to retrieve block 2") {
            Block::Text(lines) => assert_eq!(lines, vec![
                "Space: the final frontier.",
                "These are the voyages..."
            ]),
            _ => panic!("Wrong type for block 2")
        }

        match blocks.next().expect("Failed to retrieve block 3") {
            Block::Text(lines) => assert_eq!(lines, vec![
                "And then he",
                "said: engage!",
            ]),
            _ => panic!("Wrong type for block 3")
        }

        match blocks.next().expect("Failed to retrieve block 4") {
            Block::Text(lines) => assert_eq!(lines, vec![
                "And now",
                "for something completely different.",
            ]),
            _ => panic!("Wrong type for block 4")
        }

        match blocks.next().expect("Failed to retrieve block 5") {
            Block::Trailer(trailers) => {
                let mut iter = trailers.iter();

                {
                    let trailer = iter.next().expect("Failed to parse trailer 2");
                    assert_eq!(trailer.key, TrailerKey::from("Signed-off-by".to_string()));
                }

                {
                    let trailer = iter.next().expect("Failed to parse trailer 3");
                    assert_eq!(trailer.key, TrailerKey::from("Dit-status".to_string()));
                }

                {
                    let trailer = iter.next().expect("Failed to parse trailer 4");
                    assert_eq!(trailer.key, TrailerKey::from("Multi-line-trailer".to_string()));
                    assert_eq!(trailer.value, TrailerValue::String("multi  line  content".to_string()));
                }

                assert!(iter.next().is_none());
            },
            _ => panic!("Wrong type for block 5")
        }

        assert!(!blocks.next().is_some())
    }
}
