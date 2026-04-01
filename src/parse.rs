pub use crate::ast::*;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{is_not, tag, take_while, take_while_m_n},
    character::complete::{char, digit1, line_ending, multispace0, multispace1, satisfy},
    combinator::{eof, opt, peek, recognize, value, verify},
    multi::{fold, separated_list0},
    sequence::{delimited, preceded, separated_pair, terminated},
};
use nom_locate::LocatedSpan;
use nom_tracable::{TracableInfo, tracable_parser};

type InputSpan<'a> = LocatedSpan<&'a str, TracableInfo>;

impl Spanned for InputSpan<'_> {
    fn span(&self) -> Span {
        (self.location_offset()..(self.location_offset() + self.len())).into()
    }
}

fn spanned_tag<
    T: nom::Input + Clone,
    I: nom::Input + nom::Compare<T> + Spanned,
    Error: nom::error::ParseError<I>,
>(
    tag: T,
) -> impl Parser<I, Output = Span, Error = Error> {
    nom::bytes::complete::tag(tag).map(|v| Spanned::span(&v))
}

fn surround_ws<I: Clone + nom::Input, E: nom::error::ParseError<I>, O>(
    f: impl Parser<I, Output = O, Error = E>,
) -> impl Parser<I, Output = O, Error = E>
where
    I::Item: nom::AsChar,
{
    delimited(multispace0, f, multispace0)
}

/// Comma-separated list with optional trailing comma and surrounding whitespace
fn csl<I: Clone + nom::Input, E: nom::error::ParseError<I>, F: Parser<I, Error = E>>(
    f: F,
) -> impl Parser<I, Output = Vec<F::Output>, Error = E>
where
    I::Item: nom::AsChar,
{
    terminated(
        separated_list0(surround_ws(char(',')), f),
        opt(surround_ws(char(','))),
    )
}

pub fn parse_program(input: &str) -> Result<Program<'_>, nom::Err<nom::error::Error<&str>>> {
    parse_statements(InputSpan::new_extra(input, TracableInfo::default()))
        .map(|(_, statements)| Program { statements })
        .map_err(|e| e.map_input(InputSpan::into_fragment))
}

#[tracable_parser]
fn parse_statements(input: InputSpan) -> IResult<InputSpan, Vec<Statement>> {
    separated_list0(multispace0, parse_statement).parse(input)
}

#[tracable_parser]
fn parse_statement(input: InputSpan) -> IResult<InputSpan, Statement> {
    terminated(
        alt((
            parse_return,
            parse_let,
            (parse_expression, opt(peek(char(';'))).map(|v| v.is_some()))
                .map(|(value, semi)| Statement::Expression { value, semi }),
        )),
        alt((tag(";"), line_ending, eof)),
    )
    .parse(input)
}

#[tracable_parser]
fn parse_let(input: InputSpan) -> IResult<InputSpan, Statement> {
    (
        spanned_tag("let"),
        surround_ws(parse_identifier),
        preceded(surround_ws(char('=')), parse_expression),
    )
        .map(|(let_span, name, value)| Statement::Let {
            let_span,
            name,
            value,
        })
        .parse(input)
}

#[tracable_parser]
fn parse_expression(input: InputSpan) -> IResult<InputSpan, Expression> {
    parse_expression_inner(input, 0)
}

fn parse_expression_inner(input: InputSpan, min_precedence: u8) -> IResult<InputSpan, Expression> {
    let (mut input, mut lhs) = alt((
        parse_boolean,
        parse_null,
        parse_function,
        parse_if,
        parse_identifier.map(Expression::Identifier),
        parse_grouped,
        parse_integer,
        parse_prefix,
        parse_string,
        parse_array,
        parse_map,
    ))
    .parse(input)?;

    loop {
        if let Ok((next_input, (arguments, close_span))) = parse_call_args(input) {
            lhs = Expression::Call {
                function: Box::new(lhs),
                arguments,
                close_span,
            };
            input = next_input;
            continue;
        }

        if let Ok((next_input, (index, close_span))) = parse_index(input) {
            lhs = Expression::Index {
                collection: Box::new(lhs),
                index,
                close_span,
            };
            input = next_input;
            continue;
        }

        let Ok((next_input, operator)) =
            delimited(multispace0, parse_infix_operator, multispace0).parse(input)
        else {
            break;
        };

        let (lp, rp) = operator.precedence();

        if lp < min_precedence {
            break;
        }

        let (next_input, rhs) = parse_expression_inner(next_input, rp)?;

        lhs = Expression::Infix {
            left: Box::new(lhs),
            operator,
            right: Box::new(rhs),
        };
        input = next_input;
    }

    Ok((input, lhs))
}

#[tracable_parser]
fn parse_grouped(input: InputSpan) -> IResult<InputSpan, Expression> {
    delimited(char('('), parse_expression, char(')')).parse(input)
}

#[tracable_parser]
fn parse_identifier(input: InputSpan) -> IResult<InputSpan, Identifier> {
    recognize((
        satisfy(unicode_ident::is_xid_start),
        take_while(unicode_ident::is_xid_continue),
    ))
    .map(|value| Identifier {
        span: Spanned::span(&value),
        name: InputSpan::into_fragment(value),
    })
    .parse(input)
}

#[tracable_parser]
fn parse_return(input: InputSpan) -> IResult<InputSpan, Statement> {
    separated_pair(spanned_tag("return"), multispace0, parse_expression)
        .map(|(return_span, value)| Statement::Return { return_span, value })
        .parse(input)
}

#[tracable_parser]
fn parse_integer(input: InputSpan) -> IResult<InputSpan, Expression> {
    digit1
        .map_res(|digits: InputSpan| {
            digits.parse().map(|value| Expression::Integer {
                span: digits.span(),
                value,
            })
        })
        .parse(input)
}

#[tracable_parser]
fn parse_prefix(input: InputSpan) -> IResult<InputSpan, Expression> {
    (parse_prefix_operator, parse_expression.map(Box::new))
        .map(|(prefix, right)| Expression::Prefix { prefix, right })
        .parse(input)
}

#[tracable_parser]
fn parse_prefix_operator(input: InputSpan) -> IResult<InputSpan, Prefix> {
    alt((
        spanned_tag("-").map(|v| (v, PrefixOperator::Neg)),
        spanned_tag("!").map(|v| (v, PrefixOperator::Not)),
    ))
    .map(|(span, operator)| Prefix { span, operator })
    .parse(input)
}

#[tracable_parser]
fn parse_infix_operator(input: InputSpan) -> IResult<InputSpan, InfixOperator> {
    alt((
        value(InfixOperator::Eq, tag("==")),
        value(InfixOperator::Neq, tag("!=")),
        value(InfixOperator::Add, char('+')),
        value(InfixOperator::Sub, char('-')),
        value(InfixOperator::Mul, char('*')),
        value(InfixOperator::Div, char('/')),
        value(InfixOperator::LT, char('<')),
        value(InfixOperator::GT, char('>')),
    ))
    .parse(input)
}

#[tracable_parser]
fn parse_boolean(input: InputSpan) -> IResult<InputSpan, Expression> {
    alt((
        tag("true").map(|v| (v, true)),
        tag("false").map(|v| (v, false)),
    ))
    .map(|(v, value): (InputSpan, _)| Expression::Boolean {
        span: v.span(),
        value,
    })
    .parse(input)
}

#[tracable_parser]
fn parse_if(input: InputSpan) -> IResult<InputSpan, Expression> {
    (
        spanned_tag("if"),
        delimited(
            multispace0,
            delimited(char('('), parse_expression.map(Box::new), char(')')),
            multispace0,
        ),
        parse_block,
        preceded(
            multispace0,
            opt(preceded((tag("else"), multispace0), parse_block)),
        ),
    )
        .map(
            |(if_span, condition, consequence, alternative)| Expression::If {
                if_span,
                condition,
                consequence,
                alternative,
            },
        )
        .parse(input)
}

#[tracable_parser]
fn parse_block(input: InputSpan) -> IResult<InputSpan, Block> {
    (
        spanned_tag("{"),
        delimited(multispace0, parse_statements, multispace0),
        spanned_tag("}"),
    )
        .map(|(open_span, statements, close_span)| Block {
            open_span,
            statements,
            close_span,
        })
        .parse(input)
}

#[tracable_parser]
fn parse_function(input: InputSpan) -> IResult<InputSpan, Expression> {
    (
        spanned_tag("fn"),
        delimited(
            (char('('), multispace0),
            csl(parse_identifier),
            (char(')'), multispace0),
        ),
        multispace0,
        parse_block,
    )
        .map(|(fn_span, parameters, _, body)| Expression::Function {
            fn_span,
            parameters,
            body,
        })
        .parse(input)
}

#[tracable_parser]
fn parse_call_args(input: InputSpan) -> IResult<InputSpan, (Vec<Expression>, Span)> {
    (preceded(char('('), csl(parse_expression)), spanned_tag(")")).parse(input)
}

#[tracable_parser]
fn parse_null(input: InputSpan) -> IResult<InputSpan, Expression> {
    spanned_tag("null").map(Expression::Null).parse(input)
}

#[tracable_parser]
fn parse_string(input: InputSpan) -> IResult<InputSpan, Expression> {
    (
        spanned_tag("\""),
        fold(0.., parse_fragment, String::new, |mut string, fragment| {
            match fragment {
                StringFragment::Literal(s) => string += s,
                StringFragment::EscapedChar(c) => string.push(c),
                StringFragment::EscapedWS => {}
            }
            string
        }),
        spanned_tag("\""),
    )
        .map(|(open, value, close)| Expression::String {
            span: open.join(close),
            value,
        })
        .parse(input)
}

#[derive(Clone)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWS,
}

#[tracable_parser]
fn parse_fragment(input: InputSpan) -> IResult<InputSpan, StringFragment> {
    alt((
        parse_literal.map(StringFragment::Literal),
        parse_escaped_char.map(StringFragment::EscapedChar),
        value(StringFragment::EscapedWS, parse_escaped_whitespace),
    ))
    .parse(input)
}

#[tracable_parser]
fn parse_literal<'a>(input: InputSpan<'a>) -> IResult<InputSpan<'a>, &'a str> {
    verify(is_not("\"\\"), |s: &InputSpan| !s.is_empty())
        .map(InputSpan::into_fragment)
        .parse(input)
}

#[tracable_parser]
fn parse_escaped_char(input: InputSpan) -> IResult<InputSpan, char> {
    preceded(
        char('\\'),
        alt((
            parse_unicode,
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            char('\\'),
            char('"'),
        )),
    )
    .parse(input)
}

#[tracable_parser]
fn parse_unicode(input: InputSpan) -> IResult<InputSpan, char> {
    preceded(
        char('u'),
        delimited(
            char('{'),
            take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit()).map(InputSpan::into_fragment),
            char('}'),
        ),
    )
    .map_res(|hex| u32::from_str_radix(hex, 16))
    .map_opt(std::char::from_u32)
    .parse(input)
}

#[tracable_parser]
fn parse_escaped_whitespace(input: InputSpan) -> IResult<InputSpan, InputSpan> {
    preceded(char('\\'), multispace1).parse(input)
}

#[tracable_parser]
fn parse_array(input: InputSpan) -> IResult<InputSpan, Expression> {
    (spanned_tag("["), csl(parse_expression), spanned_tag("]"))
        .map(|(open_span, elements, close_span)| Expression::Array {
            open_span,
            elements,
            close_span,
        })
        .parse(input)
}

#[tracable_parser]
fn parse_index(input: InputSpan) -> IResult<InputSpan, (Box<Expression>, Span)> {
    (
        preceded(char('['), parse_expression).map(Box::new),
        spanned_tag("]"),
    )
        .parse(input)
}

#[tracable_parser]
fn parse_map(input: InputSpan) -> IResult<InputSpan, Expression> {
    (
        surround_ws(spanned_tag("{")),
        csl(separated_pair(
            parse_expression,
            surround_ws(char(':')),
            parse_expression,
        )),
        surround_ws(spanned_tag("}")),
    )
        .map(|(open_span, elements, close_span)| Expression::Map {
            open_span,
            elements,
            close_span,
        })
        .parse(input)
}
