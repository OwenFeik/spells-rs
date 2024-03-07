#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    Assign(usize, usize),
    Call(String, Vec<usize>),
    Add(usize, usize),
    Sub(usize, usize),
    Mul(usize, usize),
    Div(usize, usize),
    Exp(usize, usize),
    Neg(usize),
    Adv(usize),
    DisAdv(usize),
    Sort(usize),
    Keep(usize, usize),
    Roll(u32, u32),
    Natural(u32),
    Identifier(String),
}

impl Node {
    fn renumber_binary(&self, lhs: usize, rhs: usize) -> Self {
        match self {
            Node::Assign(_, _) => Node::Assign(lhs, rhs),
            Node::Add(_, _) => Node::Add(lhs, rhs),
            Node::Sub(_, _) => Node::Sub(lhs, rhs),
            Node::Mul(_, _) => Node::Mul(lhs, rhs),
            Node::Div(_, _) => Node::Div(lhs, rhs),
            Node::Exp(_, _) => Node::Exp(lhs, rhs),
            Node::Keep(_, _) => Node::Keep(lhs, rhs),
            _ => self.clone(),
        }
    }

    fn renumber_unary(&self, arg: usize) -> Self {
        match self {
            Node::Neg(_) => Node::Neg(arg),
            Node::Adv(_) => Node::Adv(arg),
            Node::DisAdv(_) => Node::DisAdv(arg),
            Node::Sort(_) => Node::Sort(arg),
            _ => self.clone(),
        }
    }

    fn copy(&self, from: &Ast, to: &mut Ast) -> Option<usize> {
        match self {
            &Node::Assign(lhs, rhs)
            | &Node::Add(lhs, rhs)
            | &Node::Sub(lhs, rhs)
            | &Node::Mul(lhs, rhs)
            | &Node::Div(lhs, rhs)
            | &Node::Exp(lhs, rhs)
            | &Node::Keep(lhs, rhs) => {
                let lhs = from.get(lhs)?.copy(from, to)?;
                let rhs = from.get(rhs)?.copy(from, to)?;
                Some(to.add(self.renumber_binary(lhs, rhs)))
            }
            &Node::Neg(arg) | &Node::Adv(arg) | &Node::DisAdv(arg) | &Node::Sort(arg) => {
                let arg = from.get(arg)?.copy(from, to)?;
                Some(to.add(self.renumber_unary(arg)))
            }
            Node::Call(name, args) => {
                let mut new_args = Vec::new();
                for &arg in args {
                    new_args.push(from.get(arg)?.copy(from, to)?);
                }
                Some(to.add(Node::Call(name.clone(), new_args)))
            }
            Node::Roll(_, _) | Node::Natural(_) | Node::Identifier(_) => Some(to.add(self.clone())),
        }
    }
}

#[derive(Debug)]
pub struct Ast(Vec<Node>);

impl Ast {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, expr: Node) -> usize {
        self.0.push(expr);
        self.0.len() - 1
    }

    pub fn get(&self, expr: usize) -> Option<&Node> {
        self.0.get(expr)
    }

    pub fn start(&self) -> usize {
        if self.0.is_empty() {
            0
        } else {
            self.0.len() - 1
        }
    }

    pub fn subtree(&self, root: usize) -> Option<Ast> {
        let mut subtree = Ast::new();
        self.get(root)?.copy(self, &mut subtree);
        Some(subtree)
    }

    #[cfg(test)]
    pub fn exprs(&self) -> &[Node] {
        &self.0
    }

    pub fn render(&self) -> String {
        self._render(self.start())
    }

    fn _render(&self, id: usize) -> String {
        if let Some(node) = self.get(id) {
            match node {
                &Node::Assign(lhs, rhs) => {
                    format!("{} = {}", self._render(lhs), self._render(rhs))
                }
                &Node::Add(lhs, rhs) => format!("{} + {}", self._render(lhs), self._render(rhs)),
                &Node::Sub(lhs, rhs) => format!("{} - {}", self._render(lhs), self._render(rhs)),
                &Node::Mul(lhs, rhs) => format!("{} * {}", self._render(lhs), self._render(rhs)),
                &Node::Div(lhs, rhs) => format!("{} / {}", self._render(lhs), self._render(rhs)),
                &Node::Exp(lhs, rhs) => format!("{} ^ {}", self._render(lhs), self._render(rhs)),
                &Node::Neg(arg) => format!("-{}", self._render(arg)),
                &Node::Adv(arg) => format!("{}a", self._render(arg)),
                &Node::DisAdv(arg) => format!("{}d", self._render(arg)),
                &Node::Sort(arg) => format!("{}s", self._render(arg)),
                &Node::Keep(lhs, rhs) => format!("{}k{}", self._render(lhs), self._render(rhs)),
                &Node::Roll(q, d) => {
                    if q == 1 {
                        format!("d{d}")
                    } else {
                        format!("{q}d{d}")
                    }
                }
                &Node::Natural(n) => n.to_string(),
                Node::Identifier(name) => name.clone(),
                Node::Call(name, args) => {
                    format!(
                        "{name}({})",
                        args.iter().fold(String::new(), |mut acc, el| {
                            if !acc.is_empty() {
                                acc.push_str(", ");
                            }
                            acc.push_str(&self._render(*el));
                            acc
                        })
                    )
                }
            }
        } else {
            "ERROR".to_string()
        }
    }
}
