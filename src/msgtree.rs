//   git-dit - the distributed issue tracker for git
//   Copyright (C) 2016 Matthias Beyer <mail@beyermatthias.de>
//   Copyright (C) 2016 Julian Ganz <neither@nut.email>
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License version 2 as
//   published by the Free Software Foundation.
//

//! Helper types for displaying message trees
//!
//! This module provides multiple helper types which may be used for generating
//! a textual tree representation from a commit sequence, e.g. using the
//! `IntoTreeGraph` convenience trait. The commit sequence has to be ordered
//! such that the iterator returns message's parents only after all replies have
//! been returned.
//!
//! The graph is represented using columns of `TreeGraphElem`s. Each single
//! column represents a slot availible for the representation of a thread, with
//! a `TreeGraphElem::Mark` denoting commits. A line may contain multiple marks,
//! in which case the commit represented by the line is a merge-point for the
//! threads on which the marks are placed. Within a column, marks are connected
//! via the `TreeGraphElem::Following` element. An `TreeGraphElem::Empty`
//! element may be used as a place holder if a column is not occupied by a
//! thread.
//!


use git2::{Commit, Oid};
use std::fmt::{self, Write};
use std::iter::FromIterator;


/// Representation of graph elements used to display trees
///
#[derive(Clone, PartialEq)]
pub enum TreeGraphElem {
    Empty,
    Following,
    Mark(MarkType),
}

impl TreeGraphElem {
    /// Get the character for the tree graph element
    ///
    pub fn to_char(&self) -> char {
        match self {
            &TreeGraphElem::Empty     => ' ',
            &TreeGraphElem::Following => '|',
            &TreeGraphElem::Mark(_)   => '+',
        }
    }
}


/// Representation of the type of mark
///
/// A mark may start or terminate a thread. This information is required for
/// proper formatting in cases where the representation of a commit spreads
/// across multiple lines.
///
#[derive(Clone, PartialEq)]
pub enum MarkType {
    Start,
    Mid,
    End
}

impl MarkType {
    /// Replace `Start` with `End` and the other way round
    ///
    pub fn reverse(&mut self) {
        match self {
            &mut MarkType::Start => *self = MarkType::End,
            &mut MarkType::End   => *self = MarkType::Start,
            _ => {},
        }
    }
}


/// Representation of one line of tree graph elements
///
pub struct TreeGraphElemLine(Vec<TreeGraphElem>);

impl TreeGraphElemLine {
    /// Append a graph element to the line
    pub fn append(&mut self, e: TreeGraphElem) {
        self.0.push(e);
    }

    /// Reverse "start" and "end" marks
    ///
    pub fn reverse_marks(&mut self) {
        for elem in self.0.iter_mut() {
            match elem {
                &mut TreeGraphElem::Mark(ref mut mt) => mt.reverse(),
                _ => {},
            }
        }
    }

    /// Transform into an iterator over lines for one commit
    ///
    pub fn commit_iterator(self) -> CommitTreeGraphLines {
        CommitTreeGraphLines(self.0)
    }
}

impl FromIterator<TreeGraphElem> for TreeGraphElemLine {
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item=TreeGraphElem>
    { TreeGraphElemLine(iter.into_iter().collect()) }
}

impl fmt::Display for TreeGraphElemLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut has_mark = false;
        for elem in self.0.iter() {
            f.write_char(if is_match!(elem, &TreeGraphElem::Mark(_)) {
                has_mark = true;
                '+'
            } else {
                if has_mark { '-' } else { elem.to_char() }
            })?;
        }
        Ok(())
    }
}


/// Iterator returning the graph elements for a single commit
///
/// Only the first line returned by the iterator will contain marks. The other
/// lines will contain only "following" and "empty" such that "start" and "end"
/// marks will delimit a thread.
///
pub struct CommitTreeGraphLines(Vec<TreeGraphElem>);

impl Iterator for CommitTreeGraphLines {
    type Item = TreeGraphElemLine;

    fn next(&mut self) -> Option<Self::Item> {
        let retval = TreeGraphElemLine(self.0.clone());

        // marks shall only be part in the first line for each commit
        for elem in self.0.iter_mut() {
            match elem {
                &mut TreeGraphElem::Mark(MarkType::End) => *elem = TreeGraphElem::Empty,
                &mut TreeGraphElem::Mark(_) => *elem = TreeGraphElem::Following,
                _ => {},
            }
        }

        Some(retval)
    }
}


/// Iterator generating graph elements for a series of commits
///
/// This iterator generates lines of tree graph elements from an iterator over
/// commits. If the lines are printed in the order extracted from this iterator,
/// it will result in a graph mirroring the topology of the commits on which the
/// iterator is based.
///
pub struct TreeGraphElemLineIterator<'r, I>
    where I: Iterator<Item = Commit<'r>>
{
    inner: I, // inner iterator over commits for which to display the graph
    parents: Vec<Option<Oid>>, // currently tracked parents
}

impl<'r, I> Iterator for TreeGraphElemLineIterator<'r, I>
    where I: Iterator<Item = Commit<'r>>
{
    type Item = (TreeGraphElemLine, Commit<'r>);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|commit| {
            // We will definitely require the parent id in order to draw a graph.
            // However, we only want to track the parent once, so we end up with
            // nice horizontal merges in our graph.
            let mut parent_update = commit.parent(0).as_ref().map(Commit::id).ok();

            // generate graph elements for the parents currently tracked
            let mut elems : TreeGraphElemLine = self.parents.iter_mut().map(|mut parent| {
                match *parent {
                    Some(id) => if commit.id() == id {
                            // the current commit is a parent we were awaiting
                            *parent = parent_update.take();
                            let mark_type = if parent.is_some() { MarkType::Mid } else { MarkType::End };
                            TreeGraphElem::Mark(mark_type)
                        } else { TreeGraphElem::Following },
                    None => TreeGraphElem::Empty,
                }
            }).collect();

            if parent_update.is_some() {
                // a history for the current commit doesn't yet exist, so we
                // should create one
                elems.append(TreeGraphElem::Mark(MarkType::Start));
                self.parents.push(parent_update);
            }

            // keep the graph slim by removing parent trackig information that
            // is no longer required from the back of the list
            while self.parents.last().map(Option::is_none).unwrap_or(false) {
                self.parents.pop();
            }

            (elems, commit)
        })
    }
}


/// Extension trait for convenient creation of graph iterators from commits
///
pub trait IntoTreeGraph<'r, I>
    where I: Iterator<Item = Commit<'r>>
{
    /// Transform self into a tree graph iterator
    ///
    /// The iterator on which this function is used must return a message only
    /// after all the replies to that message.
    ///
    fn into_tree_graph(self) -> TreeGraphElemLineIterator<'r, I>;
}

impl<'r, I> IntoTreeGraph<'r, I> for I
    where I: Iterator<Item = Commit<'r>>
{
    fn into_tree_graph(self) -> TreeGraphElemLineIterator<'r, Self> {
        TreeGraphElemLineIterator { inner: self, parents: vec![] }
    }
}

