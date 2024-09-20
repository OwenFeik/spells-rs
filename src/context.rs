use std::{collections::HashMap, fmt::Display, rc::Rc, sync::atomic::AtomicUsize};

use crate::{
    ast::Ast,
    eval::{check_argument_count, evaluate},
    eval_tome,
    outcome::Outcome,
    value::Value,
    Res,
};

#[derive(Debug)]
struct Function {
    name: String,
    body: Ast,
    parameters: Vec<String>,

    /// Unique ID of this function. This keeps track of declaration order, which
    /// is important because when we are saving defined functions, we need to
    /// ensure that all functions used within a function are available in the
    /// scope the function is evaluated in.
    id: usize,
}

impl Function {
    fn new<S: ToString>(name: S, body: Ast, parameters: Vec<String>) -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
        Self {
            name: name.to_string(),
            body,
            parameters,
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({}) = {}",
            &self.name,
            self.parameters.join(", "),
            self.body.render()
        )
    }
}

#[derive(Debug)]
enum ScopeObject {
    Value(Value),
    Function(Rc<Function>),
    Child(usize),
}

#[derive(Debug)]
struct Scope {
    parent: usize,
    objects: HashMap<String, ScopeObject>,
}

impl Scope {
    fn new(parent: usize) -> Self {
        Self {
            parent,
            objects: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct Context {
    scopes: Vec<Scope>,
}

impl Context {
    pub const GLOBAL_SCOPE: usize = 0;

    fn new() -> Self {
        Self {
            scopes: vec![Scope::new(usize::MAX)],
        }
    }

    pub fn empty() -> Self {
        Self::new()
    }

    fn lookup(&self, scope: usize, name: &str) -> Option<&ScopeObject> {
        let scope = self.scopes.get(scope)?;
        scope
            .objects
            .get(name)
            .or_else(|| self.lookup(scope.parent, name))
    }

    fn child_scope(&mut self, parent: usize) -> usize {
        let scope = Scope::new(parent);
        let idx = self.scopes.len();
        self.scopes.push(scope);
        idx
    }

    fn scope_stack(&self, mut idx: usize) -> Vec<usize> {
        let mut stack = Vec::new();
        while idx != usize::MAX {
            if let Some(scope) = self.scopes.get(idx) {
                stack.push(idx);
                idx = scope.parent;
            } else {
                break;
            }
        }
        stack
    }

    pub fn get_variable(&self, scope: usize, name: &str) -> Option<&Value> {
        if let ScopeObject::Value(val) = self.lookup(scope, name)? {
            Some(val)
        } else {
            None
        }
    }

    pub fn get_global(&self, name: &str) -> Option<&Value> {
        self.get_variable(Self::GLOBAL_SCOPE, name)
    }

    pub fn set_variable<S: ToString>(&mut self, scope: usize, name: S, value: Value) {
        let name = name.to_string();
        let mut set_scope = scope;
        for idx in self.scope_stack(scope) {
            if let Some(scope) = self.scopes.get_mut(scope) {
                if scope.objects.contains_key(&name) {
                    set_scope = idx;
                    break;
                }
            }
        }

        self.scopes
            .get_mut(set_scope)
            .expect("Attempted to set variable in scope which doesn't exist.")
            .objects
            .insert(name.to_string(), ScopeObject::Value(value));
    }

    fn get_function(&self, scope: usize, name: &str) -> Option<Rc<Function>> {
        if let ScopeObject::Function(func) = self.lookup(scope, name)? {
            Some(func.clone())
        } else {
            None
        }
    }

    pub fn define_function<S: ToString>(
        &mut self,
        scope: usize,
        name: S,
        body: Ast,
        parameters: Vec<String>,
    ) {
        let function = Function::new(name.to_string(), body, parameters);
        self.scopes
            .get_mut(scope)
            .expect("Attempted to define function in scope that doesn't exist.")
            .objects
            .insert(name.to_string(), ScopeObject::Function(Rc::new(function)));
    }

    pub fn call(&mut self, scope: usize, name: &str, args: Vec<Value>) -> Res<Outcome> {
        if let Some(function) = self.get_function(scope, name) {
            let func_scope = self.child_scope(scope);
            check_argument_count(name, function.parameters.len(), &args)?;
            for (name, value) in function.parameters.iter().zip(args) {
                self.set_variable(func_scope, name, value);
            }
            let ret = evaluate(&function.body, self, func_scope);
            self.scopes.pop();
            ret
        } else {
            crate::builtins::call(name, args)
        }
    }

    pub fn dump_to_string(&self) -> Res<String> {
        let mut ret = String::new();

        // TODO establish module syntax.
        // let Some(user_scope) = self.scope.last() else {
        //     return err("No scope available to dump to string.");
        // };

        // // Sort functions by definition order.
        // let mut functions: Vec<&Rc<Function>> = user_scope.functions.values().collect();
        // functions.sort_by(|a, b| (a.id).cmp(&b.id));
        // for func in functions {
        //     ret += &format!("{func}\n");
        // }

        // for (k, v) in &user_scope.variables {
        //     ret += &format!("{k} = {v}\n");
        // }
        Ok(ret)
    }
}

impl Default for Context {
    fn default() -> Self {
        let mut context = Self::new();
        eval_tome(include_str!("tomes/default.tome"), &mut context).unwrap();
        context
    }
}

#[cfg(test)]
mod test {
    use crate::eval;

    use super::*;

    #[test]
    fn test_definition() {
        let mut context = Context::empty();
        eval("func(x, y) := x + y", &mut context).unwrap();
        let func = context.get_function(Context::GLOBAL_SCOPE, "func").unwrap();
        assert_eq!(func.body.render(), "x + y");
        assert_eq!(func.parameters, vec!["x".to_string(), "y".to_string()]);
    }
}
