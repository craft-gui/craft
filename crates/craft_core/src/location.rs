use smol_str::SmolStr;

#[derive(Debug, Clone)]
pub struct Location {
    url: String,
}

impl Location {
    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    pub fn pathname(&self) -> String {
        let mut pathname: String = String::with_capacity(self.url().len());
        for char in self.url.chars() {
            pathname.push(char);
            if char == '?' {
                break;
            }
        }

        pathname
    }

    pub fn query(&self) -> Query {
        todo!()
    }
}

pub struct Query(pub Vec<(SmolStr, SmolStr)>);

impl Query {

    pub fn get(&self, name: &str) -> Option<&str> {
        for (key, value) in &self.0 {
            if key == name {
                return Some(value.as_str());
            }
        }

        None
    }

    pub fn set(&mut self, name: &str, value: &str) {
        let contains = self.0.iter().any(|(k, _)| k == name);
        
        let mut is_first = true;
        if contains {
            self.0.retain_mut(|(key, current_value)| {
                if key == name && is_first {
                    is_first = false;
                    *current_value = value.into();
                    true
                } else {
                    key != name
                }
            });
        } else {
            self.0.push((SmolStr::new(name), SmolStr::new(value)));   
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::location::Query;

    #[test]
    fn get_value_first_selected() {
        const FIRST_VALUE: &str = "value";
        let query = Query(
            vec![
                ("key".into(), FIRST_VALUE.into()),
                ("key".into(), "value1".into()),
                ("key".into(), "value2".into()) 
            ]
        );
       
        assert_eq!(query.get("key"), Some(FIRST_VALUE));
    }

    #[test]
    fn set_value_new_name_should_append_end() {
        let mut query = Query(vec![
            ("key".into(), "value".into()),
        ]);

        query.set("NEW", "new value");
        assert_eq!(query.0.last().unwrap().0, "NEW");
    }

    #[test]
    fn set_value_existing_name_should_update_existing_value() {
        let mut query = Query(vec![
            ("key".into(), "value".into()),
            ("NEW".into(), "my value".into()),
        ]);

        query.set("NEW", "new value");
        assert_eq!(query.0.last().unwrap().1, "my value");
    }

    #[test]
    fn set_value_duplicate_name_should_delete_existing_value() {
        let mut query = Query(vec![
            ("key".into(), "value".into()),
            ("NEW".into(), "my value".into()),
            ("NEW".into(), "my value12".into()),
        ]);

        query.set("NEW", "new value");
        println!("{:?}", query.0);
        assert_eq!(query.0.get(1).unwrap().1, "new value");
        assert!(query.0.len() < 3);
    }
}
