use std::fmt::Display;

use crate::input;

mod commands;

pub use self::commands::handle;

const INDENT_SIZE: usize = 4;

pub struct Tracker {
    name: String,
    value: Option<i32>,
    children: Vec<Tracker>,
}

impl Tracker {
    fn make(name: &str, value: Option<i32>) -> Self {
        Self {
            name: name.to_string(),
            value,
            children: Vec::new(),
        }
    }

    pub fn new(name: &str) -> Self {
        Self::make(name, None)
    }

    pub fn add(&mut self, child: Tracker) {
        self.children.push(child);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get(&self, name: &str) -> Option<&Tracker> {
        let child = self.children.iter().find(|c| c.name() == name);
        if child.is_some() {
            child
        } else {
            for child in &self.children {
                let descendent = child.get(name);
                if descendent.is_some() {
                    return descendent;
                }
            }
            None
        }
    }

    pub fn print(&self) {
        println!("{self}");
    }

    fn format(&self, indent: usize) -> String {
        let mut heading = format!("{}{}:", " ".repeat(indent * INDENT_SIZE), self.name());
        if let Some(value) = self.value {
            heading.push_str(&format!(" {value}"));
        }

        if self.children.is_empty() {
            heading
        } else {
            self.children
                .iter()
                .map(|c| c.format(indent + 1))
                .fold(heading, |mut s, c| {
                    s.push('\n');
                    s.push_str(&c);
                    s
                })
        }
    }

    pub fn handle(&self, input: &str) {
        match input::command(input) {
            "" => self.print(),
            _ => {}
        };
    }
}

impl Display for Tracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format(0))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_format_tracker() {
        assert_eq!(
            Tracker::make("tracker", Some(42)).format(2),
            "        tracker: 42"
        );
    }

    #[test]
    fn test_format_collection() {
        assert_eq!(
            Tracker {
                name: "collection".to_string(),
                value: None,
                children: vec![
                    Tracker::make("tracker1", Some(1)),
                    Tracker::make("tracker2", Some(2)),
                ]
            }
            .format(1),
            "    collection:\n        tracker1: 1\n        tracker2: 2"
        );
    }

    #[test]
    fn test_get() {
        let mut root = Tracker::new("trackers");
        root.add(Tracker::new("child1"));
        let mut child2 = Tracker::new("child2");
        child2.add(Tracker::new("grandchild"));
        root.add(child2);
        assert!(root.get("grandchild").is_some());
    }
}
