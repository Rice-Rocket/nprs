use std::{collections::HashMap, io::Read};

use ast::Statement;
use interpreter::{Interpreter, InterpreterError, ParsedValue};
use thiserror::Error;
use lalrpop_util::lalrpop_mod;

use crate::{image::{pixel::rgba::Rgba, Image, ImageError}, pass::Pass, render_graph::{NodeId, RenderGraph}};

pub mod ast;
pub mod interpreter;
pub mod parse_primitives;
lalrpop_mod!(pub grammar, "/parser/grammar.rs");

pub struct RawRenderGraph {
    passes: HashMap<String, Box<dyn Pass>>,
    edges: HashMap<String, Vec<String>>,
    display: String
}

#[derive(Debug, Error)]
pub enum RenderGraphReadError {
    /// An IO error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// An interpreter error.
    #[error(transparent)]
    Interpreter(#[from] InterpreterError),
    /// Missing required display image.
    #[error("missing required display image")]
    MissingDisplay,
    /// Reference to undefined pass.
    #[error("reference to undefined pass '{0}'")]
    UndefinedPass(String),
    /// Duplicate pass name.
    #[error("duplicate pass name '{0}'")]
    DuplicateName(String),
}

impl RawRenderGraph {
    pub fn read<P: AsRef<std::path::Path>>(path: P) -> Result<RawRenderGraph, RenderGraphReadError> {
        let data = std::fs::read_to_string(path)?;
        let stmts: Vec<Box<Statement>> = grammar::StatementsParser::new()
            .parse(&data)
            .unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.run(stmts);

        let Some(display) = interpreter.display else {
            return Err(RenderGraphReadError::MissingDisplay);
        };


        Ok(RawRenderGraph {
            passes: interpreter.passes,
            edges: interpreter.edges,
            display,
        })
    }
    pub fn build(self, input: Image<4, f32, Rgba<f32>>) -> Result<(RenderGraph, NodeId), RenderGraphReadError> {
        let mut render_graph = RenderGraph::new(input);

        let mut nodes = HashMap::new();

        for (name, pass) in self.passes.into_iter() {
            if nodes.insert(name.clone(), render_graph.add_node(pass, &[])).is_some() {
                return Err(RenderGraphReadError::DuplicateName(name))
            }
        }

        for (name, id) in nodes.iter() {
            let Some(edges) = self.edges.get(name) else { continue };

            for edge in edges {
                if edge == "source" {
                    render_graph.add_edge(*id, NodeId::SOURCE);
                    continue;
                }

                let Some(edge_id) = nodes.get(edge) else {
                    return Err(RenderGraphReadError::UndefinedPass(edge.to_string()));
                };

                render_graph.add_edge(*id, *edge_id);
            }
        }

        let Some(display_node) = nodes.get(&self.display) else {
            return Err(RenderGraphReadError::UndefinedPass(self.display));
        };

        Ok((render_graph, *display_node))
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
    #[error(transparent)]
    Image(#[from] ImageError),
}
