use logos::Logos;
use std::fmt;

// format required for lalrpop
pub type Spanned<Token> = Result<(usize, Token, usize), LexError>;

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

pub struct Lexer<'input, Token: Logos<'input>> {
    inner: logos::Lexer<'input, Token>,
}

impl<'input, Token> Lexer<'input, Token>
where
    Token: Logos<'input, Source = str>,
    Token::Extras: Default,
{
    // you could use Logos' `Extras` option if you want. Also str can be [u8] if you need to use
    // non-unicode tokens.
    pub fn new(source: &'input str) -> Self {
        Lexer {
            inner: logos::Lexer::new(source),
        }
    }
}

impl<'input, Token> Iterator for Lexer<'input, Token>
where
    Token: Logos<'input, Source = str> + PartialEq, // + fmt::Debug,
{
    type Item = Spanned<Token>;
    fn next(&mut self) -> Option<Self::Item> {
        let tok = self.inner.next()?;
        if tok == Logos::ERROR {
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
