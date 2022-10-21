use logos::Logos;
use std::fmt;
use thiserror::Error;

type Lexer<'input> = logos::Lexer<'input, Token<'input>>;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token<'input> {
    #[token("flowchart")]
    Flowchart,
    #[token("TD")]
    #[token("TB")]
    TopDown,
    #[token("LR")]
    LeftRight,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("<")]
    LArrow,
    #[token(">")]
    RArrow,
    #[token("/")]
    Slash,
    #[token("\\")]
    BSlash,
    #[token("&")]
    Amp,

    #[token("---")]
    Link,
    #[token("--")]
    LinkStart,
    #[token("-->")]
    LRArrow,

    #[regex("[a-zA-Z0-9]+")]
    Ident(&'input str),
    #[regex(r#""[^"]*""#, |lex| strip_input(1, 1)(lex))]
    #[regex(r#"\|[^|]*\|"#, |lex| strip_input(1, 1)(lex))]
    String(&'input str),

    #[error]
    // Skip whitespace
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

impl<'input> fmt::Display for Token<'input> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Flowchart => f.write_str("flowchart"),
            Token::TopDown => f.write_str("TD"),
            Token::LeftRight => f.write_str("LR"),

            Token::LParen => f.write_str("("),
            Token::RParen => f.write_str(")"),
            Token::LBracket => f.write_str("["),
            Token::RBracket => f.write_str("]"),
            Token::LBrace => f.write_str("{"),
            Token::RBrace => f.write_str("}"),
            Token::LArrow => f.write_str("<"),
            Token::RArrow => f.write_str(">"),
            Token::Slash => f.write_str("/"),
            Token::BSlash => f.write_str("\\"),
            Token::Amp => f.write_str("&"),

            Token::Link => f.write_str("---"),
            Token::LinkStart => f.write_str("--"),
            Token::LRArrow => f.write_str("-->"),
            Token::Ident(s) => f.write_str(s),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Error => f.write_str("ERROR TOKEN"),
        }
    }
}

#[derive(Debug, Error)]
#[error("parse error")]
pub struct ParseError;

/// Strip the first `start` and last `end` characters. Can use for e.g. `".."` and `'..'` strings.
fn strip_input<'input>(start: usize, end: usize) -> impl Fn(&mut Lexer<'input>) -> &'input str {
    move |lex| {
        let s = lex.slice();
        &s[start..s.len() - end]
    }
}
