use std::{
    collections::HashSet,
    fmt::Display,
};
use crate::{
    error::Error,
    model::{Arguments, CommandArguments, ComponentArguments, Entry, EntryKind, Identifier, Name, Rustfile, ToolchainArguments, Value},
};
use ariadne::{Color, Label, Report, ReportKind, sources};
use chumsky::{
    extra::ParserExtra,
    input::{SliceInput, StrInput, ValueInput},
    label::LabelError,
    prelude::*,
    text::{TextExpected, Char},
};
use url::Url;

// ------------------------------------------------------------------------------------------------
// Public Functions ❱ Parser
// ------------------------------------------------------------------------------------------------

pub fn parse_str(src: &str, file_name: &str) -> Result<Rustfile, Error> {
    let (tokens,  errs) = rustfile_lexer().parse(src).into_output_errors();

    let parse_errs = if let Some(tokens) = &tokens {
        let (entries, parse_errs) = rustfile_parser()
//            .map_with(|ast: (Vec<Entry<'src>>, SimpleSpan), e: extra::Err<Rich<'_, Token<'src>, Span>>| (ast, e.span()))
            .parse(
                tokens
                    .as_slice()
                    .map((src.len()..src.len()).into(), |(t, s)| (t, s)),
            )
            .into_output_errors();

            if let Some(entries) = entries && parse_errs.is_empty() {
                return Ok(Rustfile::from(verify_entries(entries, file_name)?));
            } else {
                parse_errs
            }
    } else {
        Vec::default()
    };

    let file_name = file_name.to_string();

    errs.into_iter()
        .map(|e| e.map_token(|c| c.to_string()))
        .chain(
            parse_errs
                .into_iter()
                .map(|e| e.map_token(|tok| tok.to_string())),
        )
        .for_each(|e| {
            Report::build(ReportKind::Error, (file_name.clone(), e.span().into_range()))
                .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
                .with_message(e.to_string())
                .with_label(
                    Label::new((file_name.clone(), e.span().into_range()))
                        .with_message(e.reason().to_string())
                        .with_color(Color::Red),
                )
                .with_labels(e.contexts().map(|(label, span)| {
                    Label::new((file_name.clone(), span.into_range()))
                        .with_message(format!("while parsing this {label}"))
                        .with_color(Color::Yellow)
                }))
                .finish()
                .print(sources([(file_name.clone(), src)]))
                .unwrap()
        });
        Err(Error::SourceParserError)
}

fn verify_entries(entries: Vec<Spanner<EntryAst>>, _file_name: &str) -> Result<Vec<Entry>, Error> {
    let mut new_entries: Vec<Entry> = Default::default();
    let mut names_so_far: HashSet<String> = Default::default();

    for entry in entries {
        let tracking_name = format!("{}::{}", entry.node.name.node, entry.node.kind.node);
        if names_so_far.contains(&tracking_name) {
            println!("WARNING");
        } else {
            names_so_far.insert(tracking_name);
        }

        let arguments: Arguments = match entry.node.kind.node {
            EntryKind::Toolchain => {
                let mut args: ToolchainArguments = ToolchainArguments::default();
                for argument in entry.node.arguments.into_iter() {
                    match argument.node.name.node.as_ref() {
                        "profile" => if let Some(name) = argument.node.value.node.into_name() {
                            args.set_profile(name);
                        } else {
                            panic!("invalid type");
                        }
                        "components" => if let Some(name) = argument.node.value.node.into_name_list() {
                            args.set_components(name);
                        } else {
                            panic!("invalid type");
                        }
                        "targets" => if let Some(name) = argument.node.value.node.into_name_list() {
                            args.set_targets(name);
                        } else {
                            panic!("invalid type");
                        }
                        _ => panic!("invalid name"),
                    }
                }
                args.into()
            },
            EntryKind::Component => {
                let mut args: ComponentArguments = ComponentArguments::default();
                for argument in entry.node.arguments.into_iter() {
                    match argument.node.name.node.as_ref() {
                        "target" => if let Some(name) = argument.node.value.node.into_name() {
                            args.set_target(name);
                        } else {
                            panic!("invalid type");
                        }
                        "toolchain" => if let Some(name) = argument.node.value.node.into_name() {
                            args.set_toolchain(name);
                        } else {
                            panic!("invalid type");
                        }
                        _ => panic!("invalid name"),
                    }
                }
                args.into()
            },
            EntryKind::Command => {
                let mut args: CommandArguments = CommandArguments::default();
                for argument in entry.node.arguments.into_iter() {
                    match argument.node.name.node.as_ref() {
                        "all_features" => if let Some(value) = argument.node.value.node.into_boolean() {
                            args.set_all_features(value);
                        } else {
                            panic!("invalid type");
                        }
                        "branch" => if let Some(value) = argument.node.value.node.into_string() {
                            args.set_branch(value);
                        } else {
                            panic!("invalid type");
                        }
                        "debug" => if let Some(value) = argument.node.value.node.into_boolean() {
                            args.set_debug(value);
                        } else {
                            panic!("invalid type");
                        }
                        "features" => if let Some(name_list) = argument.node.value.node.into_name_list() {
                            args.set_features(name_list);
                        } else {
                            panic!("invalid type");
                        }
                        "frozen" => if let Some(value) = argument.node.value.node.into_boolean() {
                            args.set_frozen(value);
                        } else {
                            panic!("invalid type");
                        }
                        "git" => if let Some(value) = argument.node.value.node.into_url() {
                            args.set_git(value);
                        } else {
                            panic!("invalid type");
                        }
                        "index" => if let Some(value) = argument.node.value.node.into_string() {
                            args.set_index(value);
                        } else {
                            panic!("invalid type");
                        }
                        "locked" => if let Some(value) = argument.node.value.node.into_boolean() {
                            args.set_locked(value);
                        } else {
                            panic!("invalid type");
                        }
                        "no_default_features" => if let Some(value) = argument.node.value.node.into_boolean() {
                            args.set_no_default_features(value);
                        } else {
                            panic!("invalid type");
                        }
                        "offline" => if let Some(value) = argument.node.value.node.into_boolean() {
                            args.set_offline(value);
                        } else {
                            panic!("invalid type");
                        }
                        "path" => if let Some(value) = argument.node.value.node.into_string() {
                            args.set_path(value);
                        } else {
                            panic!("invalid type");
                        }
                        "registry" => if let Some(value) = argument.node.value.node.into_string() {
                            args.set_registry(value);
                        } else {
                            panic!("invalid type");
                        }
                        "rev" => if let Some(value) = argument.node.value.node.into_string() {
                            args.set_rev(value);
                        } else {
                            panic!("invalid type");
                        }
                        "tag" => if let Some(value) = argument.node.value.node.into_string() {
                            args.set_tag(value);
                        } else {
                            panic!("invalid type");
                        }
                        "target" => if let Some(name) = argument.node.value.node.into_name() {
                            args.set_target(name);
                        } else {
                            panic!("invalid type");
                        }
                        _ => panic!("invalid name"),
                    }
                }
                args.into()
            },
        };

        new_entries.push(Entry::new(
            0,
            entry.node.name.node,
            arguments,
        ));
    }

    Ok(new_entries)
}

// ------------------------------------------------------------------------------------------------
// Private Types
// ------------------------------------------------------------------------------------------------

// A few type definitions to be used by our parsers below
pub(crate) type Span = SimpleSpan;
pub(crate) type Spanned<T> = (T, Span);

// ------------------------------------------------------------------------------------------------
// Private Types ❱ Lexer
// ------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
enum Token {
    // Raw Identifier
    Ident(Identifier),
    // Values
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Url(Url),
    // Quoted Identifier
    Name(Name),
    EntryKind(EntryKind),
    // Punctuation
    ListStart,
    ListSeparator,
    ListEnd,
    PairNameSymbol,
}

// ------------------------------------------------------------------------------------------------
// Private Types ❱ Parser
// ------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Spanner<T> {
    pub(crate)node: T,
    pub(crate)span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EntryAst {
    pub(crate) kind: Spanner<EntryKind>,
    pub(crate) name: Spanner<Name>,
    pub(crate) arguments: Vec<Spanner<EntryArgumentAst>>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EntryArgumentAst {
    pub(crate) name: Spanner<Identifier>,
    pub(crate) value: Spanner<Value>,
}

// ------------------------------------------------------------------------------------------------
// Private Functions ❱ Lexer
// -----------------------------------------------------------------------------------------------

const IDENT_EXTRA_CHARS: [char; 4] = ['-', '_', '+', '@'];

const NUMBER_RADIX: u32 = 10;

fn rustfile_lexer<'src>()
-> impl Parser<'src, &'src str, Vec<Spanned<Token>>, extra::Err<Rich<'src, char, Span>>> {
    // A parser for booleans
    let boolean = just("true")
        .or(just("false)"))
        .from_str()
        .unwrapped()
        .map(Token::Boolean)
        .labelled("token_boolean");

    let integer = integer(NUMBER_RADIX)
        .from_str()
        .unwrapped()
        .map(Token::Integer)
        .labelled("token_integer");

    let float = float(NUMBER_RADIX)
        .from_str()
        .unwrapped()
        .map(Token::Float)
        .labelled("token_float");

    // A parser for strings
    let string = just('"')
        .ignore_then(none_of('"').repeated().to_slice())
        .then_ignore(just('"'))
        .map(|s: &str| Token::String(s.to_string()))
        .labelled("token_string");

    // A parser for Url
    let url = just('<')
        .ignore_then(none_of('"').repeated().to_slice())
        .then_ignore(just('>'))
        .from_str()
        .unwrapped()
        .map(Token::Url)
        .labelled("token_url");

    let identifier = ident(&IDENT_EXTRA_CHARS)
        .from_str()
        .unwrapped()
        .map(Token::Ident)
        .labelled("token_identifier");

    // A parser for names
    let name = just('\'')
        .ignore_then(ident(&IDENT_EXTRA_CHARS))
        .then_ignore(just('\''))
        .from_str()
        .unwrapped()
        .map(Token::Name)
        .labelled("token_name");

    let entry_kind = just::<&str, &'src str, extra::Err<Rich<'src, char, Span>>>("toolchain")
        .or(just::<&str, &'src str, extra::Err<Rich<'src, char, Span>>>(
            "component)",
        ))
        .or(just::<&str, &'src str, extra::Err<Rich<'src, char, Span>>>(
            "command)",
        ))
        .from_str()
        .unwrapped()
        .map(Token::EntryKind)
        .labelled("token_entry_kind");

    let list_start = just('[').to(Token::ListStart).labelled("token_list_start");
    let list_separator = just(',')
        .to(Token::ListSeparator)
        .labelled("token_list_separator");
    let list_end = just(']').to(Token::ListEnd).labelled("token_list_end");
    let pair_name_symbol = just(':')
        .to(Token::PairNameSymbol)
        .labelled("token_pair_name_symbol");

    // A single token can be one of the above
    let token = boolean
        .or(integer)
        .or(float)
        .or(string)
        .or(url)
        .or(identifier)
        .or(name)
        .or(entry_kind)
        .or(list_start)
        .or(list_separator)
        .or(list_end)
        .or(pair_name_symbol)
        .labelled("token_token");

    let comment = just('#')
        .then(any().and_is(just('\n').not()).repeated())
        .padded()
        .labelled("token_comment");

    token
        .map_with(|tok, e| (tok, e.span()))
        .padded_by(comment.repeated())
        .padded()
        // If we encounter an error, skip and attempt to lex the next character as a token instead
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
        .labelled("all_tokens")
}

fn ident<'src, I, E>(
    extra_chars: &[char],
) -> impl Parser<'src, I, <I as SliceInput<'src>>::Slice, E> + Copy
where
    I: StrInput<'src>,
    I::Token: Char + 'src,
    E: ParserExtra<'src, I>,
    E::Error: LabelError<'src, I, TextExpected<'src, I>>,
{
    any()
        .filter(|c: &I::Token| c.to_ascii().map_or(false, |i| i.is_ascii_alphabetic()))
        .then(
            any()
                .filter(|c: &I::Token| {
                    c.to_ascii().map_or(false, |i| {
                        i.is_ascii_alphanumeric() || extra_chars.contains(&(i as char))
                    })
                })
                .repeated(),
        )
        .to_slice()
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Token::Ident(identifier) => format!("Ident::({identifier})"),
            Token::Boolean(b) => format!("Boolean::({b})"),
            Token::Integer(i) => format!("Integer({i})"),
            Token::Float(f) => format!("Float({f})"),
            Token::String(cow) => format!("String({cow})"),
            Token::Url(url) => format!("Url({url})"),
            Token::Name(name) => format!("Name({name})"),
            Token::EntryKind(entry_kind) => format!("EntryKind({entry_kind})"),
            Token::ListStart => format!("ListStart::`[`"),
            Token::ListSeparator => format!("ListSeparator::`,`"),
            Token::ListEnd => format!("ListEnd::`]"),
            Token::PairNameSymbol => format!("PairNameSymbol::`:"),
        })
    }
}

// ------------------------------------------------------------------------------------------------
// Private Functions ❱ Parser
// ------------------------------------------------------------------------------------------------

fn rustfile_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Vec<Spanner<EntryAst>>, extra::Err<Rich<'tokens, Token, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = Span>,
{
    let ident = select! { Token::Ident(ident) => ident }
        .map_with(|node, xtra| Spanner { node, span: xtra.span() })
        .labelled("Identifier");

    let name = select! { Token::Name(name) => name }
        .map_with(|node, xtra| Spanner { node, span: xtra.span() })
        .labelled("Name");

    let name_list = select! { Token::Name(name) => name, }
        .separated_by(just(Token::ListSeparator))
        .allow_trailing()
        .collect()
        .delimited_by(just(Token::ListStart), just(Token::ListEnd))
        .map(Value::NameList)
        .labelled("NameList");

    let atomic_value = select! {
        Token::Boolean(v) => Value::Boolean(v),
        Token::Integer(v) => Value::Integer(v),
        Token::Float(v) => Value::Float(v),
        Token::String(v) => Value::String(v),
        Token::Url(v) => Value::Url(v),
        Token::Name(v) => Value::Name(v),
    }
        .labelled("AtomicValue");

    let value = atomic_value.or(name_list)
        .map_with(|node, xtra| Spanner { node, span: xtra.span() })
        .labelled("Value");

    let argument_pair = ident
        .then_ignore(just(Token::PairNameSymbol))
        .then(value)
        .map(|(ident, value)| EntryArgumentAst { name: ident, value })
        .map_with(|node, xtra| Spanner { node, span: xtra.span() })
        .labelled("EntryArgument");

    let argument_list = just(Token::ListSeparator)
        .ignore_then(argument_pair)
        .separated_by(just(Token::ListSeparator))
        .allow_trailing()
        .collect::<Vec<Spanner<EntryArgumentAst>>>()
        .labelled("EntryArgumentList");

    let entry_kind = select! { Token::EntryKind(entry_kind) => entry_kind }
        .map_with(|node, xtra| Spanner { node, span: xtra.span() })
        .labelled("EntryKind");

    let entry = entry_kind
        .then(name)
        .then(argument_list.or_not())
        .map(|((entry_kind, name), arguments)| EntryAst { name, kind: entry_kind, arguments: arguments.unwrap_or_default() })
        .map_with(|node, xtra| Spanner { node, span: xtra.span() })
        .labelled("Entry");

    entry.repeated().collect::<Vec<_>>()
}

// ------------------------------------------------------------------------------------------------
// Modules
// ------------------------------------------------------------------------------------------------

mod numbers;
use numbers::{float, integer};
