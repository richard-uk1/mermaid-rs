// This is a template for using lalrpop with logos.
//
// You need to know how they both work to edit this template, but having it here can save some
// boilerplate work.
#[macro_use]
extern crate lalrpop_util;

use logos::{Logos, Span};
use std::fmt;

mod parser_utils;

lalrpop_mod!(lang);

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token<'input> {
    // Tokens can be literal strings, of any length.
    #[token("+")]
    Add,
    #[token("-")]
    Sub,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("=")]
    Equals,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token(";")]
    Semicolon,

    #[token("let")]
    Keyword,

    // Or regular expressions.
    #[regex("[a-zA-Z][a-zA-Z0-9]*")]
    Ident(&'input str),

    // A callback
    #[regex("[0-9]+", |s| s.slice().parse())]
    #[regex("[0-9]+k", kilo)]
    Number(i64),

    // Logos requires one token variant to handle errors,
    // it can be named anything you wish.
    #[error]
    // We can also use this variant to define whitespace,
    // or any other matches we wish to skip.
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

impl<'input> fmt::Display for Token<'input> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Add => write!(f, "+"),
            Token::Sub => write!(f, "-"),
            Token::Mul => write!(f, "*"),
            Token::Div => write!(f, "/"),
            Token::Equals => write!(f, "="),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Semicolon => write!(f, ";"),
            Token::Keyword => write!(f, "let"),
            Token::Ident(ident) => write!(f, "{}", ident),
            Token::Number(num) => write!(f, "{}", num),
            Token::Error => write!(f, "ERROR"),
        }
    }
}

// demonstrate data transforms as part of the lexer
fn kilo<'input>(lex: &mut logos::Lexer<'input, Token<'input>>) -> Option<i64> {
    let slice = lex.slice();
    // there's an unreachable panic here (assuming we only use this function for [0-9]+k
    // - hope the optimizer gets rid of it!
    let n: i64 = slice[..slice.len() - 1].parse().ok()?; // skip 'k'
    Some(n * 1_000)
}

#[derive(Debug)]
pub struct Statement {
    ident: Ident,
    value: i64,
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} = {}", self.ident.label, self.value)
    }
}

#[derive(Debug)]
pub struct Ident {
    label: String,
    #[allow(dead_code)]
    span: Span,
}

impl Ident {
    fn no_span(label: String) -> Self {
        Self { label, span: 0..0 }
    }
}

// format required for lalrpop
type Spanned<'input> = Result<(usize, Token<'input>, usize), LexError>;

pub struct LexError {
    token: String,
    span: logos::Span,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "lexer could not parse \"{}\" at offset {}-{}",
            self.token, self.span.start, self.span.end
        )
    }
}

impl fmt::Debug for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

struct Lexer<'input> {
    inner: logos::Lexer<'input, Token<'input>>,
}

impl<'input> Lexer<'input> {
    // you could use Logos' `Extras` option if you want. Also str can be [u8] if you need to use
    // non-unicode tokens.
    fn new(source: &'input str) -> Self {
        Lexer {
            inner: logos::Lexer::new(source),
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<'input>;
    fn next(&mut self) -> Option<Self::Item> {
        let tok = self.inner.next()?;
        if matches!(tok, Token::Error) {
            Some(Err(LexError {
                token: self.inner.slice().to_owned(),
                span: self.inner.span(),
            }))
        } else {
            let span = self.inner.span();
            Some(Ok((span.start, tok, span.end)))
        }
    }
}

fn main() {
    let input = "let x = 3 + 7;";
    let lexer = Lexer::new(input);

    let parsed = match lang::StatementParser::new().parse(input, lexer) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };
    println!("{}", parsed);
}
