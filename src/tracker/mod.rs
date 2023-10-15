const INDENT_SIZE: usize = 4;

trait TrackerNode {
    fn name(&self) -> &str;

    fn children(&self) -> &[&dyn TrackerNode];

    fn print(&self, indent: usize);

    fn handle(&self, command: &str);
}

struct Tracker {
    name: String,
    value: u32,
}

impl TrackerNode for Tracker {
    fn name(&self) -> &str {
        &self.name
    }

    fn children(&self) -> &[&dyn TrackerNode] {
        &[]
    }

    fn print(&self, indent: usize) {
        println!(
            "{}{}: {}",
            " ".repeat(indent * INDENT_SIZE),
            self.name(),
            self.value
        );
    }

    fn handle(&self, command: &str) {
        let mut parts = command.trim().split(' ');
        match parts.next() {
            None => self.print(0),
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

    fn print(&self, indent: usize) {
        println!("{}{}:", " ".repeat(indent * INDENT_SIZE), self.name());
        for tracker in &self.children {
            tracker.print(indent + 1);
        }
    }

    fn handle(&self, command: &str) {
        let mut parts = command.trim().split(' ');
        match parts.next() {
            None => self.print(0),
            _ => {}
        };
    }
}
