pub use crate::ast::*;
use chumsky::{
    IterParser, ParseResult, Parser,
    error::Rich,
    extra::Err,
    pratt::{infix, left, postfix, prefix},
    prelude::{any, none_of},
    primitive::{choice, group, just},
    recursive::recursive,
    span::{Span, SpanWrap, Spanned},
    text::{inline_whitespace, int, newline, whitespace},
};

pub fn parse_program(input: &str) -> ParseResult<Program<'_>, Rich<'_, char>> {
    let parser = parse_statements(parse_expression()).map(|statements| Program { statements });

    #[cfg(feature = "debug")]
    {
        let debug = parser.debug();
        let _ = std::fs::write("parser.svg", debug.to_railroad_svg().to_string());
        eprintln!("{}", debug.to_ebnf());
    }

    parser.parse(input)
}

fn parse_statements<'a>(
    parse_expression: impl Parser<'a, &'a str, Spanned<Expression<'a>>, Err<Rich<'a, char>>> + Clone,
) -> impl Parser<'a, &'a str, Vec<Spanned<Statement<'a>>>, Err<Rich<'a, char>>> + Clone {
    parse_statement(parse_expression)
        .spanned()
        .padded()
        .separated_by(
            just(';')
                .to(())
                .or(inline_whitespace().ignore_then(newline())),
        )
        .allow_trailing()
        .collect()
        .labelled("statements")
}

fn parse_statement<'a>(
    parse_expression: impl Parser<'a, &'a str, Spanned<Expression<'a>>, Err<Rich<'a, char>>> + Clone,
) -> impl Parser<'a, &'a str, Statement<'a>, Err<Rich<'a, char>>> + Clone {
    choice((
        parse_return(parse_expression.clone()),
        parse_let(parse_expression.clone()),
        parse_expression
            .then(just(';').rewind().or_not().map(|v| v.is_some()))
            .map(|(value, semi)| Statement::Expression { value, semi }),
    ))
    .labelled("statement")
}

fn parse_return<'a>(
    parse_expression: impl Parser<'a, &'a str, Spanned<Expression<'a>>, Err<Rich<'a, char>>> + Clone,
) -> impl Parser<'a, &'a str, Statement<'a>, Err<Rich<'a, char>>> + Clone {
    group((just("return"), whitespace()))
        .ignore_then(parse_expression)
        .map(Statement::Return)
        .labelled("return")
}

fn parse_let<'a>(
    parse_expression: impl Parser<'a, &'a str, Spanned<Expression<'a>>, Err<Rich<'a, char>>> + Clone,
) -> impl Parser<'a, &'a str, Statement<'a>, Err<Rich<'a, char>>> + Clone {
    just("let")
        .ignore_then(parse_identifier().spanned().padded())
        .then_ignore(just('='))
        .then(parse_expression.padded())
        .map(|(name, value)| Statement::Let { name, value })
        .labelled("let")
}

fn parse_expression<'a>()
-> impl Parser<'a, &'a str, Spanned<Expression<'a>>, Err<Rich<'a, char>>> + Clone {
    let parse_boolean = just("true")
        .to(true)
        .or(just("false").to(false))
        .labelled("boolean");

    let parse_null = just("null").to(()).labelled("null");

    let parse_integer = int(10).from_str::<i64>().unwrapped().labelled("integer");

    recursive(|parse_expression| {
        choice((
            parse_expression
                .clone()
                .padded()
                .delimited_by(just('('), just(')'))
                .map(|s: Spanned<_>| s.inner),
            parse_boolean.map(Expression::Boolean),
            parse_null.to(Expression::Null),
            parse_function(parse_expression.clone()),
            parse_if(parse_expression.clone()),
            parse_identifier().map(Expression::Identifier),
            parse_integer.map(Expression::Integer),
            parse_string().map(Expression::String),
            parse_array(parse_expression.clone()).map(Expression::Array),
            parse_map(parse_expression.clone()).map(Expression::Map),
        ))
        .spanned()
        .pratt((
            postfix(
                5,
                parse_expression
                    .clone()
                    .padded()
                    .separated_by(just(','))
                    .allow_trailing()
                    .collect()
                    .delimited_by(just('('), just(')'))
                    .spanned()
                    .labelled("function call"),
                |function: Spanned<_>, arguments: Spanned<_>, _| {
                    let span = function.span.union(arguments.span);
                    Expression::Call {
                        function: Box::new(function),
                        arguments: arguments.inner,
                    }
                    .with_span(span)
                },
            ),
            postfix(
                5,
                parse_expression
                    .padded()
                    .delimited_by(just('['), just(']'))
                    .spanned()
                    .labelled("index"),
                |collection: Spanned<_>, index: Spanned<_>, _| {
                    let span = collection.span.union(index.span);
                    Expression::Index {
                        collection: Box::new(collection),
                        index: Box::new(index.inner),
                    }
                    .with_span(span)
                },
            ),
            prefix(
                4,
                choice((
                    just('-').to(PrefixOperator::Neg),
                    just('!').to(PrefixOperator::Not),
                ))
                .spanned()
                .labelled("prefix"),
                |op: Spanned<_>, right: Spanned<_>, _| {
                    let span = op.span.union(right.span);
                    Expression::Prefix {
                        prefix: op.inner,
                        right: Box::new(right),
                    }
                    .with_span(span)
                },
            ),
            infix(
                left(3),
                choice((
                    just('*').padded().to(InfixOperator::Mul),
                    just('/').padded().to(InfixOperator::Div),
                ))
                .labelled("infix"),
                |left: Spanned<_>, operator, right: Spanned<_>, _| {
                    let span = left.span.union(right.span);
                    Expression::Infix {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                    }
                    .with_span(span)
                },
            ),
            infix(
                left(2),
                choice((
                    just('+').padded().to(InfixOperator::Add),
                    just('-').padded().to(InfixOperator::Sub),
                ))
                .labelled("infix"),
                |left: Spanned<_>, operator, right: Spanned<_>, _| {
                    let span = left.span.union(right.span);
                    Expression::Infix {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                    }
                    .with_span(span)
                },
            ),
            infix(
                left(1),
                choice((
                    just('<').padded().to(InfixOperator::LT),
                    just('>').padded().to(InfixOperator::GT),
                ))
                .labelled("infix"),
                |left: Spanned<_>, operator, right: Spanned<_>, _| {
                    let span = left.span.union(right.span);
                    Expression::Infix {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                    }
                    .with_span(span)
                },
            ),
            infix(
                left(0),
                choice((
                    just("==").padded().to(InfixOperator::Eq),
                    just("!=").padded().to(InfixOperator::Neq),
                ))
                .labelled("infix"),
                |left: Spanned<_>, operator, right: Spanned<_>, _| {
                    let span = left.span.union(right.span);
                    Expression::Infix {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                    }
                    .with_span(span)
                },
            ),
        ))
        .labelled("expression")
    })
    .labelled("expression")
}

fn parse_block<'a>(
    parse_expression: impl Parser<'a, &'a str, Spanned<Expression<'a>>, Err<Rich<'a, char>>> + Clone,
) -> impl Parser<'a, &'a str, Block<'a>, Err<Rich<'a, char>>> + Clone {
    parse_statements(parse_expression)
        .padded()
        .delimited_by(just('{'), just('}'))
        .map(|statements| Block { statements })
        .labelled("block")
}

fn parse_function<'a>(
    parse_expression: impl Parser<'a, &'a str, Spanned<Expression<'a>>, Err<Rich<'a, char>>> + Clone,
) -> impl Parser<'a, &'a str, Expression<'a>, Err<Rich<'a, char>>> + Clone {
    just("fn")
        .ignore_then(
            parse_identifier()
                .spanned()
                .padded()
                .separated_by(just(','))
                .allow_trailing()
                .collect()
                .delimited_by(just('('), just(')'))
                .padded(),
        )
        .then(parse_block(parse_expression).spanned())
        .map(|(parameters, body)| Expression::Function { parameters, body })
        .labelled("function")
}

fn parse_if<'a>(
    parse_expression: impl Parser<'a, &'a str, Spanned<Expression<'a>>, Err<Rich<'a, char>>> + Clone,
) -> impl Parser<'a, &'a str, Expression<'a>, Err<Rich<'a, char>>> + Clone {
    just("if")
        .ignore_then(
            parse_expression
                .clone()
                .padded()
                .delimited_by(just('('), just(')'))
                .padded()
                .map(Box::new),
        )
        .then(parse_block(parse_expression.clone()).spanned())
        .then(
            just("else")
                .padded()
                .ignore_then(parse_block(parse_expression).spanned())
                .or_not(),
        )
        .map(|((condition, consequence), alternative)| Expression::If {
            condition,
            consequence,
            alternative,
        })
        .labelled("if")
}

fn parse_identifier<'a>() -> impl Parser<'a, &'a str, &'a str, Err<Rich<'a, char>>> + Clone {
    any()
        .filter(|c| unicode_ident::is_xid_start(*c))
        .then(
            any()
                .filter(|c| unicode_ident::is_xid_continue(*c))
                .repeated(),
        )
        .to_slice()
        .labelled("identifier")
}

fn parse_string<'a>() -> impl Parser<'a, &'a str, String, Err<Rich<'a, char>>> + Clone {
    #[derive(Clone)]
    enum StringFragment<'a> {
        Literal(&'a str),
        EscapedChar(char),
        EscapedWS,
    }

    let parse_literal = none_of("\"\\")
        .repeated()
        .to_slice()
        .filter(|s: &&str| !s.is_empty())
        .labelled("literal");

    let parse_unicode = just('u').ignore_then(
        int(16)
            .repeated()
            .at_least(1)
            .at_most(6)
            .delimited_by(just('{'), just('}'))
            .to_slice()
            .try_map(|s, span| {
                u32::from_str_radix(s, 16)
                    .ok()
                    .and_then(char::from_u32)
                    .ok_or_else(|| Rich::custom(span, "Invalid hex codepoint"))
            })
            .labelled("unicode"),
    );
    let parse_escaped_char = just('\\')
        .ignore_then(choice((
            parse_unicode,
            just('n').to('\n'),
            just('r').to('\r'),
            just('t').to('\t'),
            just('\\'),
            just('"'),
        )))
        .labelled("escaped character");

    let parse_escaped_whitespace = just('\\')
        .ignore_then(whitespace().at_least(1).to_slice())
        .labelled("escaped whitespace");

    let parse_fragment = choice((
        parse_literal.map(StringFragment::Literal),
        parse_escaped_char.map(StringFragment::EscapedChar),
        parse_escaped_whitespace.to(StringFragment::EscapedWS),
    ));

    parse_fragment
        .repeated()
        .fold(String::new(), |mut string, fragment| {
            match fragment {
                StringFragment::Literal(s) => string += s,
                StringFragment::EscapedChar(c) => string.push(c),
                StringFragment::EscapedWS => {}
            }
            string
        })
        .padded_by(just('"'))
        .labelled("string")
}

fn parse_array<'a>(
    parse_expression: impl Parser<'a, &'a str, Spanned<Expression<'a>>, Err<Rich<'a, char>>> + Clone,
) -> impl Parser<'a, &'a str, Vec<Spanned<Expression<'a>>>, Err<Rich<'a, char>>> + Clone {
    parse_expression
        .padded()
        .separated_by(just(','))
        .allow_trailing()
        .collect()
        .delimited_by(just('['), just(']'))
        .labelled("array")
}

fn parse_map<'a>(
    parse_expression: impl Parser<'a, &'a str, Spanned<Expression<'a>>, Err<Rich<'a, char>>> + Clone,
) -> impl Parser<
    'a,
    &'a str,
    Vec<(Spanned<Expression<'a>>, Spanned<Expression<'a>>)>,
    Err<Rich<'a, char>>,
> + Clone {
    parse_expression
        .clone()
        .padded()
        .then_ignore(just(':'))
        .then(parse_expression.padded())
        .separated_by(just(','))
        .allow_trailing()
        .collect()
        .delimited_by(just('{'), just('}'))
        .labelled("map")
}

pub fn report_errors(errors: Vec<Rich<'_, char>>, input: &str) {
    use ariadne::{Color, Label, Report, ReportKind, Source};

    for error in errors {
        Report::build(ReportKind::Error, error.span().into_range())
            .with_message(message(&error))
            .with_label(
                Label::new(error.span().into_range())
                    .with_message(format!(
                        "Unexpected {}",
                        error
                            .found()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "end of input".to_string())
                    ))
                    .with_color(Color::Red),
            )
            .finish()
            .eprint(Source::from(input))
            .unwrap();
    }
}

fn message(error: &Rich<'_, char>) -> String {
    let mut message = "Unexpected ".to_string();
    match error.found() {
        Some(token) => message += &token.to_string(),
        None => message += "end of input",
    }

    if let Some((label, _)) = error.contexts().next() {
        message += &format!(" while parsing {label}");
    }

    if error.expected().len() > 0 {
        message += "; expected one of ";
        message += &error.expected().next().unwrap().to_string();
        message += &error
            .expected()
            .skip(1)
            .fold(String::new(), |string, pat| format!("{string}, {pat}"));
    }

    message
}
