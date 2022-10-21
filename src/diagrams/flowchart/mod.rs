mod lexer;
lalrpop_mod!(grammar, "/diagrams/flowchart/grammar.rs");

use crate::parser_utils::Lexer;
pub use lexer::ParseError;

#[derive(Debug)]
pub struct Flowchart<'input> {
    dir: Direction,
    spec: Vec<Statement<'input>>,
}

impl<'input> Flowchart<'input> {
    pub fn parse(input: &'input str) -> Result<Self, lexer::ParseError> {
        grammar::FlowchartParser::new()
            .parse(input, Lexer::new(input))
            .map_err(|e| {
                eprintln!("{}", e);
                ParseError
            })
    }

    pub fn dir(&self) -> Direction {
        self.dir
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    TopDown,
    LeftRight,
}

#[derive(Debug, Copy, Clone)]
pub struct Statement<'input> {
    left: Node<'input>,
    right: Node<'input>,
    connection: Connection<'input>,
}

#[derive(Debug, Copy, Clone)]
pub struct Node<'input> {
    label: &'input str,
    style: (),
}

#[derive(Debug, Copy, Clone)]
pub struct Connection<'input> {
    line_style: LineStyle,
    arrow_start: ArrowStyle,
    arrow_end: ArrowStyle,
    label: &'input str,
}

#[derive(Debug, Copy, Clone)]
pub enum LineStyle {
    Normal,
    Thick,
    Dotted,
}

#[derive(Debug, Copy, Clone)]
pub enum ArrowStyle {
    None,
    Arrow,
    Circle,
    Cross,
}
