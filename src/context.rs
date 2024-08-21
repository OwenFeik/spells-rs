use std::{collections::HashMap, fmt::Display, process::exit, rc::Rc, sync::atomic::AtomicUsize};

use crate::{
    ast::Ast,
    err,
    eval::{check_argument_count, evaluate},
    eval_tome,
    outcome::Outcome,
    value::Value,
    Res,
};

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

enum ScopeObject {
    Value(Value),
    Function(Function),
    Child(usize),
}

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

pub struct Context {
    scopes: Vec<Scope>,
    initialised: bool,
}

impl Context {
    const GLOBAL_SCOPE_INDEX: usize = 0;
    const USER_SCOPE_INDEX: usize = 1;

    fn new() -> Self {
        Self {
            scopes: vec![Scope::new(usize::MAX)],
            initialised: false,
        }
    }

    pub fn empty() -> Self {
        Self::new()
    }

    fn initialise_globals(&mut self) {
        if self.initialised {
            return;
        }

        // Add default globals.
        if let Err(e) = eval_tome(include_str!("tomes/default.tome"), self) {
            eprintln!("Failed to evaluate default.tome:\n{}", e);
            exit(1);
        }

        self.push_scope(); // Add user scope.
        self.initialised = true;
    }

    fn push_scope(&mut self) -> &mut Scope {
        self.scope.push(Scope::new());
        self.scope.last_mut().unwrap()
    }

    fn pop_scope(&mut self) {
        // Never pop user or global scope.
        if self.scope.len() > 2 {
            self.scope.pop();
        }
    }

    fn lookup(&self, scope: usize, name: &str) -> Option<&ScopeObject> {
        let scope = self.scopes.get(scope)?;
        scope
            .objects
            .get(name)
            .or_else(|| self.lookup(scope.parent, name))
    }

    pub fn get_variable(&self, scope: usize, name: &str) -> Option<&Value> {
        if let ScopeObject::Value(val) = self.lookup(scope, name)? {
            Some(val)
        } else {
            None
        }
    }

    pub fn set_variable<S: ToString>(&mut self, scope: usize, name: S, value: Value) {
        self.definition_scope()
            .variables
            .insert(name.to_string(), value);
    }

    fn get_function(&self, name: &str) -> Option<Rc<Function>> {
        self.scope
            .iter()
            .rev()
            .map(|scope| scope.functions.get(name).cloned())
            .find(Option::is_some)
            .flatten()
    }

    pub fn define_function<S: ToString>(&mut self, name: S, body: Ast, parameters: Vec<String>) {
        let function = Function::new(name, body, parameters);
        self.definition_scope()
            .functions
            .insert(function.name.clone(), Rc::new(function));
    }

    pub fn call(&mut self, name: &str, args: Vec<Value>) -> Res<Outcome> {
        if let Some(function) = self.get_function(name) {
            let scope = self.push_scope();
            check_argument_count(name, function.parameters.len(), &args)?;
            for (name, value) in function.parameters.iter().zip(args) {
                scope.variables.insert(name.clone(), value);
            }
            let ret = evaluate(&function.body, self);
            self.pop_scope();
            ret
        } else {
            crate::builtins::call(name, args)
        }
    }

    pub fn dump_to_string(&self) -> Res<String> {
        let mut ret = String::new();

        let Some(user_scope) = self.scope.last() else {
            return err("No scope available to dump to string.");
        };

        // Sort functions by definition order.
        let mut functions: Vec<&Rc<Function>> = user_scope.functions.values().collect();
        functions.sort_by(|a, b| (a.id).cmp(&b.id));
        for func in functions {
            ret += &format!("{func}\n");
        }

        for (k, v) in &user_scope.variables {
            ret += &format!("{k} = {v}\n");
        }
        Ok(ret)
    }

    pub fn load_from(&mut self, from: Context) -> Res<()> {
        let mut scopes = from.scope;
        if !scopes.is_empty() {
            *self.definition_scope() = scopes.swap_remove(scopes.len() - 1);
            Ok(())
        } else {
            err("Unable to load from empty context.")
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        let mut context = Self::new();
        context.initialise_globals();
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
        eval("func(x, y) = x + y", &mut context).unwrap();
        let func = context.get_function("func").unwrap();
        assert_eq!(func.body.render(), "x + y");
        assert_eq!(func.parameters, vec!["x".to_string(), "y".to_string()]);
    }
}
