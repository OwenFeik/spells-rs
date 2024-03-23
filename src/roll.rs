use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Roll {
    pub quantity: u32,
    pub die: u32,
    pub advantage: bool,
    pub disadvantage: bool,
}

impl Roll {
    pub fn new(quantity: u32, die: u32) -> Self {
        Roll {
            quantity,
            die,
            advantage: false,
            disadvantage: false,
        }
    }
}

impl Display for Roll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let quantity = if self.quantity == 1 {
            "".to_string()
        } else {
            self.quantity.to_string()
        };

        let die = self.die;

        let advstr = if self.advantage && !self.disadvantage {
            "a"
        } else if self.disadvantage && !self.advantage {
            "d"
        } else {
            ""
        };

        write!(f, "{quantity}d{die}{advstr}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RollOutcome {
    pub roll: Roll,
    pub rolls: Vec<u32>,
    pub result: u32,
}

impl Display for RollOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\tRolls: \t{}\tTotal: {}",
            self.roll,
            self.rolls
                .iter()
                .map(u32::to_string)
                .collect::<Vec<String>>()
                .join(", "),
            self.result
        )
    }
}
