use alloc::vec::Vec;

use crate::Command;

#[derive(Default, Clone)]
pub struct UndoManager<T: Command> {
    undo_stack: Vec<T>,
    redo_stack: Vec<T>,
}

impl<T: Command> UndoManager<T> {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn undo_command(&mut self) -> Option<&T> {
        if let Some(command) = self.undo_stack.pop() {
            self.redo_stack.push(command);
            self.redo_stack.last()
        } else {
            None
        }
    }

    pub fn execute_command(&mut self, command: T) {
        self.redo_stack.clear();
        self.undo_stack.push(command);
    }

    pub fn redo_command(&mut self) -> Option<&T> {
        if let Some(command) = self.redo_stack.pop() {
            self.undo_stack.push(command);
            self.undo_stack.last()
        } else {
            None
        }
    }

    pub fn merge(&mut self) {
        let len = self.undo_stack.len();
        if len < 2 {
            return;
        }
        let [c1, c2] = self.undo_stack.get_disjoint_mut([len - 2, len - 1]).unwrap();
        if c1.merge(c2) {
            self.undo_stack.pop();
        }
    }

    pub fn undo_commands(&self) -> &[T] {
        &self.undo_stack
    }

    pub fn redo_commands(&self) -> &[T] {
        &self.redo_stack
    }
}
