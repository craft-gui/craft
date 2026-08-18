pub trait Command {
    fn merge(&mut self, other: &Self) -> bool;
}

#[cfg(test)]
mod tests {
    use alloc::string::{String, ToString};

    use super::Command;

    enum TextCommand {
        Insert(String),
        Delete,
    }

    impl Command for TextCommand {
        fn merge(&mut self, other: &Self) -> bool {
            if let TextCommand::Insert(left) = self
                && let TextCommand::Insert(right) = other
            {
                left.push_str(right);
                true
            } else {
                false
            }
        }
    }

    #[test]
    fn test_mergeable_commands_is_true() {
        let mut insert_a = TextCommand::Insert("a".to_string());
        let insert_b = TextCommand::Insert("a".to_string());

        assert!(insert_a.merge(&insert_b))
    }

    #[test]
    fn test_non_mergeable_commands_is_false() {
        let mut insert_a = TextCommand::Insert("a".to_string());
        let delete = TextCommand::Delete;

        assert!(!insert_a.merge(&delete))
    }
}
