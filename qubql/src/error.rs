use std::fmt;

use crate::{token::{Location, Token}, util::Str};

#[derive(Debug)]
pub enum Error<'data, 'token> where 'data: 'token {
    /// An IO error has occured.
    Io(Box<std::io::Error>),
    /// String wasn't closed.
    UnenclosedString {
        start: Location,
        pos: Option<Location>,
        err_code: &'static str,
    },
    UnenclosedBlock {
        start: Location,
        pos: Option<Location>,
        /// '}', ']' or ')'.
        expected: char,
        err_code: &'static str,
    },
    UnfinishedEscapeX {
        start: Location,
        /// 0 or 1 since \x only requires 2 chars.
        idx: u8,
        err_code: &'static str,
    },
    EscapeXInvalidChar {
        start: Location,
        /// 0 or 1 since \x only requires 2 chars.
        idx: u8,
        err_code: &'static str,
    },
    UnfinishedEscapeU {
        start: Location,
        /// 0 or 1 since \x only requires 2 chars.
        idx: u8,
        err_code: &'static str,
    },
    EscapeUInvalidChar {
        start: Location,
        /// 0 or 1 since \x only requires 2 chars.
        idx: u8,
        err_code: &'static str,
    },
    EscapeUInvalidCode {
        start: Location,
        provided: u32,
        err_code: &'static str,
    },
    EscapeUInvalidEscape {
        start: Location,
        provided: char,
        err_code: &'static str,
    },
    /// Unexpected parenthesis.
    UnexpectedParen {
        char: char,
        /// '\0' if none.
        expected: char,
        pos: Location,
        err_code: &'static str,
    },
    /// Multiple errors have occured.
    MultipleErrors(Vec<Error<'data, 'token>>),
    /// Too much nesting.
    Nesting {
        loc: Location,
        err_code: &'static str,
    },
    StatementInvalidToken {
        token: &'token Token<'data>,
        err_code: &'static str,
    },
    UnexpectedToken {
        token: &'token Token<'data>,
        err_code: &'static str,
    },
    MissingToken {
        loc: Location,
        err_code: &'static str,
    },
    InvalidObjectKind {
        provided: Str<'token>,
        loc: Location,
        err_code: &'static str,
    },
    DuplicateField {
        prev_loc: Location,
        token: &'token Token<'data>,
        err_code: &'static str,
    },
}
impl<'data, 'token> Error<'data, 'token> {
    /// Whether the error is "recoverable".
    ///
    /// This is a lie. It's just a funny flag for parser to continue parsing
    /// to give you all errors.
    ///
    /// You can use this if you want, but you've been warned.
    pub const fn is_recoverable(&self) -> bool {
        match self {
            Self::MultipleErrors(x) => {
                let slice = x.as_slice();
                let mut i = 0;
                while i < slice.len() {
                    if !slice[i].is_recoverable() { return false; }
                    i += 1;
                }
                true
            },
            Self::Io(_) => false,
            Self::Nesting { .. } => false,
            _ => true,
        }
    }

    pub fn fancy<'borrow>(&'borrow self) -> Display<'borrow, 'data, 'token> {
        Display(self)
    }
}
impl<'data, 'token> fmt::Display for Error<'data, 'token> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(error) => fmt::Display::fmt(error, f),
            Error::UnenclosedString { start, pos, err_code }
                => match pos {
                    Some(pos) => f.write_fmt(format_args!("{pos}: encountered a newline while parsing a string at {start} ({err_code})")),
                    None => f.write_fmt(format_args!("{start}: file ended before a string closing character could've been encountered ({err_code})")),
                },
            Error::UnenclosedBlock { start, pos, expected, err_code }
                => match pos {
                    Some(pos) => f.write_fmt(format_args!("{pos}: was trying to find {expected} to close a block at {start} ({err_code})")),
                    None => f.write_fmt(format_args!("{start}: file ended before '{expected}' character could've been encountered ({err_code})")),
                },
            Error::UnfinishedEscapeX { start, idx, err_code }
                => f.write_fmt(format_args!("{start}: \\x requires 2 extra characters while only {idx} were supplied ({err_code})")),
            Error::EscapeXInvalidChar { start, idx, err_code }
                => f.write_fmt(format_args!("{start}: invalid character on position {idx} ({err_code})")),
            Error::UnfinishedEscapeU { start, idx, err_code }
                => f.write_fmt(format_args!("{start}: \\u requires 4 extra characters while only {idx} were supplied ({err_code})")),
            Error::EscapeUInvalidChar { start, idx, err_code }
                => f.write_fmt(format_args!("{start}: invalid character on position {idx} ({err_code})")),
            Error::EscapeUInvalidCode { start, provided, err_code }
                => f.write_fmt(format_args!("{start}: {provided} is not a valid codepoint ({err_code})")),
            Error::EscapeUInvalidEscape { start, provided, err_code }
                => f.write_fmt(format_args!("{start}: {:?} is not a valid escape ({err_code})", format_args!("\\{provided}"))),
            Error::UnexpectedParen { char, expected, pos, err_code }
                => if *expected == '\0' {
                    f.write_fmt(format_args!("{pos}: unexpected parenthesis '{char}' ({err_code})"))
                } else {
                    f.write_fmt(format_args!("{pos}: expected to find '{expected}', got '{char}' instead ({err_code})"))
                },
            Error::MultipleErrors(errors) => {
                for x in errors {
                    f.write_fmt(format_args!("{x}\n"))?;
                }
                Ok(())
            },
            Error::Nesting { loc, err_code }
                => f.write_fmt(format_args!("{loc}: too many nested blocks ({err_code})")),
            Error::StatementInvalidToken { token, err_code }
                => f.write_fmt(format_args!("{}: there is no {:?} statement ({err_code})", token.location(), token.display())),
            Error::UnexpectedToken { token, err_code }
                => f.write_fmt(format_args!("{}: unexpected token {:?} ({err_code})", token.location(), token.display())),
            Error::MissingToken { loc, err_code }
                => f.write_fmt(format_args!("{loc}: there's no following token ({err_code})")),
            Error::InvalidObjectKind { provided, loc, err_code }
                => f.write_fmt(format_args!("{loc}: invalid object kind {provided:?} ({err_code})")),
            Error::DuplicateField { prev_loc, token, err_code }
                => f.write_fmt(format_args!("{}: duplicate field {:?}, previous token defined at {prev_loc} ({err_code})", token.location(), token.display())),
        }
    }
}
impl<'data, 'token> std::error::Error for Error<'data, 'token> {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Error::Io(error) => Some(error),
            _ => None,
        }
    }
}

pub struct OwnedDisplay<'err, 'token>(Error<'err, 'token>);
impl<'err, 'token> OwnedDisplay<'err, 'token> {
    fn as_refd<'borrow>(&'borrow self) -> Display<'borrow, 'err, 'token> {
        Display(&self.0)
    }
}
impl fmt::Display for OwnedDisplay<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_refd().fmt(f)
    }
}
impl fmt::Debug for OwnedDisplay<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_refd().fmt(f)
    }
}

pub struct Display<'borrow, 'data, 'token>(&'borrow Error<'data, 'token>);
impl Display<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Error::MultipleErrors(errors) => {
                for x in errors {
                    x.fancy().fmt(f)?;
                }
                Ok(())
            },
            x => f.write_fmt(format_args!("{x}\n")),
        }
    }
}
impl fmt::Display for Display<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Self::fmt(self, f)
    }
}
impl fmt::Debug for Display<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Self::fmt(self, f)
    }
}

pub trait ResultExt<T, E> {
    fn into_fancy(self) -> std::result::Result<T, E>;
}
impl<'data, 'token, T> ResultExt<T, OwnedDisplay<'data, 'token>> for std::result::Result<T, Error<'data, 'token>> {
    fn into_fancy(self) -> std::result::Result<T, OwnedDisplay<'data, 'token>> {
        match self {
            Ok(x) => Ok(x),
            Err(why) => Err(OwnedDisplay(why)),
        }
    }
}

pub type Result<'a, 'b, T, E = Error<'a, 'b>> = std::result::Result<T, E>;
