use std::fmt::Display;

use crate::input;

const INDENT_SIZE: usize = 4;

trait TrackerNode {
    fn name(&self) -> &str;

    fn children(&self) -> &[&dyn TrackerNode];

    fn format(&self, indent: usize) -> String;

    fn print(&self) {
        println!("{}", self.format(0));
    }

    fn handle(&self, input: &str);
}

impl Display for dyn TrackerNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format(0))
    }
}

struct Tracker {
    name: String,
    value: i32,
}

impl TrackerNode for Tracker {
    fn name(&self) -> &str {
        &self.name
    }

    fn children(&self) -> &[&dyn TrackerNode] {
        &[]
    }

    fn format(&self, indent: usize) -> String {
        format!(
            "{}{}: {}",
            " ".repeat(indent * INDENT_SIZE),
            self.name(),
            self.value
        )
    }

    fn handle(&self, input: &str) {
        match input::word(input) {
            "" => self.print(),
            _ => {}
        };
    }
}

struct TrackerCollection<'a> {
    name: String,
    children: Vec<&'a dyn TrackerNode>,
}

impl<'a> TrackerNode for TrackerCollection<'a> {
    fn name(&self) -> &str {
        &self.name
    }

    fn children(&self) -> &[&dyn TrackerNode] {
        &self.children
    }

    fn format(&self, indent: usize) -> String {
        let mut ret = format!("{}{}:", " ".repeat(indent * INDENT_SIZE), self.name());
        for tracker in &self.children {
            ret.push('\n');
            ret.push_str(&tracker.format(indent + 1));
        }
        ret
    }

    fn handle(&self, input: &str) {
        match input::word(input) {
            "" => self.print(),
            _ => {}
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_format_tracker() {
        assert_eq!(
            Tracker {
                name: "tracker".to_string(),
                value: 42
            }
            .format(2),
            "        tracker: 42"
        );
    }

    #[test]
    fn test_format_collection() {
        assert_eq!(
            TrackerCollection {
                name: "collection".to_string(),
                children: vec![
                    &Tracker {
                        name: "tracker1".to_string(),
                        value: 1
                    },
                    &Tracker {
                        name: "tracker2".to_string(),
                        value: 2
                    }
                ]
            }
            .format(1),
            "    collection:\n        tracker1: 1\n        tracker2: 2"
        );
    }
}
