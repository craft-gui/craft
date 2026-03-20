use craft_undo::{Command, UndoManager};

#[derive(Debug, PartialEq)]
enum TextCommand {
    Insert(String),
}

impl Command for TextCommand {
    fn merge(&mut self, other: &Self) -> bool {
        if let TextCommand::Insert(left) = self
            && let TextCommand::Insert(right) = other
        {
            left.push_str(right);
            return true;
        }
        false
    }
}

#[test]
fn test_merge() {
    let mut x = UndoManager::<TextCommand>::new();
    x.execute_command(TextCommand::Insert("a".to_string()));
    x.execute_command(TextCommand::Insert("b".to_string()));
    x.execute_command(TextCommand::Insert("c".to_string()));
    x.merge();
    x.merge();
    let foo = x.undo_commands().last();
    assert_eq!(foo, Some(&TextCommand::Insert("abc".to_string())))
}
