use super::{ArrowStyle, Connector, Direction, Flowchart, LineStyle, Node, NodeStyle};
use anyhow::{anyhow, Result};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, multispace0, space0},
    combinator::{opt, value},
    multi::many1_count,
    Finish, IResult,
};

struct ParseCtx<'input> {
    left_node_scratch: Vec<Node<'input>>,
    right_node_scratch: Vec<Node<'input>>,
}

impl<'input> ParseCtx<'input> {
    fn new() -> Self {
        Self {
            left_node_scratch: vec![],
            right_node_scratch: vec![],
        }
    }
    fn scratches(&mut self) -> (&mut Vec<Node<'input>>, &mut Vec<Node<'input>>) {
        (&mut self.left_node_scratch, &mut self.right_node_scratch)
    }
}

pub fn parse_flowchart(input: &str) -> Result<Flowchart<'_>, nom::error::Error<&str>> {
    let mut ctx = ParseCtx::new();
    let (_, chart) = flowchart(&mut ctx, input).finish()?;
    Ok(chart)
}

// inner parse_flowchart
fn flowchart<'input>(
    ctx: &mut ParseCtx<'input>,
    i: &'input str,
) -> IResult<&'input str, Flowchart<'input>> {
    let (i, _) = multispace0(i)?;
    let (i, _) = flowchart_tok(i)?;
    let (i, _) = ws(i)?;
    let (i, direction) = direction(i)?;

    let mut flow = Flowchart::new(direction);
    for line in i.lines() {
        let line = line.trim();
        if !line.is_empty() {
            parse_line(ctx, line, &mut flow)?;
        }
    }
    Ok((i, flow))
}

/// Parse the flowchart token
fn flowchart_tok(i: &str) -> IResult<&str, &str> {
    tag("flowchart")(i)
}

/// Parse the flowchart direction
fn direction(i: &str) -> IResult<&str, Direction> {
    alt((
        value(Direction::TopBottom, alt((tag("TB"), tag("TD")))),
        value(Direction::BottomTop, tag("BT")),
        value(Direction::RightLeft, tag("RL")),
        value(Direction::LeftRight, tag("LR")),
    ))(i)
}

/// Parse a line of the source input.
///
/// A line can have more than one connection in it. The line should already have been trimmed
/// before calling this function.
fn parse_line<'input>(
    ctx: &mut ParseCtx<'input>,
    i: &'input str,
    flow: &mut Flowchart<'input>,
) -> IResult<&'input str, ()> {
    let (left_scratch, right_scratch) = ctx.scratches();

    // first connection
    let (i, left_nodes) = node_list(left_scratch, i)?;
    let (i, _) = ws(i)?;
    let (i, conn) = connector(i)?;
    let (i, _) = ws(i)?;
    let (i, right_nodes) = node_list(right_scratch, i)?;
    let (mut i_outer, _) = ws(i)?;
    for node in left_nodes {
        flow.add_node(node);
    }
    for node in right_nodes {
        flow.add_node(node);
    }
    for left in left_nodes {
        for right in right_nodes {
            flow.add_edge(left.id, right.id, conn);
        }
    }

    // 2nd+ connections (optional)
    while !i_outer.is_empty() {
        // TODO we could avoid this copy by just switching which of the two vecs we consider the
        // left one.
        std::mem::swap(left_scratch, right_scratch);
        // The next line took the `&mut *` dance to convince the borrow checker (&mut isn't Copy,
        // so we need to reborrow).
        let left_nodes = &mut *left_scratch;
        let (i, conn) = connector(i_outer)?;
        let (i, _) = ws(i)?;
        let (i, right_nodes) = node_list(right_scratch, i)?;
        let (i, _) = ws(i)?;

        i_outer = i;
        for node in right_nodes {
            flow.add_node(node);
        }
        for left in left_nodes {
            for right in right_nodes {
                flow.add_edge(left.id, right.id, conn);
            }
        }
    }

    Ok((i_outer, ()))
}

/// Parse a list of 1 or more nodes separated by `'&'`.
fn node_list<'input, 'ctx>(
    nodes: &'ctx mut Vec<Node<'input>>,
    i: &'input str,
) -> IResult<&'input str, &'ctx [Node<'input>]> {
    nodes.clear();
    let (i, first) = node(i)?;
    nodes.push(first);
    let (mut i_outer, _) = ws(i)?;
    while matches!(i_outer.chars().next(), Some('&')) {
        let (i, _) = tag("&")(i_outer)?;
        let (i, _) = ws(i)?;
        let (i, node) = node(i)?;
        nodes.push(node);
        let (i, _) = ws(i)?;

        i_outer = i;
    }
    Ok((i_outer, nodes))
}

/// Parse a node
fn node(i: &str) -> IResult<&str, Node> {
    let (i, id) = ident(i)?;
    let (i, _) = ws(i)?;
    let (i, style_start) = opt(node_style_start)(i)?;
    let style_start = match style_start {
        Some(v) => v,
        None => {
            return Ok((
                i,
                Node {
                    id,
                    label: "",
                    style: NodeStyle::Square,
                },
            ))
        }
    };
    let (i, _) = ws(i)?;
    let (i, label, style) = if matches!(i.chars().next(), Some('"')) {
        // quoted label
        let (i, label) = node_label_quoted(i)?;
        let (i, _) = ws(i)?;
        let (i, style) = node_style_end(style_start)(i)?;
        (i, label, style)
    } else {
        let (i, (label, style)) = node_label_unquoted(style_start, i)?;
        (i, label, style)
    };
    Ok((i, Node { id, label, style }))
}

fn node_style_start(i: &str) -> IResult<&str, &str> {
    // TODO check order (longer before shorter)
    alt((
        tag("((("),
        tag("(["),
        tag("[["),
        tag("[("),
        tag("(("),
        tag("{{"),
        tag("[/"),
        tag(r"[\"),
        tag("["),
        tag("("),
        tag(">"),
        tag("{"),
    ))(i)
}

fn node_style_end<'a>(start: &str) -> impl FnMut(&'a str) -> IResult<&'a str, NodeStyle> {
    // TODO check order (longer before shorter)
    match start {
        "[" => match_end_tester(&[("]", NodeStyle::Square)]),
        "(" => match_end_tester(&[(")", NodeStyle::Round)]),
        "([" => match_end_tester(&[("])", NodeStyle::Stadium)]),
        "[[" => match_end_tester(&[("]]", NodeStyle::Subroutine)]),
        "[(" => match_end_tester(&[(")]", NodeStyle::Cylinder)]),
        "((" => match_end_tester(&[("))", NodeStyle::Circle)]),
        ">" => match_end_tester(&[("]", NodeStyle::Asymmetric)]),
        "{" => match_end_tester(&[("}", NodeStyle::Rhombus)]),
        "{{" => match_end_tester(&[("}}", NodeStyle::Hexagon)]),
        "[/" => match_end_tester(&[
            ("/]", NodeStyle::Parallelogram),
            ("\\]", NodeStyle::Trapezoid),
        ]),
        "[\\" => match_end_tester(&[
            ("\\]", NodeStyle::ParallelogramRev),
            ("/]", NodeStyle::TrapezoidRev),
        ]),
        "(((" => match_end_tester(&[(")))", NodeStyle::DoubleCircle)]),
        _ => unreachable!(),
    }
}

fn match_end_tester<'a>(
    tests: &'static [(&'static str, NodeStyle)],
) -> impl Fn(&'a str) -> IResult<&'a str, NodeStyle> {
    move |input| {
        for (test, style) in tests {
            if input.starts_with(test) {
                return Ok((&input[test.len()..], *style));
            }
        }
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )))
    }
}

fn node_label_quoted(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag("\"")(i)?;
    let mut iter = i.splitn(2, '"');
    let inner = iter.next().expect("unreachable");
    let i = iter.next().expect("TODO error handling");
    Ok((i, inner))
}

fn node_label_unquoted<'input>(
    style_start: &str,
    i: &'input str,
) -> IResult<&'input str, (&'input str, NodeStyle)> {
    // I haven't done this using nom because honestly I don't know how to (without allocating a vec
    // using many0)
    let end_test = node_style_end(style_start);
    input_until(end_test)(i)
}

fn connector(i: &str) -> IResult<&str, Connector> {
    // The rules here are that if there is a starting arrow, then we take 1 off the calculated
    // rank, unless it is a dotted line, in which case there must be exactly 1 `-` either side of
    // the dots irrespective, and to get the rank we count the docs. So we split the two cases.
    //
    // TODO we don't handle labels mid-way thru yet.
    alt((connector_dotted, connector_solid))(i)
}

fn connector_dotted(i: &str) -> IResult<&str, Connector> {
    let (i, arrow_start) = opt(arrow(true))(i)?;
    let (i, _) = tag("-")(i)?;
    let (i, rank) = many1_count(tag("."))(i)?;
    let (i, _) = tag("-")(i)?;
    let (i, arrow_end) = opt(arrow(false))(i)?;
    let (i, _) = ws(i)?;
    Ok((
        i,
        Connector {
            line_style: LineStyle::Dotted,
            arrow_start,
            arrow_end,
            label: "",
            rank: rank.try_into().expect("rank must be <= 65535"),
        },
    ))
}

fn connector_solid(i: &str) -> IResult<&str, Connector> {
    let mut line_ty = LineTy::new();
    let (i, arrow_start) = opt(arrow(true))(i)?;

    // if no arrow, there is an extra line segment
    let i = if arrow_start.is_none() {
        let (i, style) = line(i)?;
        line_ty.set(style).expect("TODO error handling");
        i
    } else {
        i
    };

    // count the line segments (we don't use many1_count because we want to check consistent style)
    let (mut i, style) = line(i)?;
    line_ty.set(style).expect("TODO error handling");
    let mut rank = 1; // we already got one line segment
    while matches!(i.chars().next(), Some('=') | Some('-')) {
        let (i_n, style) = line(i)?;
        line_ty.set(style).expect("TODO error handling");
        i = i_n;
        rank += 1;
    }

    // end arrow
    let (i, arrow_end) = opt(arrow(false))(i)?;
    if arrow_end.is_none() {
        // if there is no arrow the last line segment does not count towards rank
        rank -= 1;
    }

    Ok((
        i,
        Connector {
            line_style: line_ty.get().expect("TODO error handling"),
            arrow_start,
            arrow_end,
            label: "",
            rank,
        },
    ))
}

/// An arrow character.
///
/// `start` is whether we are looking for a left-facing arrow (at the start of a line)
fn arrow(start: bool) -> impl FnMut(&str) -> IResult<&str, ArrowStyle> {
    move |i| {
        alt((
            value(ArrowStyle::Circle, tag("o")),
            value(ArrowStyle::Cross, tag("x")),
            value(ArrowStyle::Arrow, if start { tag("<") } else { tag(">") }),
        ))(i)
    }
}

/// A line character (either `=` or `-`)
fn line(i: &str) -> IResult<&str, LineStyle> {
    alt((
        value(LineStyle::Normal, tag("-")),
        value(LineStyle::Thick, tag("=")),
    ))(i)
}

/// A node identifier
fn ident(i: &str) -> IResult<&str, &str> {
    alphanumeric1(i)
}

/// Whitespace
fn ws(i: &str) -> IResult<&str, &str> {
    space0(i)
}

/// Utility for checking for consistent line style.
struct LineTy {
    ty: Option<LineStyle>,
}

impl LineTy {
    fn new() -> Self {
        LineTy { ty: None }
    }

    fn set(&mut self, ty: LineStyle) -> Result<()> {
        match self.ty.replace(ty) {
            Some(old_ty) if ty == old_ty => Ok(()),
            Some(_) => Err(anyhow!("mixed - and = in the same connection")),
            None => Ok(()),
        }
    }

    /// Get the line style
    ///
    /// Errors if the line style was never set.
    fn get(mut self) -> Result<LineStyle> {
        self.ty
            .take()
            .ok_or_else(|| anyhow!("line style was never set"))
    }
}

/// Keep trying `p` until we get a match, then return all the input before the match and the result
/// of the parse.
fn input_until<I: nom::InputLength + nom::InputTake, O, E>(
    mut p: impl nom::Parser<I, O, E>,
) -> impl FnMut(I) -> IResult<I, (I, O), E>
where
    I: nom::InputLength + nom::InputTake,
    E: nom::error::ParseError<I>,
{
    move |i| {
        let input_len = i.input_len();
        for offset in 0..input_len {
            let (i, taken) = i.take_split(offset);
            if let Ok((i, res)) = p.parse(i) {
                return Ok((i, (taken, res)));
            }
        }
        Err(nom::Err::Error(E::from_error_kind(
            i,
            nom::error::ErrorKind::TakeUntil,
        )))
    }
}
