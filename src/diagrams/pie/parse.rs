use super::{Datum, Pie};
use nom::{bytes::complete::take_until, character::complete::multispace0};
use nom_locate::LocatedSpan;
use std::fmt;

#[derive(Debug)]
pub struct Error {
    pub line: u32,
    pub col: usize,
    pub offset: usize,
    kind: ErrorKind,
}

impl Error {
    fn new(span: &Span<'_>, kind: ErrorKind) -> Self {
        Self {
            line: span.location_line(),
            col: span.get_column(),
            offset: span.location_offset(),
            kind,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "on line {}, col {}: {}", self.line, self.col, self.kind)
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    ExpectedLiteral(&'static str),
    ExpectedFloat(Option<String>),
    UnclosedQuote(&'static str),
    SearchLiteral(&'static str),
    UnexpectedTrailing,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::ExpectedLiteral(lit) => write!(f, "expected {:?}", lit),
            ErrorKind::ExpectedFloat(Some(inner)) => {
                write!(f, "couldn't parse number because {}", inner)
            }
            ErrorKind::ExpectedFloat(None) => write!(f, "expected a number"),
            ErrorKind::UnclosedQuote(lit) => {
                write!(f, "unclosed quoted string (expected {:?}, found EOF)", lit)
            }
            ErrorKind::SearchLiteral(lit) => write!(f, "ran out of input searching for {:?}", lit),
            ErrorKind::UnexpectedTrailing => write!(f, "unexpected trailing characters"),
        }
    }
}

type Span<'input> = LocatedSpan<&'input str>;
type IResult<'input, Out> = nom::IResult<Span<'input>, Out, Error>;

/// input is expected to be pre-trimmed
pub fn parse_pie(i: &str) -> IResult<Pie> {
    let i = LocatedSpan::new(i);
    let (i, _) = ws(i)?;
    let (mut i, (title, show_data)) = parse_header(i)?;
    let mut data = vec![];
    loop {
        let _tmp;
        (i, _tmp) = ws(i)?;
        if i.is_empty() {
            break;
        }
        let datum;
        (i, datum) = parse_datum(i)?;
        data.push(datum);
    }
    if !i.trim().is_empty() {
        // we will have tried to parse it above
        unreachable!()
    }
    Ok((
        i,
        Pie {
            title,
            show_data,
            data,
        },
    ))
}

fn parse_header(i: Span<'_>) -> IResult<(Option<&str>, bool)> {
    let (i, _) = tag("pie")(i)?;
    let (i, _) = ws(i)?;
    let (i, show_data) = opt(tag("showData"))(i)?;
    let (i, _) = ws(i)?;
    let (i, title) = opt(parse_title)(i)?;
    Ok((i, (title.map(|s| s.trim()), show_data.is_some())))
}

/// Parses "title The title" into 'The title'.
fn parse_title(i: Span) -> IResult<&str> {
    let (i, _) = tag("title")(i)?;
    let (i, title) = take_until("\"")(i).map_error(|_| ErrorKind::SearchLiteral("\""))?;
    Ok((i, title.fragment()))
}

/// Parse a data point.
///
/// Expect that whitespace has already been consumed.
fn parse_datum(i: Span) -> IResult<Datum> {
    let (i, label) = quoted(i)?;
    let (i, _) = ws(i)?;
    let (i, _) = tag(":")(i)?;
    let (i, _) = ws(i)?;
    let (i, value) = float(i)?;
    Ok((i, Datum { label, value }))
}

/// A string surrouded by double quotes (")
fn quoted(i: Span) -> IResult<&str> {
    let (i, _) = tag("\"")(i)?;
    let (i, label) = take_until("\"")(i).map_error(|_| ErrorKind::UnclosedQuote("\""))?;
    let (i, _) = tag("\"")(i)?;
    Ok((i, label.fragment()))
}

/// Whitespace using our error type
fn ws(i: Span) -> IResult<Span> {
    multispace0(i).map_err(|_: nom::Err<nom::error::Error<Span>>| unreachable!())
}

/// A version of `tag` that uses our error type.
fn tag(val: &'static str) -> impl Fn(Span<'_>) -> IResult<Span<'_>> {
    move |input| {
        nom::bytes::complete::tag(val)(input).map_error(|_| ErrorKind::ExpectedLiteral(val))
    }
}

/// A floating point number
fn float(i: Span) -> IResult<f64> {
    let (i, num) =
        nom::number::complete::recognize_float(i).map_error(|_| ErrorKind::ExpectedFloat(None))?;
    match num.parse::<f64>() {
        Ok(v) => Ok((i, v)),
        Err(e) => {
            let kind = ErrorKind::ExpectedFloat(Some(e.to_string()));
            Err(nom::Err::Error(Error::new(&num, kind)))
        }
    }
}

fn opt<'input, T, F: Fn(Span<'input>) -> IResult<'input, T>>(
    f: F,
) -> impl Fn(Span<'input>) -> IResult<'input, Option<T>> {
    move |input| match f(input) {
        Ok((i, out)) => Ok((i, Some(out))),
        Err(nom::Err::Error(_)) => Ok((input, None)),
        Err(other) => Err(other),
    }
}

/// Helper trait for mapping errors to our type.
trait MapErr {
    type Output;
    /// Given a way of getting the error kind, construct an error pointing at the current position.
    fn map_error(self, f: impl FnOnce(&nom::error::Error<Span<'_>>) -> ErrorKind) -> Self::Output;
}

impl<'a, T> MapErr for nom::IResult<Span<'a>, T> {
    type Output = IResult<'a, T>;
    fn map_error(self, f: impl FnOnce(&nom::error::Error<Span<'_>>) -> ErrorKind) -> Self::Output {
        self.map_err(|e| {
            e.map(|e| {
                let kind = f(&e);
                Error::new(&e.input, kind)
            })
        })
    }
}
