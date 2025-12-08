use chumsky::{
    prelude::*,
    text::{digits, int},
};
use std::fmt::Display;

// ------------------------------------------------------------------------------------------------
// Public Types ❱ Tokens
// ------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(super) enum Signum {
    #[default]
    Positive,
    Negative,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct IntegerToken<'src>(&'src str);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct DecimalToken<'src>(&'src str);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct FloatToken<'src>(&'src str);

// ------------------------------------------------------------------------------------------------
// Public Functions ❱ Lexer
// ------------------------------------------------------------------------------------------------

pub(super) fn signum<'src>() -> impl Parser<'src, &'src str, Signum, extra::Err<Rich<'src, char>>> {
    just("+")
        .to(Signum::Positive)
        .or(just("-").to(Signum::Negative))
}

pub(super) fn integer<'src>(
    radix: u32,
) -> impl Parser<'src, &'src str, IntegerToken<'src>, extra::Err<Rich<'src, char>>> {
    signum()
        .or_not()
        .then(int(radix))
        .to_slice()
        .map(IntegerToken)
}

pub(super) fn decimal<'src>(
    radix: u32,
) -> impl Parser<'src, &'src str, DecimalToken<'src>, extra::Err<Rich<'src, char>>> {
    integer(radix)
        .then(just("."))
        .then(digits(radix))
        .to_slice()
        .map(DecimalToken)
}

pub(super) fn float<'src>(
    radix: u32,
) -> impl Parser<'src, &'src str, FloatToken<'src>, extra::Err<Rich<'src, char>>> {
    decimal(radix)
        .then(just("e").or(just("E")))
        .then(integer(radix))
        .to_slice()
        .map(FloatToken)
}

// ------------------------------------------------------------------------------------------------
// Implementations ❱ Tokens
// ------------------------------------------------------------------------------------------------

impl Display for Signum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Positive => "+",
                Self::Negative => "-",
            }
        )
    }
}

// ------------------------------------------------------------------------------------------------

impl Display for IntegerToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for IntegerToken<'_> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

// ------------------------------------------------------------------------------------------------

impl Display for DecimalToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for DecimalToken<'_> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

// ------------------------------------------------------------------------------------------------

impl Display for FloatToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for FloatToken<'_> {
    fn as_ref(&self) -> &str {
        self.0
    }
}
