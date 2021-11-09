use nom::{
    self,
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::anychar,
    combinator::{peek, value},
    multi::{many0, many_till},
    sequence::delimited,
    IResult,
};
use nom_locate::{position, LocatedSpan};

use crate::format::{Document, Hunk, HunkPos};

type Span<'a> = LocatedSpan<&'a [u8]>;

struct CodeBlock<'a> {
    info: Option<String>,
    _start: usize,
    end: usize,
    data: &'a [u8],
}

fn parse_header(s: Span) -> IResult<Span, String> {
    let (s, name) = delimited(tag("[testmark]:# ("), is_not(")"), tag(")"))(s)?;
    let (s, _) = take_until("\n")(s)?;
    let (s, _) = tag(b"\n")(s)?;
    let name = String::from_utf8_lossy(name.fragment()).to_string();
    Ok((s, name))
}

fn parse_code_block(s: Span) -> IResult<Span, CodeBlock> {
    let (s, start_pos) = position(s)?;
    let (s, _) = tag(b"```")(s)?;
    let (s, info) = take_until("\n")(s)?;
    let (s, _) = tag(b"\n")(s)?;
    let (s, data) = take_until("```")(s)?;
    let (s, _) = tag(b"```")(s)?;
    let (s, end_pos) = position(s)?;
    let info = String::from_utf8_lossy(info.fragment());
    let info = info.trim();
    let info = if info == "" {
        None
    } else {
        Some(info.to_string())
    };

    let c = CodeBlock {
        info,
        _start: start_pos.location_offset(),
        end: end_pos.location_offset(),
        data: data.fragment(),
    };
    Ok((s, c))
}

fn other_text(s: Span) -> IResult<Span, ()> {
    let (s, _) = many_till(anychar, peek(alt((tag("[testmark]:#"), tag("```")))))(s)?;
    Ok((s, ()))
}

fn parse_block(s: Span) -> IResult<Span, Option<Hunk>> {
    let (s, is_testmark) = peek(alt((
        value(true, tag("[testmark]:#")),
        value(false, tag("```")),
    )))(s)?;
    let (s, res) = if is_testmark {
        let (s, start_pos) = position(s)?;
        let (s, name) = parse_header(s)?;
        let (s, block) = parse_code_block(s)?;
        let h = Hunk::new(
            name,
            block.info,
            block.data.to_vec(),
            HunkPos::new(start_pos.location_offset(), block.end),
        );
        (s, Some(h))
    } else {
        let (s, _) = parse_code_block(s)?;
        (s, None)
    };

    Ok((s, res))
}

fn parse_next_section(s: Span) -> IResult<Span, Option<Hunk>> {
    let (s, _) = other_text(s)?;
    parse_block(s)
}

fn parse_doc(s: Span) -> IResult<Span, Vec<Hunk>> {
    let (s, hunks) = many0(parse_next_section)(s)?;
    let hunks = hunks.into_iter().filter_map(|h| h).collect();
    Ok((s, hunks))
}

pub fn create_document(contents: Vec<u8>) -> Document {
    let (_, hunks) = parse_doc(Span::new(&contents)).unwrap();
    Document {
        body: contents,
        hunks,
    }
}
