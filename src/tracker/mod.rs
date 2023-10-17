use std::fmt::Display;

use crate::input;

const INDENT_SIZE: usize = 4;

enum TrackerInner {
    Tracker(i32),
    Collection(Vec<Tracker>),
}

pub struct Tracker {
    name: String,
    inner: TrackerInner,
}

impl Tracker {
    fn make(name: &str, inner: TrackerInner) -> Self {
        Self {
            name: name.to_string(),
            inner,
        }
    }

    pub fn new(name: &str) -> Self {
        Self::make(name, TrackerInner::Tracker(0))
    }

    pub fn add(&mut self, child: Tracker) -> String {
        if let TrackerInner::Collection(children) = &mut self.inner {
            let child_name = child.name.clone();
            children.push(child);
            format!("Added new tracker {child_name} to {}.", self.name())
        } else {
            format!("Can't add child to non-collection tracker {}.", self.name())
        }
    }

    pub fn collection(name: &str) -> Self {
        Self::make(name, TrackerInner::Collection(Vec::new()))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn child(&self, name: &str) -> Option<&Tracker> {
        self.children().iter().find(|c| c.name() == name)
    }

    fn children(&self) -> &[Tracker] {
        match &self.inner {
            TrackerInner::Tracker(_) => &[],
            TrackerInner::Collection(children) => children,
        }
    }

    fn format(&self, indent: usize) -> String {
        let heading = format!("{}{}:", " ".repeat(indent * INDENT_SIZE), self.name());
        match &self.inner {
            TrackerInner::Tracker(value) => format!("{heading} {value}"),
            TrackerInner::Collection(children) => children
                .iter()
                .map(|c| c.format(indent + 1))
                .fold(heading, |mut s, c| {
                    s.push('\n');
                    s.push_str(&c);
                    s
                }),
        }
    }

    fn handle(&self, input: &str) {
        match input::command(input) {
            "" => println!("{self}"),
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
            Tracker {
                name: "tracker".to_string(),
                inner: TrackerInner::Tracker(42),
            }
            .format(2),
            "        tracker: 42"
        );
    }

    #[test]
    fn test_format_collection() {
        assert_eq!(
            Tracker {
                name: "collection".to_string(),
                inner: TrackerInner::Collection(vec![
                    Tracker {
                        name: "tracker1".to_string(),
                        inner: TrackerInner::Tracker(1)
                    },
                    Tracker {
                        name: "tracker2".to_string(),
                        inner: TrackerInner::Tracker(2)
                    }
                ])
            }
            .format(1),
            "    collection:\n        tracker1: 1\n        tracker2: 2"
        );
    }
}
