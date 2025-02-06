use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{
    ast::Stmt,
    environment::Environment,
    interpreter::{execute_block, InterpreterEnvironment, InterpreterSignal, RuntimeError},
    token::{LiteralType, Token},
};

pub trait CallableTrait {
    fn arity(&self) -> u8;
    fn call(
        &self,
        args: &[LiteralType],
        env: &InterpreterEnvironment,
    ) -> Result<LiteralType, InterpreterSignal>;
}

#[derive(Debug, Clone)]
pub enum Callable {
    Function {
        name: Box<Token>,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
    },
    NativeFunction(NativeFunction),
}

impl CallableTrait for Callable {
    fn arity(&self) -> u8 {
        match self {
            Callable::Function { params, .. } => params.len() as u8,
            Callable::NativeFunction(native_function) => native_function.arity,
        }
    }

    fn call(
        &self,
        args: &[LiteralType],
        env: &InterpreterEnvironment,
    ) -> Result<LiteralType, InterpreterSignal> {
        match self {
            Callable::Function {
                name: _,
                params,
                body,
                closure,
            } => {
                let environment = Rc::new(RefCell::new(Environment::with_enclosing(closure)));

                for (param, arg) in params.iter().zip(args) {
                    environment
                        .borrow_mut()
                        .define(&param.lexeme, Some(arg.clone()));
                }

                let environment = InterpreterEnvironment {
                    globals: Rc::clone(&env.globals),
                    environment,
                };

                match execute_block(body, &environment) {
                    Err(InterpreterSignal::Return(v)) => Ok(v),
                    v => v.map(|_| LiteralType::Nil),
                }
            }
            Callable::NativeFunction(native_function) => (native_function.call_impl)(args),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NativeFunction {
    name: String,
    arity: u8,
    call_impl: fn(&[LiteralType]) -> Result<LiteralType, InterpreterSignal>,
}

impl NativeFunction {
    pub fn new(
        name: String,
        arity: u8,
        call_impl: fn(&[LiteralType]) -> Result<LiteralType, InterpreterSignal>,
    ) -> Self {
        Self {
            name,
            arity,
            call_impl,
        }
    }
}

impl Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Callable::Function { name, .. } => {
                write!(f, "{}", name.lexeme)
            }
            Callable::NativeFunction(native_function) => {
                write!(f, "{}", native_function.name)
            }
        }
    }
}
