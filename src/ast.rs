use crate::{operator::Operator, value::Value};

#[derive(Debug, PartialEq)]
pub enum Node {
    Value(Value),
    Identifier(String),
    List(Vec<usize>),
    Call(String, Vec<usize>),
    Binary(usize, Operator, usize),
    Unary(usize, Operator),
    If(usize, usize, Option<usize>), // Condition, block if true, optional else.
}

impl Node {
    fn copy(&self, from: &Ast, to: &mut Ast) -> Option<usize> {
        match self {
            Node::Value(val) => Some(to.add(Self::Value(val.clone()))),
            Node::Identifier(name) => Some(to.add(Self::Identifier(name.clone()))),
            Node::List(values) => {
                let mut new_vals = Vec::new();
                for &val in values {
                    new_vals.push(from.get(val)?.copy(from, to)?);
                }
                Some(to.add(Self::List(new_vals)))
            }
            Node::Call(name, args) => {
                let mut new_args = Vec::new();
                for &arg in args {
                    new_args.push(from.get(arg)?.copy(from, to)?);
                }
                Some(to.add(Node::Call(name.clone(), new_args)))
            }
            &Node::Binary(lhs, op, rhs) => {
                let lhs = from.get(lhs)?.copy(from, to)?;
                let rhs = from.get(rhs)?.copy(from, to)?;
                Some(to.add(Self::Binary(lhs, op, rhs)))
            }
            &Node::Unary(arg, op) => {
                let arg = from.get(arg)?.copy(from, to)?;
                Some(to.add(Self::Unary(arg, op)))
            }
            &Node::If(cond, expr, fail) => {
                let cond = from.get(cond)?.copy(from, to)?;
                let expr = from.get(expr)?.copy(from, to)?;
                let fail = fail
                    .and_then(|n| from.get(n))
                    .and_then(|n| n.copy(from, to));
                Some(to.add(Self::If(cond, expr, fail)))
            }
        }
    }

    #[cfg(test)]
    pub fn name<S: ToString>(name: S) -> Self {
        Self::Identifier(name.to_string())
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
                Node::Value(Value::Outcome(oc)) => format!("{}", oc.roll),
                Node::Value(Value::Empty) => "ERROR".to_string(),
                Node::Value(v) => format!("{v}"),
                Node::Identifier(name) => name.clone(),
                Node::List(values) => {
                    format!(
                        "[{}]",
                        values.iter().fold(String::new(), |mut acc, el| {
                            if !acc.is_empty() {
                                acc.push_str(", ");
                            }
                            acc.push_str(&self._render(*el));
                            acc
                        })
                    )
                }
                &Node::Binary(lhs, op, rhs) => {
                    format!("{} {} {}", self._render(lhs), op.char(), self._render(rhs))
                }
                &Node::Unary(arg, op) => {
                    let arg = self._render(arg);
                    if op.is_unary_postfix() {
                        format!("{}{}", arg, op.char())
                    } else {
                        format!("{}{}", op.char(), arg)
                    }
                }
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
                &Node::If(cond, expr, fail) => {
                    if let Some(node) = fail {
                        format!(
                            "if ({}) then ({}) else ({})",
                            self._render(cond),
                            self._render(expr),
                            self._render(node)
                        )
                    } else {
                        format!(
                            "if ({}) then ({})",
                            self._render(cond),
                            self._render(expr)
                        )
                    }
                }
            }
        } else {
            "ERROR".to_string()
        }
    }
}
