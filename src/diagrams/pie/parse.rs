use super::{Datum, Pie};
use nom::{
    bytes::complete::{tag, take_until},
    character::complete::multispace0,
    combinator::opt,
    multi::many1,
    IResult,
};

// input is expected to be pre-trimmed
pub fn parse_pie(i: &str) -> IResult<&str, Pie<'_>> {
    let (i, (title, show_data)) = parse_header(i)?;
    let (i, data) = many1(parse_datum)(i)?;
    assert!(i.trim().is_empty(), "{:?}", i);
    Ok((
        i,
        Pie {
            title,
            show_data,
            data,
        },
    ))
}

fn parse_header(i: &str) -> IResult<&str, (Option<&str>, bool)> {
    let (i, _) = tag("pie")(i)?;
    let (i, _) = ws(i)?;
    let (i, show_data) = opt(tag("showData"))(i)?;
    let (i, _) = ws(i)?;
    let (i, title) = opt(parse_title)(i)?;
    Ok((i, (title.map(|s| s.trim()), show_data.is_some())))
}

/// Parses "title The title" into 'The title'.
fn parse_title(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag("title")(i)?;
    take_until("\"")(i)
}

fn parse_datum(i: &str) -> IResult<&str, Datum> {
    let (i, _) = ws(i)?;
    let (i, _) = tag("\"")(i)?;
    let (i, label) = take_until("\"")(i)?;
    let (i, _) = tag("\"")(i)?;
    let (i, _) = ws(i)?;
    let (i, _) = tag(":")(i)?;
    let (i, _) = ws(i)?;
    let (i, value) = nom::number::complete::recognize_float(i)?;
    Ok((
        i,
        Datum {
            label,
            value: value.parse().unwrap(),
        },
    ))
}

/// Whitespace
fn ws(i: &str) -> IResult<&str, &str> {
    multispace0(i)
}
