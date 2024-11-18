use std::collections::HashMap;

use thiserror::Error;

use crate::pass::{FromNamedParsedValue, Pass, RenderPassError};

use super::ast::{Expr, Statement};

pub struct Interpreter {
    pub passes: HashMap<String, Box<dyn Pass>>,
    pub edges: HashMap<String, Vec<String>>,
    pub display: Option<String>,
    symbols: HashMap<String, ParsedValue>,
}

#[derive(Debug, Error)]
pub enum InterpreterError {
    #[error("undefined variable '{0}'")]
    UndefinedVariable(String),
    #[error("invalid pass '{0}'. expected struct, tuple struct, or unit struct")]
    InvalidPass(String),
    #[error("invalid pass assignment with left hand side '{0}', expected right hand side to be struct, tuple struct, or unit struct")]
    InvalidPassAssignment(String),
    #[error("multiple display calls")]
    MultipleDisplays,
    #[error(transparent)]
    RenderPass(#[from] RenderPassError),
}

#[derive(Debug, Clone)]
pub enum ParsedValue {
    Int(i32),
    Float(f32),
    Path(String),
    Bool(bool),
    UnitStruct(String),
    Struct {
        name: String,
        fields: HashMap<String, Box<ParsedValue>>,
    }
}

impl ParsedValue {
    pub fn struct_name(&self) -> Option<String> {
        match self {
            ParsedValue::UnitStruct(name) => Some(name.to_string()),
            ParsedValue::Struct { name, .. } => Some(name.to_string()),
            _ => None
        }
    }

    pub fn struct_properties(self) -> Option<(String, HashMap<String, Box<ParsedValue>>)> {
        match self {
            ParsedValue::UnitStruct(name) => Some((name, HashMap::new())),
            ParsedValue::Struct { name, fields } => Some((name, fields)),
            _ => None
        }
    }

    pub fn type_name(&self) -> String {
        match self {
            ParsedValue::Int(_) => "int",
            ParsedValue::Float(_) => "float",
            ParsedValue::Path(_) => "path",
            ParsedValue::Bool(_) => "bool",
            ParsedValue::UnitStruct(_) => "unit struct",
            ParsedValue::Struct { name, fields } => "struct",
        }.to_string()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            passes: HashMap::new(),
            edges: HashMap::new(),
            display: None,
            symbols: HashMap::new(),
        }
    }

    #[allow(clippy::vec_box)]
    pub fn run(&mut self, stmts: Vec<Box<Statement>>) -> Result<(), InterpreterError> {
        for stmt in stmts {
            self.run_stmt(*stmt)?;
        }

        Ok(())
    }

    fn run_stmt(&mut self, stmt: Statement) -> Result<(), InterpreterError> {
        match stmt {
            Statement::Assign { var, value: expr } => {
                let value = self.run_expr(*expr)?;
                self.symbols.insert(var, value);
            },
            Statement::Pass { name, value: expr } => {
                let value = self.run_expr(*expr)?;

                let Some(pass_name) = value.struct_name() else {
                    return Err(InterpreterError::InvalidPassAssignment(name));
                };

                let pass = <Box<dyn Pass>>::from_named_parsed_value(&pass_name, value)?;

                self.passes.insert(name, pass);
            },
            Statement::Edge { pass, dependencies } => {
                self.edges.insert(pass, dependencies);
            },
            Statement::Display { pass } => {
                if self.display.is_some() {
                    return Err(InterpreterError::MultipleDisplays);
                };

                self.display = Some(pass);
            },
        }

        Ok(())
    }

    fn run_expr(&mut self, expr: Expr) -> Result<ParsedValue, InterpreterError> {
        match expr {
            Expr::Int(v) => Ok(ParsedValue::Int(v)),
            Expr::Float(v) => Ok(ParsedValue::Float(v)),
            Expr::Path(p) => Ok(ParsedValue::Path(p)),
            Expr::VarAccess(var) => {
                match self.symbols.get(&var) {
                    Some(v) => Ok(v.clone()),
                    None => Err(InterpreterError::UndefinedVariable(var)),
                }
            },
            Expr::Ident(ident) => {
                match ident.as_str() {
                    "true" => Ok(ParsedValue::Bool(true)),
                    "false" => Ok(ParsedValue::Bool(false)),
                    _ => Ok(ParsedValue::UnitStruct(ident)),
                }
            },
            Expr::Struct { name, fields } => {
                let mut field_values = HashMap::new();

                for field in fields {
                    let value = self.run_expr(*field.value)?;
                    field_values.insert(field.ident, Box::new(value));
                }

                Ok(ParsedValue::Struct {
                    name,
                    fields: field_values,
                })
            },
        }
    }
}

pub trait FromParsedValue: Sized {
    fn from_parsed_value(value: ParsedValue) -> Result<Self, ParseValueError>;
}

#[derive(Debug, Error)]
pub enum ParseValueError {
    #[error("incorrect type. expected {0} but got {1}")]
    WrongType(String, String),
    #[error("duplicate field '{0}'")]
    DuplicateField(String),
    #[error("unknown field '{0}'")]
    UnknownField(String),
    #[error("missing required field '{0}'")]
    MissingField(String),
    #[error("unknown enum variant '{0}'")]
    UnknownVariant(String),
}
