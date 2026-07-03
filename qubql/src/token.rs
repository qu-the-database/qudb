use std::{fmt, iter::Peekable, marker::PhantomData, ops::Deref};

use crate::{error::{Error, Result}, util::Str};

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum TokenImportance {
    /// Comments
    Cosmetic,
    /// Doc-comments
    Docs,
    /// Actual code.
    Code,
}
impl TokenImportance {
    pub const fn is_code(&self) -> bool {
        matches!(self, Self::Code)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Char {
    pub c: char,
    pub loc: Location,
}
impl Deref for Char {
    type Target = char;

    fn deref(&self) -> &Self::Target {
        &self.c
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Location {
    pub row: usize,
    pub column: usize,
    pub idx: usize,
}
impl Location {
    pub const ZERO: Location = Location { row: 0, column: 0, idx: 0 };
}
impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}:{}", self.row, self.column))
    }
}

pub trait SomeToken<'data> {
    /// Make this token render nice.
    ///
    /// Used in traces.
    fn display<'borrow>(&'borrow self) -> TokenDisplay<'borrow, 'data, Self> { TokenDisplay::new(self) }
    /// Obtain this token's [Location].
    ///
    /// Used to give usable traces.
    fn location(&self) -> Location;
    /// Obtain this token's [TokenImportance].
    ///
    /// Used to exclude cosmetic components when parsing.
    fn importance(&self) -> TokenImportance;
    /// Wrap this token into [Token].
    fn wrap(self) -> Token<'data> where Self: Sized;
}

#[derive(Debug, Clone)]
pub struct IdentToken<'data> {
    pub location: Location,
    pub value: Str<'data>,
}
impl<'data> IdentToken<'data> {
    /// Create an already wrapped [Token].
    pub const fn new_wrapped(location: Location, value: Str<'data>) -> Token<'data> {
        Token::Ident(IdentToken { location, value })
    }
    /// Create an unwrapped token.
    pub const fn new(location: Location, value: Str<'data>) -> Self {
        Self { location, value }
    }
    /// Make this token render nice.
    ///
    /// Used in traces.
    pub const fn display<'borrow>(&'borrow self) -> TokenDisplay<'borrow, 'data, Self> { TokenDisplay::new(self) }
    /// This token's [TokenImportance].
    pub const IMPORTANCE: TokenImportance = TokenImportance::Code;
    /// Obtain this token's [Location].
    ///
    /// Used to give usable traces.
    pub const fn location(&self) -> Location { self.location }
    /// Obtain this token's [TokenImportance].
    ///
    /// Used to exclude cosmetic components when parsing.
    pub const fn importance(&self) -> TokenImportance { Self::IMPORTANCE }
    /// Wrap this token into [Token].
    pub const fn wrap(self) -> Token<'data> { Token::Ident(self) }
    /// Obtain the value of this token as an [str].
    pub const fn as_str(&self) -> &str {
        match &self.value {
            Str::Borrowed(x) => *x,
            Str::Owned(x) => &**x,
        }
    }
}
impl<'data> SomeToken<'data> for IdentToken<'data> {
    fn location(&self) -> Location { self.location }
    fn importance(&self) -> TokenImportance { self.importance() }
    fn wrap(self) -> Token<'data> { self.wrap() }
}
impl<'borrow, 'data> fmt::Display for TokenDisplay<'borrow, 'data, IdentToken<'data>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.kind.value)
    }
}
impl<'borrow, 'data> fmt::Debug for TokenDisplay<'borrow, 'data, IdentToken<'data>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.kind.value.contains(|x: char| x.is_whitespace() || x.is_ascii_punctuation() && x != '$') {
            f.write_str("`")?;

            let search: fn(char) -> bool = |x: char| x.is_whitespace() || x.is_ascii_punctuation() && x != '$';
            let mut s = self.kind.value.as_ref();

            while let Some(i) = s.find(search) {
                if i != 0 {
                    f.write_str(&s[..i])?;
                }
                let cl = s[i..].chars().next().unwrap().len_utf8();
                f.write_str("/")?;
                f.write_str(&s[i..][..cl])?;
                s = &s[i..][cl..];
            }

            f.write_str(s)?;
            f.write_str("`")
        } else {
            f.write_str(self.kind.value.as_ref())
        }
    }
}

#[derive(Debug, Clone)]
pub struct StringToken<'data> {
    pub location: Location,
    pub value: Str<'data>,
}
impl<'data> StringToken<'data> {
    /// Create an already wrapped [Token].
    pub const fn new_wrapped(location: Location, value: Str<'data>) -> Token<'data> {
        Token::String(StringToken { location, value })
    }
    /// Create an unwrapped token.
    pub const fn new(location: Location, value: Str<'data>) -> Self {
        Self { location, value }
    }
    /// Make this token render nice.
    ///
    /// Used in traces.
    pub const fn display<'borrow>(&'borrow self) -> TokenDisplay<'borrow, 'data, Self> { TokenDisplay::new(self) }
    /// This token's [TokenImportance].
    pub const IMPORTANCE: TokenImportance = TokenImportance::Code;
    /// Obtain this token's [Location].
    ///
    /// Used to give usable traces.
    pub const fn location(&self) -> Location { self.location }
    /// Obtain this token's [TokenImportance].
    ///
    /// Used to exclude cosmetic components when parsing.
    pub const fn importance(&self) -> TokenImportance { Self::IMPORTANCE }
    /// Wrap this token into [Token].
    pub const fn wrap(self) -> Token<'data> { Token::String(self) }
    /// Obtain the value of this token as an [str].
    pub const fn as_str(&self) -> &str {
        match &self.value {
            Str::Borrowed(x) => *x,
            Str::Owned(x) => &**x,
        }
    }
}
impl<'data> SomeToken<'data> for StringToken<'data> {
    fn location(&self) -> Location { self.location }
    fn importance(&self) -> TokenImportance { self.importance() }
    fn wrap(self) -> Token<'data> { self.wrap() }
}
impl<'borrow, 'data> fmt::Display for TokenDisplay<'borrow, 'data, StringToken<'data>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.kind.value)
    }
}
impl<'borrow, 'data> fmt::Debug for TokenDisplay<'borrow, 'data, StringToken<'data>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self.kind.value))
    }
}

#[derive(Debug, Clone)]
pub struct OpToken<'data> {
    pub location: Location,
    pub value: &'data str,
}
impl<'data> OpToken<'data> {
    /// Create an already wrapped [Token].
    pub const fn new_wrapped(location: Location, value: &'data str) -> Token<'data> {
        Token::Operator(OpToken { location, value })
    }
    /// Create an unwrapped token.
    pub const fn new(location: Location, value: &'data str) -> Self {
        Self { location, value }
    }
    /// Make this token render nice.
    ///
    /// Used in traces.
    pub const fn display<'borrow>(&'borrow self) -> TokenDisplay<'borrow, 'data, Self> { TokenDisplay::new(self) }
    /// This token's [TokenImportance].
    pub const IMPORTANCE: TokenImportance = TokenImportance::Code;
    /// Obtain this token's [Location].
    ///
    /// Used to give usable traces.
    pub const fn location(&self) -> Location { self.location }
    /// Obtain this token's [TokenImportance].
    ///
    /// Used to exclude cosmetic components when parsing.
    pub const fn importance(&self) -> TokenImportance { Self::IMPORTANCE }
    /// Wrap this token into [Token].
    pub const fn wrap(self) -> Token<'data> { Token::Operator(self) }
}
impl<'data> SomeToken<'data> for OpToken<'data> {
    fn location(&self) -> Location { self.location }
    fn importance(&self) -> TokenImportance { self.importance() }
    fn wrap(self) -> Token<'data> { self.wrap() }
}
impl<'borrow, 'data> fmt::Display for TokenDisplay<'borrow, 'data, OpToken<'data>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.kind.value)
    }
}
impl<'borrow, 'data> fmt::Debug for TokenDisplay<'borrow, 'data, OpToken<'data>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.kind.value)
    }
}

#[derive(Debug, Clone)]
pub struct CommentToken<'data> {
    pub location: Location,
    pub value: &'data str,
}
impl<'data> CommentToken<'data> {
    /// Create an already wrapped [Token].
    pub const fn new_wrapped(location: Location, value: &'data str) -> Token<'data> {
        Token::Comment(CommentToken { location, value })
    }
    /// Create an unwrapped token.
    pub const fn new(location: Location, value: &'data str) -> Self {
        Self { location, value }
    }
    /// Make this token render nice.
    ///
    /// Used in traces.
    pub const fn display<'borrow>(&'borrow self) -> TokenDisplay<'borrow, 'data, Self> { TokenDisplay::new(self) }
    /// This token's [TokenImportance].
    pub const IMPORTANCE: TokenImportance = TokenImportance::Cosmetic;
    /// Obtain this token's [Location].
    ///
    /// Used to give usable traces.
    pub const fn location(&self) -> Location { self.location }
    /// Obtain this token's [TokenImportance].
    ///
    /// Used to exclude cosmetic components when parsing.
    pub const fn importance(&self) -> TokenImportance { Self::IMPORTANCE }
    /// Wrap this token into [Token].
    pub const fn wrap(self) -> Token<'data> { Token::Comment(self) }
}
impl<'data> SomeToken<'data> for CommentToken<'data> {
    fn location(&self) -> Location { self.location }
    fn importance(&self) -> TokenImportance { self.importance() }
    fn wrap(self) -> Token<'data> { self.wrap() }
}
impl<'borrow, 'data> fmt::Display for TokenDisplay<'borrow, 'data, CommentToken<'data>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("/* .. */")
    }
}
impl<'borrow, 'data> fmt::Debug for TokenDisplay<'borrow, 'data, CommentToken<'data>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("/* .. */")
    }
}

#[derive(Debug, Clone)]
pub struct DocToken<'data> {
    pub location: Location,
    pub value: &'data str,
}
impl<'data> DocToken<'data> {
    /// Create an already wrapped [Token].
    pub const fn new_wrapped(location: Location, value: &'data str) -> Token<'data> {
        Token::Doc(DocToken { location, value })
    }
    /// Create an unwrapped token.
    pub const fn new(location: Location, value: &'data str) -> Self {
        Self { location, value }
    }
    /// Make this token render nice.
    ///
    /// Used in traces.
    pub const fn display<'borrow>(&'borrow self) -> TokenDisplay<'borrow, 'data, Self> { TokenDisplay::new(self) }
    /// This token's [TokenImportance].
    pub const IMPORTANCE: TokenImportance = TokenImportance::Docs;
    /// Obtain this token's [Location].
    ///
    /// Used to give usable traces.
    pub const fn location(&self) -> Location { self.location }
    /// Obtain this token's [TokenImportance].
    ///
    /// Used to exclude cosmetic components when parsing.
    pub const fn importance(&self) -> TokenImportance { Self::IMPORTANCE }
    /// Wrap this token into [Token].
    pub const fn wrap(self) -> Token<'data> { Token::Doc(self) }
}
impl<'data> SomeToken<'data> for DocToken<'data> {
    fn location(&self) -> Location { self.location }
    fn importance(&self) -> TokenImportance { self.importance() }
    fn wrap(self) -> Token<'data> { self.wrap() }
}
impl<'borrow, 'data> fmt::Display for TokenDisplay<'borrow, 'data, DocToken<'data>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("/** .. */")
    }
}
impl<'borrow, 'data> fmt::Debug for TokenDisplay<'borrow, 'data, DocToken<'data>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("/** .. */")
    }
}

#[derive(Debug, Clone)]
pub struct BlockCurlyToken<'data> {
    pub location: Location,
    pub value: Vec<Token<'data>>,
}
impl<'data> BlockCurlyToken<'data> {
    /// Create an already wrapped [Token].
    pub const fn new_wrapped(location: Location, value: Vec<Token<'data>>) -> Token<'data> {
        Token::BlockCurly(BlockCurlyToken { location, value })
    }
    /// Create an unwrapped token.
    pub const fn new(location: Location, value: Vec<Token<'data>>) -> Self {
        Self { location, value }
    }
    /// Make this token render nice.
    ///
    /// Used in traces.
    pub const fn display<'borrow>(&'borrow self) -> TokenDisplay<'borrow, 'data, Self> { TokenDisplay::new(self) }
    /// This token's [TokenImportance].
    pub const IMPORTANCE: TokenImportance = TokenImportance::Code;
    /// Obtain this token's [Location].
    ///
    /// Used to give usable traces.
    pub const fn location(&self) -> Location { self.location }
    /// Obtain this token's [TokenImportance].
    ///
    /// Used to exclude cosmetic components when parsing.
    pub const fn importance(&self) -> TokenImportance { Self::IMPORTANCE }
    /// Wrap this token into [Token].
    pub const fn wrap(self) -> Token<'data> { Token::BlockCurly(self) }
}
impl<'data> SomeToken<'data> for BlockCurlyToken<'data> {
    fn location(&self) -> Location { self.location }
    fn importance(&self) -> TokenImportance { self.importance() }
    fn wrap(self) -> Token<'data> { self.wrap() }
}
impl<'borrow, 'data> fmt::Display for TokenDisplay<'borrow, 'data, BlockCurlyToken<'data>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{ .. }")
    }
}
impl<'borrow, 'data> fmt::Debug for TokenDisplay<'borrow, 'data, BlockCurlyToken<'data>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{ .. }")
    }
}

#[derive(Debug, Clone)]
pub struct BlockRoundToken<'data> {
    pub location: Location,
    pub value: Vec<Token<'data>>,
}
impl<'data> BlockRoundToken<'data> {
    /// Create an already wrapped [Token].
    pub const fn new_wrapped(location: Location, value: Vec<Token<'data>>) -> Token<'data> {
        Token::BlockRound(BlockRoundToken { location, value })
    }
    /// Create an unwrapped token.
    pub const fn new(location: Location, value: Vec<Token<'data>>) -> Self {
        Self { location, value }
    }
    /// Make this token render nice.
    ///
    /// Used in traces.
    pub const fn display<'borrow>(&'borrow self) -> TokenDisplay<'borrow, 'data, Self> { TokenDisplay::new(self) }
    /// This token's [TokenImportance].
    pub const IMPORTANCE: TokenImportance = TokenImportance::Code;
    /// Obtain this token's [Location].
    ///
    /// Used to give usable traces.
    pub const fn location(&self) -> Location { self.location }
    /// Obtain this token's [TokenImportance].
    ///
    /// Used to exclude cosmetic components when parsing.
    pub const fn importance(&self) -> TokenImportance { Self::IMPORTANCE }
    /// Wrap this token into [Token].
    pub const fn wrap(self) -> Token<'data> { Token::BlockRound(self) }
}
impl<'data> SomeToken<'data> for BlockRoundToken<'data> {
    fn location(&self) -> Location { self.location }
    fn importance(&self) -> TokenImportance { self.importance() }
    fn wrap(self) -> Token<'data> { self.wrap() }
}
impl<'borrow, 'data> fmt::Display for TokenDisplay<'borrow, 'data, BlockRoundToken<'data>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("( .. )")
    }
}
impl<'borrow, 'data> fmt::Debug for TokenDisplay<'borrow, 'data, BlockRoundToken<'data>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("( .. )")
    }
}

#[derive(Debug, Clone)]
pub struct BlockSquareToken<'data> {
    pub location: Location,
    pub value: Vec<Token<'data>>,
}
impl<'data> BlockSquareToken<'data> {
    /// Create an already wrapped [Token].
    pub const fn new_wrapped(location: Location, value: Vec<Token<'data>>) -> Token<'data> {
        Token::BlockSquare(BlockSquareToken { location, value })
    }
    /// Create an unwrapped token.
    pub const fn new(location: Location, value: Vec<Token<'data>>) -> Self {
        Self { location, value }
    }
    /// Make this token render nice.
    ///
    /// Used in traces.
    pub const fn display<'borrow>(&'borrow self) -> TokenDisplay<'borrow, 'data, Self> { TokenDisplay::new(self) }
    /// This token's [TokenImportance].
    pub const IMPORTANCE: TokenImportance = TokenImportance::Code;
    /// Obtain this token's [Location].
    ///
    /// Used to exclude cosmetic components when parsing.
    pub const fn location(&self) -> Location { self.location }
    /// Obtain this token's [TokenImportance].
    ///
    /// Used to exclude cosmetic components when parsing.
    pub const fn importance(&self) -> TokenImportance { Self::IMPORTANCE }
    /// Wrap this token into [Token].
    pub const fn wrap(self) -> Token<'data> { Token::BlockSquare(self) }
}
impl<'data> SomeToken<'data> for BlockSquareToken<'data> {
    fn location(&self) -> Location { self.location }
    fn importance(&self) -> TokenImportance { self.importance() }
    fn wrap(self) -> Token<'data> { self.wrap() }
}
impl<'borrow, 'data> fmt::Display for TokenDisplay<'borrow, 'data, BlockSquareToken<'data>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[ .. ]")
    }
}
impl<'borrow, 'data> fmt::Debug for TokenDisplay<'borrow, 'data, BlockSquareToken<'data>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[ .. ]")
    }
}

#[derive(Debug, Clone)]
pub enum Token<'data> {
    /// `ident` or `'ident'` (replace quotes with graves)
    Ident(IdentToken<'data>),
    /// `"ident"` or `'ident'`
    String(StringToken<'data>),
    /// `:`, `,`, `.`, `==`, etc.
    Operator(OpToken<'data>),
    /// `# comment`, `// comment` or `/* comment */`.
    Comment(CommentToken<'data>),
    /// `/// doc-comment` or `## doc-comment`.
    Doc(DocToken<'data>),
    /// `{ that }`.
    BlockCurly(BlockCurlyToken<'data>),
    /// `( that )`.
    BlockRound(BlockRoundToken<'data>),
    /// `[ that ]`.
    BlockSquare(BlockSquareToken<'data>),
}
impl<'data> Token<'data> {
    /// Obtain this token's [TokenImportance].
    ///
    /// Used to exclude cosmetic components when parsing.
    pub const fn importance(&self) -> TokenImportance {
        match self {
            Token::Ident(x) => x.importance(),
            Token::String(x) => x.importance(),
            Token::Operator(x) => x.importance(),
            Token::Comment(x) => x.importance(),
            Token::Doc(x) => x.importance(),
            Token::BlockCurly(x) => x.importance(),
            Token::BlockRound(x) => x.importance(),
            Token::BlockSquare(x) => x.importance(),
        }
    }

    /// Obtain this token's [Location].
    ///
    /// Used to give usable traces.
    pub const fn location(&self) -> Location {
        match self {
            Token::Ident(x) => x.location(),
            Token::String(x) => x.location(),
            Token::Operator(x) => x.location(),
            Token::Comment(x) => x.location(),
            Token::Doc(x) => x.location(),
            Token::BlockCurly(x) => x.location(),
            Token::BlockRound(x) => x.location(),
            Token::BlockSquare(x) => x.location(),
        }
    }

    /// Make this token render nice.
    ///
    /// Used in traces.
    pub const fn display(&self) -> TokenDisplay<'_, 'data, Self> { TokenDisplay::new(self) }

    /// Wrap this token into [Token].
    pub const fn wrap(self) -> Token<'data> { self }

    /// Try convert this token to an [IdentToken].
    pub const fn as_ident(&self) -> Option<&IdentToken<'data>> {
        match self {
            Token::Ident(x) => Some(x),
            _ => None,
        }
    }

    /// Try convert this token to a [StringToken].
    pub const fn as_str(&self) -> Option<&StringToken<'data>> {
        match self {
            Token::String(x) => Some(x),
            _ => None,
        }
    }

    /// Try convert this token to an [OpToken].
    pub const fn as_op(&self) -> Option<&OpToken<'data>> {
        match self {
            Token::Operator(x) => Some(x),
            _ => None,
        }
    }

    /// Try convert this token to a [CommentToken].
    pub const fn as_comment(&self) -> Option<&CommentToken<'data>> {
        match self {
            Token::Comment(x) => Some(x),
            _ => None,
        }
    }

    /// Try convert this token to a [DocToken].
    pub const fn as_doc(&self) -> Option<&DocToken<'data>> {
        match self {
            Token::Doc(x) => Some(x),
            _ => None,
        }
    }

    /// Try convert this token to a [BlockCurlyToken].
    pub const fn as_curly_block(&self) -> Option<&BlockCurlyToken<'data>> {
        match self {
            Token::BlockCurly(x) => Some(x),
            _ => None,
        }
    }

    /// Try convert this token to a [BlockRoundToken].
    pub const fn as_round_block(&self) -> Option<&BlockRoundToken<'data>> {
        match self {
            Token::BlockRound(x) => Some(x),
            _ => None,
        }
    }

    /// Try convert this token to a [BlockSquareToken].
    pub const fn as_square_block(&self) -> Option<&BlockSquareToken<'data>> {
        match self {
            Token::BlockSquare(x) => Some(x),
            _ => None,
        }
    }
}
impl<'data> SomeToken<'data> for Token<'data> {
    fn location(&self) -> Location { self.location() }
    fn importance(&self) -> TokenImportance { self.importance() }
    fn wrap(self) -> Token<'data> { self.wrap() }
}
impl<'borrow, 'data> fmt::Display for TokenDisplay<'borrow, 'data, Token<'data>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            Token::Ident(x) => fmt::Display::fmt(&x.display(), f),
            Token::String(x) => fmt::Display::fmt(&x.display(), f),
            Token::Operator(x) => fmt::Display::fmt(&x.display(), f),
            Token::Comment(x) => fmt::Display::fmt(&x.display(), f),
            Token::Doc(x) => fmt::Display::fmt(&x.display(), f),
            Token::BlockCurly(x) => fmt::Display::fmt(&x.display(), f),
            Token::BlockRound(x) => fmt::Display::fmt(&x.display(), f),
            Token::BlockSquare(x) => fmt::Display::fmt(&x.display(), f),
        }
    }
}
impl<'borrow, 'data> fmt::Debug for TokenDisplay<'borrow, 'data, Token<'data>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            Token::Ident(x) => fmt::Debug::fmt(&x.display(), f),
            Token::String(x) => fmt::Debug::fmt(&x.display(), f),
            Token::Operator(x) => fmt::Debug::fmt(&x.display(), f),
            Token::Comment(x) => fmt::Debug::fmt(&x.display(), f),
            Token::Doc(x) => fmt::Debug::fmt(&x.display(), f),
            Token::BlockCurly(x) => fmt::Debug::fmt(&x.display(), f),
            Token::BlockRound(x) => fmt::Debug::fmt(&x.display(), f),
            Token::BlockSquare(x) => fmt::Debug::fmt(&x.display(), f),
        }
    }
}

pub struct TokenDisplay<'borrow, 'data, T: SomeToken<'data> + ?Sized> {
    kind: &'borrow T,
    _phantom: PhantomData<&'borrow Token<'data>>,
}
impl<'borrow, 'data, T: SomeToken<'data> + ?Sized> TokenDisplay<'borrow, 'data, T> {
    const fn new(token: &'borrow T) -> Self { TokenDisplay { kind: token, _phantom: PhantomData } }
}

/// Parse input text into tokens.
pub fn into_tokens<'data>(data: &'data str) -> Result<'data, 'data, Vec<Token<'data>>> {
    IntoTokens { str: data, chars: Chars {
        row: 1,
        column: 1,
        iter: data.char_indices() }.peekable(),
        end: '\0',
        nesting: 0,
        prev_block: Location { idx: 0, column: 0, row: 0 },
    }.finish_parsing()
}

struct Chars<'a> {
    iter: std::str::CharIndices<'a>,
    row: usize,
    column: usize,
}
impl<'a> Iterator for Chars<'a> {
    type Item = Char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some((idx, c)) => {
                let char = Char {
                    c,
                    loc: Location { row: self.row, column: self.column, idx },
                };
                if c == '\n' {
                    self.row += 1;
                    self.column = 1;
                } else {
                    self.column += 1;
                }
                Some(char)
            },
            None => None,
        }
    }
}

pub struct IntoTokens<'data> {
    str: &'data str,
    chars: Peekable<Chars<'data>>,
    prev_block: Location,
    end: char,
    nesting: u16,
}
impl<'a> IntoTokens<'a> {
    pub fn finish_parsing(self) -> Result<'a, 'a, Vec<Token<'a>>> {
        enum Storing<'a> {
            Tokens(Vec<Token<'a>>),
            Errors(Vec<Error<'a, 'a>>),
        }
        let mut ret = Storing::Tokens(Vec::new());

        for x in self {
            match x {
                Ok(token) => if let Storing::Tokens(ret) = &mut ret { ret.push(token); },
                Err(why) => match &mut ret {
                    Storing::Tokens(_) => ret = Storing::Errors(vec![why]),
                    Storing::Errors(errors) => errors.push(why),
                },
            };
        }

        match ret {
            Storing::Tokens(tokens) => Ok(tokens),
            Storing::Errors(mut errors) => if errors.len() == 1 {
                Err(errors.pop().unwrap())
            } else {
                Err(Error::MultipleErrors(errors))
            },
        }
    }
}
impl<'a> Iterator for IntoTokens<'a> {
    type Item = Result<'a, 'a, Token<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.chars.peek().is_some_and(|x| x.is_whitespace()) {
            self.chars.next();
        }

        let Some(v) = self.chars.next() else {
            return if self.end == '\0' {
                None
            } else {
                Some(Err(Error::UnenclosedBlock {
                    start: self.prev_block,
                    pos: None,
                    expected: self.end,
                    err_code: "next-block-0",
                }))
            };
        };

        match v.c {
            x if x == self.end && self.end != '\0' => return None,
            ')' | ']' | '}' => return Some(Err(Error::UnexpectedParen {
                char: v.c,
                expected: self.end,
                pos: v.loc,
                err_code: "next-char-paren",
            })),
            '(' | '[' | '{' => {
                self.nesting += 1;
                if self.nesting >= 128 {
                    return Some(Err(Error::Nesting { loc: v.loc, err_code: "nesting-paren" }));
                }

                let old_end = self.end;
                let old_loc = self.prev_block;

                self.end = match v.c {
                    '(' => ')',
                    '[' => ']',
                    '{' => '}',
                    _ => unreachable!(),
                };
                self.prev_block = v.loc;

                let mut tokens = Vec::new();
                let mut errors = Vec::new();
                loop { match self.next() {
                    None => break,
                    Some(Ok(x)) => tokens.push(x),
                    Some(Err(why)) => {
                        let r = why.is_recoverable();
                        errors.push(why);
                        if r { break; }
                    }
                } }

                self.end = old_end;
                self.prev_block = old_loc;
                self.nesting -= 1;

                if errors.is_empty() {
                    match v.c {
                        '(' => Some(Ok(BlockRoundToken::new_wrapped(v.loc, tokens))),
                        '[' => Some(Ok(BlockSquareToken::new_wrapped(v.loc, tokens))),
                        '{' => Some(Ok(BlockCurlyToken::new_wrapped(v.loc, tokens))),
                        _ => unreachable!(),
                    }
                } else {
                    Some(Err(Error::MultipleErrors(errors)))
                }
            }
            '\'' => {
                let mut buf = None;
                let mut len = 0usize;
                let mut backspace = false;

                loop {
                    let Some(w) = self.chars.next() else {
                        return Some(Err(Error::UnenclosedString {
                            start: v.loc,
                            pos: None,
                            err_code: "str-single-none",
                        }));
                    };

                    match w.c {
                        '\n' => return Some(Err(Error::UnenclosedString {
                            start: v.loc,
                            pos: Some(w.loc),
                            err_code: "str-single-newline",
                        })),
                        '\'' if !backspace => break,
                        '\\' if !backspace => {
                            backspace = true;
                            if buf.is_none() {
                                buf = Some(self.str[1 + v.loc.idx..][..len].to_string());
                            }
                            continue;
                        }
                        _ if backspace => if let Some(buf) = buf.as_mut() {
                            buf.push('\\');
                        } else {
                            len += 1;
                        },
                        _ => (),
                    }
                    backspace = false;

                    if let Some(buf) = buf.as_mut() {
                        buf.push(w.c);
                    } else {
                        len += w.c.len_utf8();
                    }
                }

                Some(Ok(StringToken::new_wrapped(v.loc, if let Some(x) = buf {
                    Str::Owned(x.into_boxed_str())
                } else {
                    Str::Borrowed(&self.str[1 + v.loc.idx..][..len])
                })))
            }
            '"' => {
                let mut buf = None;
                let mut len = 0usize;
                let mut backspace = false;

                loop {
                    let Some(w) = self.chars.next() else {
                        return Some(Err(Error::UnenclosedString {
                            start: v.loc,
                            pos: None,
                            err_code: "str-double-none",
                        }));
                    };

                    let mut chr = w.c;

                    match w.c {
                        '\n' => return Some(Err(Error::UnenclosedString {
                            start: v.loc,
                            pos: Some(w.loc),
                            err_code: "str-double-netline",
                        })),
                        '\"' if !backspace => break,
                        'r' if backspace => {
                            backspace = false;
                            if buf.is_none() {
                                buf = Some(self.str[1 + v.loc.idx..][..len].to_string());
                            }
                            chr = '\r';
                        }
                        'n' if backspace => {
                            backspace = false;
                            if buf.is_none() {
                                buf = Some(self.str[1 + v.loc.idx..][..len].to_string());
                            }
                            chr = '\n';
                        }
                        '0' if backspace => {
                            backspace = false;
                            if buf.is_none() {
                                buf = Some(self.str[1 + v.loc.idx..][..len].to_string());
                            }
                            chr = '\0';
                        }
                        'x' if backspace => {
                            backspace = false;
                            if buf.is_none() {
                                buf = Some(self.str[1 + v.loc.idx..][..len].to_string());
                            }
                            let Some(first) = self.chars.next() else {
                                return Some(Err(Error::UnfinishedEscapeX {
                                    start: w.loc,
                                    idx: 0,
                                    err_code: "str-double-x-first-none",
                                }));
                            };
                            let Some(second) = self.chars.next() else {
                                return Some(Err(Error::UnfinishedEscapeX {
                                    start: w.loc,
                                    idx: 1,
                                    err_code: "str-double-x-second-none",
                                }));
                            };
                            chr = (16 * {
                                if ('0'..='9').contains(&*first) { *first as u8 - b'0' }
                                else if ('a'..='f').contains(&*first) { *first as u8 - b'a' + 10 }
                                else if ('A'..='F').contains(&*first) { *first as u8 - b'A' + 10 }
                                else {
                                    return Some(Err(Error::EscapeXInvalidChar {
                                        start: w.loc,
                                        idx: 0,
                                        err_code: "str-double-x-first-invalid",
                                    }));
                                }
                            } + {
                                if ('0'..='9').contains(&*second) { *second as u8 - b'0' }
                                else if ('a'..='f').contains(&*second) { *second as u8 - b'a' + 10 }
                                else if ('A'..='F').contains(&*second) { *second as u8 - b'A' + 10 }
                                else {
                                    return Some(Err(Error::EscapeXInvalidChar {
                                        start: w.loc,
                                        idx: 1,
                                        err_code: "str-double-x-second-invalid",
                                    }));
                                }
                            }) as char;
                        }
                        'u' if backspace => {
                            backspace = false;
                            if buf.is_none() {
                                buf = Some(self.str[1 + v.loc.idx..][..len].to_string());
                            }
                            let Some(first) = self.chars.next() else {
                                return Some(Err(Error::UnfinishedEscapeU {
                                    start: w.loc,
                                    idx: 0,
                                    err_code: "str-double-u-first-none",
                                }));
                            };
                            let Some(second) = self.chars.next() else {
                                return Some(Err(Error::UnfinishedEscapeU {
                                    start: w.loc,
                                    idx: 1,
                                    err_code: "str-double-u-second-none",
                                }));
                            };
                            let Some(third) = self.chars.next() else {
                                return Some(Err(Error::UnfinishedEscapeU {
                                    start: w.loc,
                                    idx: 2,
                                    err_code: "str-double-u-third-none",
                                }));
                            };
                            let Some(forth) = self.chars.next() else {
                                return Some(Err(Error::UnfinishedEscapeU {
                                    start: w.loc,
                                    idx: 3,
                                    err_code: "str-double-u-forth-none",
                                }));
                            };
                            let code = 16 * (16 * (
                                    16 * {
                                        if ('0'..='9').contains(&*first) { *first as u32 - b'0' as u32 }
                                        else if ('a'..='f').contains(&*first) { *first as u32 - b'a' as u32 + 10 }
                                        else if ('A'..='F').contains(&*first) { *first as u32 - b'A' as u32 + 10 }
                                        else {
                                            return Some(Err(Error::EscapeUInvalidChar {
                                                start: w.loc,
                                                idx: 0,
                                                err_code: "str-double-u-first-invalid",
                                            }));
                                        }
                                    } +
                                    {
                                        if ('0'..='9').contains(&*second) { *second as u32 - b'0' as u32 }
                                        else if ('a'..='f').contains(&*second) { *second as u32 - b'a' as u32 + 10 }
                                        else if ('A'..='F').contains(&*second) { *second as u32 - b'A' as u32 + 10 }
                                        else {
                                            return Some(Err(Error::EscapeUInvalidChar {
                                                start: w.loc,
                                                idx: 1,
                                                err_code: "str-double-u-second-invalid",
                                            }));
                                        }
                                    }
                                ) +
                                {
                                    if ('0'..='9').contains(&*third) { *third as u32 - b'0' as u32 }
                                    else if ('a'..='f').contains(&*third) { *third as u32 - b'a' as u32 + 10 }
                                    else if ('A'..='F').contains(&*third) { *third as u32 - b'A' as u32 + 10 }
                                    else {
                                        return Some(Err(Error::EscapeUInvalidChar {
                                            start: w.loc,
                                            idx: 2,
                                            err_code: "str-double-u-third-invalid",
                                        }));
                                    }
                                }
                            ) + {
                                if ('0'..='9').contains(&*forth) { *forth as u32 - b'0' as u32 }
                                else if ('a'..='f').contains(&*forth) { *forth as u32 - b'a' as u32 + 10 }
                                else if ('A'..='F').contains(&*forth) { *forth as u32 - b'A' as u32 + 10 }
                                else {
                                    return Some(Err(Error::EscapeUInvalidChar {
                                        start: w.loc,
                                        idx: 3,
                                        err_code: "str-double-u-forth-invalid",
                                    }));
                                }
                            };
                            if let Some(x) = char::from_u32(code) { chr = x; } else {
                                return Some(Err(Error::EscapeUInvalidCode {
                                    start: w.loc,
                                    provided: code,
                                    err_code: "str-double-u-char-invalid",
                                }));
                            };
                        }
                        _ if backspace => return Some(Err(Error::EscapeUInvalidEscape {
                            start: w.loc,
                            provided: w.c,
                            err_code: "str-double-u-esc-invalid",
                        })),
                        '\\' if !backspace => {
                            backspace = true;
                            if buf.is_none() {
                                buf = Some(self.str[1 + v.loc.idx..][..len].to_string());
                            }
                            continue;
                        }
                        _ => (),
                    }

                    if let Some(buf) = buf.as_mut() {
                        buf.push(chr);
                    } else {
                        len += chr.len_utf8();
                    }
                }

                Some(Ok(StringToken::new_wrapped(v.loc, if let Some(x) = buf {
                    Str::Owned(x.into_boxed_str())
                } else {
                    Str::Borrowed(&self.str[1 + v.loc.idx..][..len])
                })))
            }
            x if x.is_ascii_punctuation() && x != '$' => {
                match x {
                    '=' | '<' | '>' | '+' | '-' | '*' => match self.chars.peek() {
                        Some(w) if w.c == '=' => {
                            self.chars.next();
                            Some(Ok(OpToken::new_wrapped(v.loc, &self.str[v.loc.idx..][..2])))
                        },
                        _ => Some(Ok(OpToken::new_wrapped(v.loc, &self.str[v.loc.idx..][..x.len_utf8()]))),
                    }
                    // TODO: Comment
                    '/' => match self.chars.peek() {
                        Some(w) if w.c == '=' => {
                            self.chars.next();
                            Some(Ok(OpToken::new_wrapped(v.loc, &self.str[v.loc.idx..][..2])))
                        },
                        _ => Some(Ok(OpToken::new_wrapped(v.loc, &self.str[v.loc.idx..][..x.len_utf8()]))),
                    }
                    ':' => match self.chars.peek() {
                        Some(w) if w.c == ':' => {
                            self.chars.next();
                            Some(Ok(OpToken::new_wrapped(v.loc, &self.str[v.loc.idx..][..2])))
                        },
                        _ => Some(Ok(OpToken::new_wrapped(v.loc, &self.str[v.loc.idx..][..x.len_utf8()]))),
                    }
                    x => Some(Ok(OpToken::new_wrapped(v.loc, &self.str[v.loc.idx..][..x.len_utf8()]))),
                }
            },
            x => {
                let mut len = x.len_utf8();

                while self.chars.peek().is_some_and(|x| !x.is_ascii_punctuation() && !x.is_whitespace() || x.c == '$') {
                    let w = self.chars.next().unwrap();
                    len += w.c.len_utf8();
                }

                Some(Ok(IdentToken::new_wrapped(v.loc, Str::Borrowed(&self.str[v.loc.idx..][..len]))))
            }
        }
    }
}
