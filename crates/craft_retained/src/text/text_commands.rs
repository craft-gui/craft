use std::ops::Range;

use craft_undo::Command;
use parley::{Affinity, Selection};

#[derive(Clone)]
pub struct TextInsertion {
    pub str: String,
    pub range: Range<usize>,
    pub affinity: Affinity,
}

impl TextInsertion {
    pub fn new(str: String, range: Range<usize>, affinity: Affinity) -> Self {
        Self {
            str,
            range,
            affinity,
        }
    }
}

#[derive(Clone)]
pub struct TextReplace {
    pub new_str: String,
    pub old_str: String,
    pub selection: Selection,
    pub affinity: Affinity,
    pub new_selection: Selection,
}

impl TextReplace {
    pub fn new(
        new_str: String,
        old_str: String,
        selection: Selection,
        affinity: Affinity,
        new_selection: Selection,
    ) -> Self {
        Self {
            new_str,
            old_str,
            selection,
            affinity,
            new_selection,
        }
    }
}

#[derive(Clone)]
pub struct Backspace {
    pub str: String,
    pub range: Range<usize>,
    pub affinity: Affinity,
}

impl Backspace {
    pub fn new(str: String, range: Range<usize>, affinity: Affinity) -> Self {
        Self {
            str,
            range,
            affinity,
        }
    }
}

#[derive(Clone)]
pub struct Delete {
    pub str: String,
    pub range: Range<usize>,
    pub affinity: Affinity,
}

impl Delete {
    pub fn new(str: String, range: Range<usize>, affinity: Affinity) -> Self {
        Self {
            str,
            range,
            affinity,
        }
    }
}

#[derive(Clone)]
pub enum TextCommand {
    TextInsertion(TextInsertion),
    TextReplace(TextReplace),
    Backspace(Backspace),
    Delete(Delete),
}

impl Command for TextCommand {
    fn merge(&mut self, other: &Self) -> bool {
        match (self, other) {
            (TextCommand::TextInsertion(left), TextCommand::TextInsertion(right)) => {
                if right.str.eq(" ")
                    || right.str.eq("\n")
                    || right.str.eq(".")
                    || left.str.eq(" ")
                    || left.str.eq("\n")
                    || left.str.eq(".")
                {
                    return false;
                }

                if left.range.start + left.str.len() == right.range.start {
                    left.str.push_str(&right.str);
                    left.range.end = right.range.end;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
