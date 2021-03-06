pub use self::BinOpToken::*;
pub use self::DelimToken::*;
pub use self::Lit::*;
pub use self::Token::*;

use std::fmt;
use std::iter;
use std::ops::Deref;
use std::rc::Rc;

use ast;
use interner::{self, StrInterner, RcStr};

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum BinOpToken {
  Plus,
  Minus,
  Star,
  Slash,
  Percent,
  Caret,
  And,
  Or,
  LShift,
  RShift
}

/// A delimiter token
#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum DelimToken {
  /// A round parenthesis: `(` or `)`
  Paren,
  /// A square bracket: `[` or `]`
  Bracket,
  /// A curly brace: `{` or `}`
  Brace,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum Lit {
  Byte(ast::Name),
  Char(ast::Name),
  Integer(ast::Name),
  Float(ast::Name),
  Str_(ast::Name),
  StrRaw(ast::Name, usize), /* raw str delimited by n hash symbols */
  ByteStr(ast::Name),
  ByteStrRaw(ast::Name, usize), /* raw byte str delimited by n hash symbols */
}

impl Lit {
  pub fn short_name(&self) -> &'static str {
    match *self {
      Byte(_) => "byte",
      Char(_) => "char",
      Integer(_) => "integer",
      Float(_) => "float",
      Str_(_) | StrRaw(..) => "string",
      ByteStr(_) | ByteStrRaw(..) => "byte string"
    }
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Token {
  /* Expression-operator symbols. */
  Eq,
  EqEq,
  Ne,
  Lt,
  Le,
  Ge,
  Gt,
  AndAnd,
  OrOr,
  Not,
  Tilde,
  BinOp(BinOpToken),
  BinOpEq(BinOpToken), // e.g. '+='

  /* Structural symbols */
  At,       // @
  Colon,    // :
  SemiColon,// ;
  Comma,    // ,
  Dot,      // .
  DotDot,   // ..
  DotDotDot,// ...
  Dollar,   // $
  Pound,    // #
  Question, // ?
  ModSep,   // ::
  LArrow,   // <-
  RArrow,   // ->
  FatArrow, // =>

  /// An opening delimiter, eg. `{`
  OpenDelim(DelimToken),
  /// A closing delimiter, eg. `}`
  CloseDelim(DelimToken),

  /* Literals */
  Literal(Lit, Option<ast::Name>),

  /* Name components */
  Ident(ast::Ident),
  Underscore,

  /// Whitespace
  Whitespace,
  // Can be expanded into several tokens.
  /// Doc comment
  DocComment(ast::Name),
  /// Comment
  Comment,

  /// End of file
  Eof,
}

impl Token {

  /// Returns `true` if the token can appear at the start of an expression.
  pub fn can_begin_expr(&self) -> bool {
    match *self {
      OpenDelim(_)                => true,
      Ident(..)                   => true,
      Underscore                  => true,
      Tilde                       => true,
      Literal(_, _)               => true,
      Not                         => true,
      BinOp(Minus)                => true,
      BinOp(Star)                 => true,
      BinOp(And)                  => true,
      BinOp(Or)                   => true, // in lambda syntax
      OrOr                        => true, // in lambda syntax
      AndAnd                      => true, // double borrow
      DotDot | DotDotDot          => true, // range notation
      ModSep                      => true,
      Pound                       => true, // for expression attributes
      _                           => false,
    }
  }

  /// Returns `true` if the token is any literal
  pub fn is_lit(&self) -> bool {
    match *self {
      Literal(_, _) => true,
      _          => false,
    }
  }

  /// Returns `true` if the token is an identifier.
  pub fn is_ident(&self) -> bool {
    match *self {
      Ident(_)    => true,
      _           => false,
    }
  }

  /// Returns `true` if the token is an interpolated path.
  pub fn is_path(&self) -> bool {
    match *self {
      //Interpolated(NtPath(..))    => true,
      _                           => false,
    }
  }

  pub fn is_path_start(&self) -> bool {
    self == &ModSep || self == &Lt ||
    self.is_path_segment_keyword() || self.is_ident() && !self.is_any_keyword()
  }

  pub fn is_path_segment_keyword(&self) -> bool {
    match *self {
      Ident(id) => id.name == keywords::Super.name() ||
      id.name == keywords::SelfValue.name() ||
      id.name == keywords::SelfType.name(),
      _ => false,
    }
  }

  /// Returns `true` if the token is a given keyword, `kw`.
  pub fn is_keyword(&self, kw: keywords::Keyword) -> bool {
    match *self {
      Ident(id) => id.name == kw.name(),
      _ => false,
    }
  }

  /// Returns `true` if the token is either a strict or reserved keyword.
  pub fn is_any_keyword(&self) -> bool {
    self.is_strict_keyword() || self.is_reserved_keyword()
  }

  /// Returns `true` if the token is a strict keyword.
  pub fn is_strict_keyword(&self) -> bool {
    match *self {
      Ident(id) => id.name >= keywords::As.name() &&
      id.name <= keywords::While.name(),
      _ => false,
    }
  }

  /// Returns `true` if the token is a keyword reserved for possible future use.
  pub fn is_reserved_keyword(&self) -> bool {
    match *self {
      Ident(id) => id.name >= keywords::Abstract.name() &&
      id.name <= keywords::Yield.name(),
      _ => false,
    }
  }
}

// In this macro, there is the requirement that the name (the number) must be monotonically
// increasing by one in the special identifiers, starting at 0; the same holds for the keywords,
// except starting from the next number instead of zero.
macro_rules! declare_keywords {(
    $( ($index: expr, $konst: ident, $string: expr) )*
) => {
    pub mod keywords {
        use ast;
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub struct Keyword {
            ident: ast::Ident,
        }
        impl Keyword {
            #[inline] pub fn ident(self) -> ast::Ident { self.ident }
            #[inline] pub fn name(self) -> ast::Name { self.ident.name }
        }
        $(
            #[allow(non_upper_case_globals)]
            pub const $konst: Keyword = Keyword {
                ident: ast::Ident::with_empty_ctxt(ast::Name($index))
            };
        )*
    }

    fn mk_fresh_ident_interner() -> IdentInterner {
        interner::StrInterner::prefill(&[$($string,)*])
    }
}}

// NB: leaving holes in the ident table is bad! a different ident will get
// interned with the id from the hole, but it will be between the min and max
// of the reserved words, and thus tagged as "reserved".
// After modifying this list adjust `is_strict_keyword`/`is_reserved_keyword`,
// this should be rarely necessary though if the keywords are kept in alphabetic order.
declare_keywords! {
    // Invalid identifier
    (0,  Invalid,        "")

    // Strict keywords used in the language.
    (1,  As,             "as")
    (2,  Box,            "box")
    (3,  Break,          "break")
    (4,  Const,          "const")
    (5,  Continue,       "continue")
    (6,  Crate,          "crate")
    (7,  Else,           "else")
    (8,  Enum,           "enum")
    (9,  Extern,         "extern")
    (10, False,          "false")
    (11, Fn,             "fn")
    (12, For,            "for")
    (13, If,             "if")
    (14, Impl,           "impl")
    (15, Import,         "import")
    (16, In,             "in")
    (17, Let,            "let")
    (18, Loop,           "loop")
    (19, Match,          "match")
    (20, Mod,            "mod")
    (21, Move,           "move")
    (22, Mut,            "mut")
    (23, Pub,            "pub")
    (24, Ref,            "ref")
    (25, Return,         "return")
    (26, SelfValue,      "self")
    (27, SelfType,       "Self")
    (28, Static,         "static")
    (29, Struct,         "struct")
    (30, Super,          "super")
    (31, Trait,          "trait")
    (32, True,           "true")
    (33, Type,           "type")
    (34, Unsafe,         "unsafe")
    (35, Var,            "var")
    (36, Where,          "where")
    (37, While,          "while")

    // Keywords reserved for future use.
    (38, Abstract,       "abstract")
    (39, Alignof,        "alignof")
    (40, Become,         "become")
    (41, Do,             "do")
    (42, Final,          "final")
    (43, Macro,          "macro")
    (44, Offsetof,       "offsetof")
    (45, Override,       "override")
    (46, Priv,           "priv")
    (47, Proc,           "proc")
    (48, Pure,           "pure")
    (49, Sizeof,         "sizeof")
    (50, Typeof,         "typeof")
    (51, Unsized,        "unsized")
    (52, Virtual,        "virtual")
    (53, Yield,          "yield")

    // Weak keywords, have special meaning only in specific contexts.
    (54, Default,        "default")
    (55, StaticLifetime, "'static")
    (56, Union,          "union")
}

// looks like we can get rid of this completely...
pub type IdentInterner = StrInterner;

// if an interner exists in TLS, return it. Otherwise, prepare a
// fresh one.
// FIXME(eddyb) #8726 This should probably use a thread-local reference.
pub fn get_ident_interner() -> Rc<IdentInterner> {
  thread_local!(static KEY: Rc<::token::IdentInterner> = {
        Rc::new(mk_fresh_ident_interner())
    });
  KEY.with(|k| k.clone())
}

/// Reset the ident interner to its initial state.
pub fn reset_ident_interner() {
  let interner = get_ident_interner();
  interner.reset(mk_fresh_ident_interner());
}

/// Represents a string stored in the thread-local interner. Because the
/// interner lives for the life of the thread, this can be safely treated as an
/// immortal string, as long as it never crosses between threads.
///
/// FIXME(pcwalton): You must be careful about what you do in the destructors
/// of objects stored in TLS, because they may run after the interner is
/// destroyed. In particular, they must not access string contents. This can
/// be fixed in the future by just leaking all strings until thread death
/// somehow.
#[derive(Clone, PartialEq, Hash, PartialOrd, Eq, Ord)]
pub struct InternedString {
  string: RcStr,
}

impl InternedString {
  #[inline]
  pub fn new(string: &'static str) -> InternedString {
    InternedString {
      string: RcStr::new(string),
    }
  }

  #[inline]
  fn new_from_rc_str(string: RcStr) -> InternedString {
    InternedString {
      string: string,
    }
  }

  #[inline]
  pub fn new_from_name(name: ast::Name) -> InternedString {
    let interner = get_ident_interner();
    InternedString::new_from_rc_str(interner.get(name))
  }
}

impl Deref for InternedString {
  type Target = str;

  fn deref(&self) -> &str { &*self.string }
}

impl fmt::Debug for InternedString {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Debug::fmt(&self.string, f)
  }
}

impl fmt::Display for InternedString {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Display::fmt(&self.string, f)
  }
}

impl<'a> PartialEq<&'a str> for InternedString {
  #[inline(always)]
  fn eq(&self, other: & &'a str) -> bool {
    PartialEq::eq(&self.string[..], *other)
  }
  #[inline(always)]
  fn ne(&self, other: & &'a str) -> bool {
    PartialEq::ne(&self.string[..], *other)
  }
}

impl<'a> PartialEq<InternedString> for &'a str {
  #[inline(always)]
  fn eq(&self, other: &InternedString) -> bool {
    PartialEq::eq(*self, &other.string[..])
  }
  #[inline(always)]
  fn ne(&self, other: &InternedString) -> bool {
    PartialEq::ne(*self, &other.string[..])
  }
}

impl PartialEq<str> for InternedString {
  #[inline(always)]
  fn eq(&self, other: &str) -> bool {
    PartialEq::eq(&self.string[..], other)
  }
  #[inline(always)]
  fn ne(&self, other: &str) -> bool {
    PartialEq::ne(&self.string[..], other)
  }
}

impl PartialEq<InternedString> for str {
  #[inline(always)]
  fn eq(&self, other: &InternedString) -> bool {
    PartialEq::eq(self, &other.string[..])
  }
  #[inline(always)]
  fn ne(&self, other: &InternedString) -> bool {
    PartialEq::ne(self, &other.string[..])
  }
}

/// Interns and returns the string contents of an identifier, using the
/// thread-local interner.
#[inline]
pub fn intern_and_get_ident(s: &str) -> InternedString {
  intern(s).as_str()
}

/// Maps a string to its interned representation.
#[inline]
pub fn intern(s: &str) -> ast::Name {
  get_ident_interner().intern(s)
}

/// gensym's a new usize, using the current interner.
#[inline]
pub fn gensym(s: &str) -> ast::Name {
  get_ident_interner().gensym(s)
}

/// Maps a string to an identifier with an empty syntax context.
#[inline]
pub fn str_to_ident(s: &str) -> ast::Ident {
  ast::Ident::with_empty_ctxt(intern(s))
}

/// Maps a string to a gensym'ed identifier.
#[inline]
pub fn gensym_ident(s: &str) -> ast::Ident {
  ast::Ident::with_empty_ctxt(gensym(s))
}

fn repeat(s: &str, n: usize) -> String { iter::repeat(s).take(n).collect() }

pub fn binop_to_string(op: BinOpToken) -> &'static str {
  match op {
    Plus     => "+",
    Minus    => "-",
    Star     => "*",
    Slash    => "/",
    Percent  => "%",
    Caret    => "^",
    And      => "&",
    Or       => "|",
    LShift   => "<<",
    RShift   => ">>",
  }
}

pub fn token_to_string(tok: &Token) -> String {
  match *tok {
    Eq                   => "=".to_string(),
    Lt                   => "<".to_string(),
    Le                   => "<=".to_string(),
    EqEq                 => "==".to_string(),
    Ne                   => "!=".to_string(),
    Ge                   => ">=".to_string(),
    Gt                   => ">".to_string(),
    Not                  => "!".to_string(),
    Tilde                => "~".to_string(),
    OrOr                 => "||".to_string(),
    AndAnd               => "&&".to_string(),
    BinOp(op)            => binop_to_string(op).to_string(),
    BinOpEq(op)          => format!("{}=", binop_to_string(op)),

    /* Structural symbols */
    At                   => "@".to_string(),
    Dot                  => ".".to_string(),
    DotDot               => "..".to_string(),
    DotDotDot            => "...".to_string(),
    Comma                => ",".to_string(),
    SemiColon            => ";".to_string(),
    Colon                => ":".to_string(),
    ModSep               => "::".to_string(),
    RArrow               => "->".to_string(),
    LArrow               => "<-".to_string(),
    FatArrow             => "=>".to_string(),
    OpenDelim(Paren)     => "(".to_string(),
    CloseDelim(Paren)    => ")".to_string(),
    OpenDelim(Bracket)   => "[".to_string(),
    CloseDelim(Bracket)  => "]".to_string(),
    OpenDelim(Brace)     => "{".to_string(),
    CloseDelim(Brace)    => "}".to_string(),
    Pound                => "#".to_string(),
    Dollar               => "$".to_string(),
    Question             => "?".to_string(),

    /* Literals */
    Literal(lit, suf) => {
      let mut out = match lit {
        Byte(b)           => format!("b'{}'", b),
        Char(c)           => format!("'{}'", c),
        Float(c)          => c.to_string(),
        Integer(c)        => c.to_string(),
        Str_(s)           => format!("\"{}\"", s),
        StrRaw(s, n)      => format!("r{delim}\"{string}\"{delim}",  delim=repeat("#", n), string=s),
        ByteStr(v)        => format!("b\"{}\"", v),
        ByteStrRaw(s, n)  => format!("br{delim}\"{string}\"{delim}", delim=repeat("#", n), string=s),
      };

      if let Some(s) = suf {
        out.push_str(&s.as_str())
      }

      out
    }

    /* Name components */
    Ident(s)             => s.to_string(),
    Underscore           => "_".to_string(),

    /* Other */
    DocComment(s)        => s.to_string(),
    Comment              => "/* */".to_string(),
    Whitespace           => " ".to_string(),
    Eof                  => "<eof>".to_string(),
  }
}