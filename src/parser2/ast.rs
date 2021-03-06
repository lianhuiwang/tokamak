use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

pub use self::ViewPath_::*;
pub use self::Mutability::*;

use abi::Abi;
use ast_printer as printer;
use attr::ThinAttributes;
use codemap::{Span, Spanned};
use comments::{doc_comment_style, strip_doc_comment_decoration};
use ptr::P;
use token::{self, InternedString};

pub type NodeId = u32;

/// Node id used to represent the root of the package.
pub const PKG_NODE_ID: NodeId = 0;

/// When parsing and doing expansions, we initially give all AST nodes this AST
/// node value. Then later, in the renumber pass, we renumber them to have
/// small, positive ids.
pub const DUMMY_NODE_ID: NodeId = !0;

/// A name is a part of an identifier, representing a string or gensym. It's
/// the result of interning.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(pub u32);

impl Name {
  pub fn as_str(self) -> token::InternedString {
    token::InternedString::new_from_name(self)
  }
}

impl fmt::Debug for Name {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}({})", self, self.0)
  }
}

impl fmt::Display for Name {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Display::fmt(&self.as_str(), f)
  }
}

/// A SyntaxContext represents a chain of macro-expandings
/// and renamings. Each macro expansion corresponds to
/// a fresh u32. This u32 is a reference to a table stored
/// in thread-local storage.
/// The special value EMPTY_CTXT is used to indicate an empty
/// syntax context.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SyntaxContext(pub u32);

/// An identifier contains a Name (index into the interner
/// table) and a SyntaxContext to track renaming and
/// macro expansion per Flatt et al., "Macros That Work Together"
#[derive(Clone, Copy, Eq)]
pub struct Ident {
  pub name: Name,
  pub ctxt: SyntaxContext
}

pub const EMPTY_CTXT : SyntaxContext = SyntaxContext(0);

impl Ident {
  pub fn new(name: Name, ctxt: SyntaxContext) -> Ident {
    Ident {name: name, ctxt: ctxt}
  }
  pub const fn with_empty_ctxt(name: Name) -> Ident {
    Ident {name: name, ctxt: EMPTY_CTXT}
  }
}

impl PartialEq for Ident {
  fn eq(&self, other: &Ident) -> bool {
    if self.ctxt != other.ctxt {
      // There's no one true way to compare Idents. They can be compared
      // non-hygienically `id1.name == id2.name`, hygienically
      // `mtwt::resolve(id1) == mtwt::resolve(id2)`, or even member-wise
      // `(id1.name, id1.ctxt) == (id2.name, id2.ctxt)` depending on the situation.
      // Ideally, PartialEq should not be implemented for Ident at all, but that
      // would be too impractical, because many larger structures (Token, in particular)
      // including Idents as their parts derive PartialEq and use it for non-hygienic
      // comparisons. That's why PartialEq is implemented and defaults to non-hygienic
      // comparison. Hash is implemented too and is consistent with PartialEq, i.e. only
      // the name of Ident is hashed. Still try to avoid comparing idents in your code
      // (especially as keys in hash maps), use one of the three methods listed above
      // explicitly.
      //
      // If you see this panic, then some idents from different contexts were compared
      // non-hygienically. It's likely a bug. Use one of the three comparison methods
      // listed above explicitly.

      panic!("idents with different contexts are compared with operator `==`: \
                {:?}, {:?}.", self, other);
    }

    self.name == other.name
  }
}

impl Hash for Ident {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.name.hash(state)
  }
}

impl fmt::Debug for Ident {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}#{}", self.name, self.ctxt.0)
  }
}

impl fmt::Display for Ident {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Display::fmt(&self.name, f)
  }
}

pub type ViewPath = Spanned<ViewPath_>;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ViewPath_ {

  /// `foo::bar::baz as quux`
  ///
  /// or just
  ///
  /// `foo::bar::baz` (with `as baz` implicitly on the right)
  ViewPathSimple(Ident, Path),

  /// `foo::bar::*`
  ViewPathGlob(Path),

  /// `foo::bar::{a,b,c}`
  ViewPathList(Path, Vec<PathListItem>)
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum PathListItemKind {
  Ident {
    name: Ident,
    /// renamed in list, eg `use foo::{bar as baz};`
    rename: Option<Ident>,
    id: NodeId
  },
  Mod {
    /// renamed in list, eg `use foo::{self as baz};`
    rename: Option<Ident>,
    id: NodeId
  }
}

impl PathListItemKind {
  pub fn id(&self) -> NodeId {
    match *self {
      PathListItemKind::Ident { id, .. } | PathListItemKind::Mod { id, .. } => id
    }
  }

  pub fn name(&self) -> Option<Ident> {
    match *self {
      PathListItemKind::Ident { name, .. } => Some(name),
      PathListItemKind::Mod { .. } => None,
    }
  }

  pub fn rename(&self) -> Option<Ident> {
    match *self {
      PathListItemKind::Ident { rename, .. } | PathListItemKind::Mod { rename, .. } => rename
    }
  }
}

pub type PathListItem = Spanned<PathListItemKind>;

/// A "Path" is essentially Rust's notion of a name; for instance:
/// std::cmp::PartialEq  .  It's represented as a sequence of identifiers,
/// along with a bunch of supporting information.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Path {
  pub span: Span,
  /// A `::foo` path, is relative to the crate root rather than current
  /// module (like paths in an import).
  pub global: bool,
  /// The segments in the path: the things separated by `::`.
  pub segments: Vec<PathSegment>,
}

impl fmt::Debug for Path {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "path({})", printer::path_to_string(self))
  }
}

impl fmt::Display for Path {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", printer::path_to_string(self))
  }
}

/// A segment of a path: an identifier, an optional lifetime, and a set of
/// types.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PathSegment {
  /// The identifier portion of this path segment.
  pub identifier: Ident
}

impl fmt::Debug for PathSegment {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "PathSegment({})", self.identifier.name)
  }
}

impl fmt::Display for PathSegment {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "PathSegment({})", self.identifier.name)
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Package {
  pub module: Module,
  pub span: Span,
  pub attrs: Vec<Attribute>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Module {
  /// A span from the first token past `{` to the last token until `}`.
  /// For `mod foo;`, the inner span ranges from the first token
  /// to the last token in the external file.
  pub inner: Span,
  pub items: Vec<P<Item>>
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Visibility {
  Public,
  Inherited,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Item {
  pub ident: Ident,
  pub attrs: Vec<Attribute>,
  pub id: NodeId,
  pub node: ItemKind,
  pub vis: Visibility,
  pub span: Span,
}

impl Item {
  pub fn attrs(&self) -> &[Attribute] {
    &self.attrs
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ItemKind {
  /// A `use` or `pub use` item
  Import(P<ViewPath>),
  /// An external module
  ForeignMod(ForeignMod),
  /// A `static` item
  Static(P<Ty>, Mutability, P<Expr>),
  /// A `const` item
  Const(P<Ty>, P<Expr>),
  /// A function declaration
  Fn(P<FnDecl>, Unsafety, Constness, Abi, P<Block>),
  /// A type alias, e.g. `type Foo = Bar`
  Ty(P<Ty>),
  Enum,
  Struct,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ForeignMod {
  pub abi: Abi,
  pub items: Vec<ForeignItem>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ForeignItem {
  pub ident: Ident,
  pub attrs: Vec<Attribute>,
  pub node: ForeignItemKind,
  pub id: NodeId,
  pub span: Span,
  pub vis: Visibility,
}

/// An item within an `extern` block
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ForeignItemKind {
  /// A foreign function
  Fn(P<FnDecl>),
  /// A foreign static item (`static ext: u8`), with optional mutability
  /// (the boolean is true when mutable)
  Static(P<Ty>, bool),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Unsafety {
  Unsafe,
  Normal,
}

impl fmt::Display for Unsafety {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Display::fmt(match *self {
      Unsafety::Normal => "normal",
      Unsafety::Unsafe => "unsafe",
    }, f)
  }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Constness {
  Const,
  NotConst,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Ty {
  pub id: NodeId,
  pub node: TyKind,
  pub span: Span,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
/// The different kinds of types recognized by the compiler
pub enum TyKind {
  Vec(P<Ty>),
  /// A path (`module::module::...::Type`), optionally
  /// "qualified", e.g. `<Vec<T> as SomeTrait>::SomeType`.
  ///
  /// Type parameters are stored in the Path itself
  Path(Path),

  /// A tuple (`(A, B, C, D,...)`)
  Tup(Vec<P<Ty>> ),

  /// No-op; kept solely so that we can pretty-print faithfully
  Paren(P<Ty>),

  /// TyKind::Infer means the type should be inferred instead of it having been
  /// specified. This can appear anywhere in a type.
  Infer
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum BlockCheckMode {
  Default,
  Unsafe(UnsafeSource),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum UnsafeSource {
  CompilerGenerated,
  UserProvided,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Block {
  /// Statements in a block
  pub stmts: Vec<Stmt>,
  /// An expression at the end of the block
  /// without a semicolon, if any
  pub expr: Option<P<Expr>>,
  pub id: NodeId,
  /// Distinguishes between `unsafe { ... }` and `{ ... }`
  pub rules: BlockCheckMode,
  pub span: Span,
}

/// A statement
pub type Stmt = Spanned<StmtKind>;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum StmtKind {
  /// Could be an item or a local (let) binding:
  Decl(P<Decl>, NodeId),

  /// Expr without trailing semi-colon (must have unit type):
  Expr(P<Expr>, NodeId),

  /// Expr with trailing semi-colon (may have any type):
  Semi(P<Expr>, NodeId),
}

impl StmtKind {
  pub fn id(&self) -> Option<NodeId> {
    match *self {
      StmtKind::Decl(_, id) => Some(id),
      StmtKind::Expr(_, id) => Some(id),
      StmtKind::Semi(_, id) => Some(id),
    }
  }

  pub fn attrs(&self) -> &[Attribute] {
    match *self {
      StmtKind::Decl(ref d, _) => d.attrs(),
      StmtKind::Expr(ref e, _) |
      StmtKind::Semi(ref e, _) => e.attrs(),
    }
  }
}

pub type Decl = Spanned<DeclKind>;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DeclKind {
  /// A local (let) binding:
  Local(P<Local>),
  /// An item binding:
  Item(P<Item>),
}

impl Decl {
  pub fn attrs(&self) -> &[Attribute] {
    match self.node {
      DeclKind::Local(ref l) => l.attrs(),
      DeclKind::Item(ref i) => i.attrs(),
    }
  }
}

/// Local represents a `let` or 'var' statement, e.g., `let <pat>:<ty> = <expr>;`,
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Local {
  pub pat: P<Pat>,
  pub ty: Option<P<Ty>>,
  pub mutbl: Mutability,
  /// Initializer expression to set the value, if any
  pub init: Option<P<Expr>>,
  pub id: NodeId,
  pub span: Span,
  pub attrs: ThinAttributes,
}

impl Local {
  pub fn attrs(&self) -> &[Attribute] {
    match self.attrs {
      Some(ref b) => b,
      None => &[],
    }
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Pat {
  pub id: NodeId,
  pub node: PatKind,
  pub span: Span,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PatKind {
  /// Represents a wildcard pattern (`_`)
  Wild,

  /// A `PatKind::Ident` may either be a new bound variable,
  /// or a unit struct/variant pattern, or a const pattern (in the last two cases
  /// the third field must be `None`).
  ///
  /// In the unit or const pattern case, the parser can't determine
  /// which it is. The resolver determines this, and
  /// records this pattern's `NodeId` in an auxiliary
  /// set (of "PatIdents that refer to unit patterns or constants").
  Ident(BindingMode, SpannedIdent, Option<P<Pat>>),

  /// A path pattern.
  /// Such pattern can be resolved to a unit struct/variant or a constant.
  Path(Path),

  /// An associated const named using the qualified path `<T>::CONST` or
  /// `<T as Trait>::CONST`. Associated consts from inherent impls can be
  /// referred to as simply `T::CONST`, in which case they will end up as
  /// PatKind::Path, and the resolver will have to sort that out.
  QPath(QSelf, Path),

  /// A struct or struct variant pattern, e.g. `Variant {x, y, ..}`.
  /// The `bool` is `true` in the presence of a `..`.
  Struct(Path, Vec<Spanned<FieldPat>>, bool),

  /// A tuple pattern `(a, b)`.
  /// If the `..` pattern fragment is present, then `Option<usize>` denotes its position.
  /// 0 <= position <= subpats.len()
  Tuple(Vec<P<Pat>>, Option<usize>),

  /// A tuple struct/variant pattern `Variant(x, y, .., z)`.
  /// If the `..` pattern fragment is present, then `Option<usize>` denotes its position.
  /// 0 <= position <= subpats.len()
  TupleStruct(Path, Vec<P<Pat>>, Option<usize>),

  /// `[a, b, ..i, y, z]` is represented as:
  ///     `PatKind::Vec(box [a, b], Some(i), box [y, z])`
  Vec(Vec<P<Pat>>, Option<P<Pat>>, Vec<P<Pat>>),

  /// A range pattern, e.g. `1...2`
  Range(P<Expr>, P<Expr>),

  /// A literal
  Lit(P<Expr>),
}

/// A single field in a struct pattern
///
/// Patterns like the fields of Foo `{ x, ref y, ref mut z }`
/// are treated the same as` x: x, y: ref y, z: ref mut z`,
/// except is_shorthand is true
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FieldPat {
  /// The identifier for the field
  pub ident: Ident,
  /// The pattern the field is destructured to
  pub pat: P<Pat>,
  pub is_shorthand: bool,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum Mutability {
  Mutable,
  Immutable,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum BindingMode {
  ByRef,
  ByValue,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Expr {
  pub id: NodeId,
  pub node: ExprKind,
  pub span: Span,
  pub attrs: ThinAttributes
}

impl Expr {
  pub fn attrs(&self) -> &[Attribute] {
    match self.attrs {
      Some(ref b) => b,
      None => &[],
    }
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Field {
  pub ident: SpannedIdent,
  pub expr: P<Expr>,
  pub span: Span,
}

pub type SpannedIdent = Spanned<Ident>;

/// Limit types of a range (inclusive or exclusive)
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum RangeLimits {
  /// Inclusive at the beginning, exclusive at the end
  HalfOpen,
  /// Inclusive at the beginning and end
  Closed,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ExprKind {
  /// A unary operation (For example: `!x`, `*x`)
  Unary(UnOp, P<Expr>),
  Binary(BinOp, P<Expr>, P<Expr>),
  /// A cast (`foo as f64`)
  Cast(P<Expr>, P<Ty>),
  Type(P<Expr>, P<Ty>),

  /// An `if` block, with an optional else block
  ///
  /// `if expr { block } else { expr }`
  If(P<Expr>, P<Block>, Option<P<Expr>>),

  /// An `if let` expression with an optional else block
  ///
  /// `if let pat = expr { block } else { expr }`
  ///
  /// This is desugared to a `match` expression.
  IfLet(P<Pat>, P<Expr>, P<Block>, Option<P<Expr>>),

  Closure,
  While,
  Loop,
  ForLoop,
  Match,
  Block,
  /// First expr is the place; second expr is the value.
  InPlace(P<Expr>, P<Expr>),
  /// An assignment (`a = foo()`)
  Assign(P<Expr>, P<Expr>),
  /// An assignment with an operator
  ///
  /// For example, `a += 1`.
  AssignOp(BinOp, P<Expr>, P<Expr>),

  Paren(P<Expr>),

  /// Variable reference, possibly containing `::` and/or type
  /// parameters, e.g. foo::bar::<baz>.
  ///
  /// Optionally "qualified",
  /// e.g. `<Vec<T> as SomeTrait>::SomeType`.
  Path(Option<QSelf>, Path),

  /// A struct literal expression.
  ///
  /// For example, `Foo {x: 1, y: 2}`, or
  /// `Foo {x: 1, .. base}`, where `base` is the `Option<Expr>`.
  Struct(Path, Vec<Field>, Option<P<Expr>>),

  /// A tuple (`(a, b, c ,d)`)
  Tup(Vec<P<Expr>>),

  /// Access of a named struct field (`obj.foo`)
  Field(P<Expr>, SpannedIdent),

  /// Access of an unnamed field of a struct or tuple-struct
  ///
  /// For example, `foo.0`.
  TupField(P<Expr>, Spanned<usize>),

  /// An indexing operation (`foo[2]`)
  Index(P<Expr>, P<Expr>),

  /// A range (`1..2`, `1..`, `..2`, `1...2`, `1...`, `...2`)
  Range(Option<P<Expr>>, Option<P<Expr>>, RangeLimits),

  /// A function call
  ///
  /// The first field resolves to the function itself,
  /// and the second field is the list of arguments
  Call(P<Expr>, Vec<P<Expr>>),

  /// A method call (`x.foo::<Bar, Baz>(a, b, c, d)`)
  ///
  /// The `SpannedIdent` is the identifier for the method name.
  /// The vector of `Ty`s are the ascripted type parameters for the method
  /// (within the angle brackets).
  ///
  /// The first element of the vector of `Expr`s is the expression that evaluates
  /// to the object on which the method is being called on (the receiver),
  /// and the remaining elements are the rest of the arguments.
  ///
  /// Thus, `x.foo::<Bar, Baz>(a, b, c, d)` is represented as
  /// `ExprKind::MethodCall(foo, [Bar, Baz], [x, a, b, c, d])`.
  MethodCall(SpannedIdent, Vec<P<Ty>>, Vec<P<Expr>>),

  /// A literal (For example: `1`, `"foo"`)
  Lit(P<Lit>),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum UnOp {
  /// The `!` operator for logical inversion
  Not,
  /// The `-` operator for negation
  Neg,
}

impl UnOp {
  pub fn to_string(op: UnOp) -> &'static str {
    match op {
      UnOp::Not => "!",
      UnOp::Neg => "-",
    }
  }
}

pub type BinOp = Spanned<BinOpKind>;

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum BinOpKind {
  /// The `+` operator (addition)
  Add,
  /// The `-` operator (subtraction)
  Sub,
  /// The `*` operator (multiplication)
  Mul,
  /// The `/` operator (division)
  Div,
  /// The `%` operator (modulus)
  Rem,
  /// The `&&` operator (logical and)
  And,
  /// The `||` operator (logical or)
  Or,
  /// The `^` operator (bitwise xor)
  BitXor,
  /// The `&` operator (bitwise and)
  BitAnd,
  /// The `|` operator (bitwise or)
  BitOr,
  /// The `<<` operator (shift left)
  LShift,
  /// The `>>` operator (shift right)
  RShift,
  /// The `==` operator (equality)
  Eq,
  /// The `<` operator (less than)
  Lt,
  /// The `<=` operator (less than or equal to)
  Le,
  /// The `!=` operator (not equal to)
  Ne,
  /// The `>=` operator (greater than or equal to)
  Ge,
  /// The `>` operator (greater than)
  Gt,
}

impl BinOpKind {
  pub fn to_string(&self) -> &'static str {
    use self::BinOpKind::*;
    match *self {
      Add => "+",
      Sub => "-",
      Mul => "*",
      Div => "/",
      Rem => "%",
      And => "&&",
      Or => "||",
      BitXor => "^",
      BitAnd => "&",
      BitOr => "|",
      LShift => "<<",
      RShift => ">>",
      Eq => "==",
      Lt => "<",
      Le => "<=",
      Ne => "!=",
      Ge => ">=",
      Gt => ">",
    }
  }
  pub fn lazy(&self) -> bool {
    match *self {
      BinOpKind::And | BinOpKind::Or => true,
      _ => false
    }
  }

  pub fn is_shift(&self) -> bool {
    match *self {
      BinOpKind::LShift | BinOpKind::RShift => true,
      _ => false
    }
  }
  pub fn is_comparison(&self) -> bool {
    use self::BinOpKind::*;
    match *self {
      Eq | Lt | Le | Ne | Gt | Ge =>
        true,
      And | Or | Add | Sub | Mul | Div | Rem |
      BitXor | BitAnd | BitOr | LShift | RShift =>
        false,
    }
  }
  /// Returns `true` if the binary operator takes its arguments by value
  pub fn is_by_value(&self) -> bool {
    !self.is_comparison()
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Copy)]
pub enum IntTy {
  Is,
  I8,
  I16,
  I32,
  I64,
}

impl fmt::Debug for IntTy {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Display::fmt(self, f)
  }
}

impl fmt::Display for IntTy {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.ty_to_string())
  }
}

impl IntTy {
  pub fn ty_to_string(&self) -> &'static str {
    match *self {
      IntTy::Is => "isize",
      IntTy::I8 => "i8",
      IntTy::I16 => "i16",
      IntTy::I32 => "i32",
      IntTy::I64 => "i64"
    }
  }

  pub fn val_to_string(&self, val: i64) -> String {
    // cast to a u64 so we can correctly print INT64_MIN. All integral types
    // are parsed as u64, so we wouldn't want to print an extra negative
    // sign.
    format!("{}{}", val as u64, self.ty_to_string())
  }

  pub fn ty_max(&self) -> u64 {
    match *self {
      IntTy::I8 => 0x80,
      IntTy::I16 => 0x8000,
      IntTy::Is | IntTy::I32 => 0x80000000, // FIXME: actually ni about Is
      IntTy::I64 => 0x8000000000000000
    }
  }

  pub fn bit_width(&self) -> Option<usize> {
    Some(match *self {
      IntTy::Is => return None,
      IntTy::I8 => 8,
      IntTy::I16 => 16,
      IntTy::I32 => 32,
      IntTy::I64 => 64,
    })
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Copy)]
pub enum UintTy {
  Us,
  U8,
  U16,
  U32,
  U64,
}

impl UintTy {
  pub fn ty_to_string(&self) -> &'static str {
    match *self {
      UintTy::Us => "usize",
      UintTy::U8 => "u8",
      UintTy::U16 => "u16",
      UintTy::U32 => "u32",
      UintTy::U64 => "u64"
    }
  }

  pub fn val_to_string(&self, val: u64) -> String {
    format!("{}{}", val, self.ty_to_string())
  }

  pub fn ty_max(&self) -> u64 {
    match *self {
      UintTy::U8 => 0xff,
      UintTy::U16 => 0xffff,
      UintTy::Us | UintTy::U32 => 0xffffffff, // FIXME: actually ni about Us
      UintTy::U64 => 0xffffffffffffffff
    }
  }

  pub fn bit_width(&self) -> Option<usize> {
    Some(match *self {
      UintTy::Us => return None,
      UintTy::U8 => 8,
      UintTy::U16 => 16,
      UintTy::U32 => 32,
      UintTy::U64 => 64,
    })
  }
}

impl fmt::Debug for UintTy {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Display::fmt(self, f)
  }
}

impl fmt::Display for UintTy {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.ty_to_string())
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Copy)]
pub enum FloatTy {
  F32,
  F64,
}

impl fmt::Debug for FloatTy {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Display::fmt(self, f)
  }
}

impl fmt::Display for FloatTy {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.ty_to_string())
  }
}

impl FloatTy {
  pub fn ty_to_string(&self) -> &'static str {
    match *self {
      FloatTy::F32 => "f32",
      FloatTy::F64 => "f64",
    }
  }

  pub fn bit_width(&self) -> usize {
    match *self {
      FloatTy::F32 => 32,
      FloatTy::F64 => 64,
    }
  }
}

/// The explicit Self type in a "qualified path". The actual
/// path, including the trait and the associated item, is stored
/// separately. `position` represents the index of the associated
/// item qualified with this Self type.
///
/// ```ignore
/// <Vec<T> as a::b::Trait>::AssociatedItem
///  ^~~~~     ~~~~~~~~~~~~~~^
///  ty        position = 3
///
/// <Vec<T>>::AssociatedItem
///  ^~~~~    ^
///  ty       position = 0
/// ```
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct QSelf {
  pub ty: P<Ty>,
  pub position: usize
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum StrStyle {
  /// A regular string, like `"foo"`
  Cooked,
  /// A raw string, like `r##"foo"##`
  ///
  /// The uint is the number of `#` symbols used
  Raw(usize)
}

/// A literal
pub type Lit = Spanned<LitKind>;

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum LitIntType {
  Signed(IntTy),
  Unsigned(UintTy),
  Unsuffixed,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum LitKind {
  /// A string literal (`"foo"`)
  Str(InternedString, StrStyle),
  /// A byte string (`b"foo"`)
  ByteStr(Rc<Vec<u8>>),
  /// A byte char (`b'f'`)
  Byte(u8),
  /// A character literal (`'a'`)
  Char(char),
  /// An integer literal (`1`)
  Int(u64, LitIntType),
  /// A float literal (`1f64` or `1E10f64`)
  Float(InternedString, FloatTy),
  /// A float literal without a suffix (`1.0 or 1.0E10`)
  FloatUnsuffixed(InternedString),
  /// A boolean literal
  Bool(bool),
}

/// Meta-data associated with an item
pub type Attribute = Spanned<Attribute_>;

/// Distinguishes between Attributes that decorate items and Attributes that
/// are contained as statements within items. These two cases need to be
/// distinguished for pretty-printing.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum AttrStyle {
  Outer,
  Inner,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub struct AttrId(pub usize);

/// Doc-comments are promoted to attributes that have is_sugared_doc = true
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Attribute_ {
  pub id: AttrId,
  pub value: P<MetaItem>,
}

pub type MetaItem = Spanned<MetaItemKind>;

#[derive(Clone, Eq, Hash, Debug)]
pub enum MetaItemKind {
  Word(InternedString),
  List(InternedString, Vec<P<MetaItem>>),
  //NameValue(InternedString, Lit),
}

// can't be derived because the MetaItemKind::List requires an unordered comparison
impl PartialEq for MetaItemKind {
  fn eq(&self, other: &MetaItemKind) -> bool {
    use self::MetaItemKind::*;
    match *self {
      Word(ref ns) => match *other {
        Word(ref no) => (*ns) == (*no),
        _ => false
      },
      List(ref ns, ref miss) => match *other {
        List(ref no, ref miso) => {
          ns == no &&
          miss.iter().all(|mi| miso.iter().any(|x| x.node == mi.node))
        }
        _ => false
      }
    }
  }
}

/// Represents the header (not the body) of a function declaration
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FnDecl {
  pub inputs: Vec<Arg>,
  pub output: FunctionRetTy,
  pub variadic: bool
}

/// represents an argument in a function header
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Arg {
  pub ty: P<Ty>,
  pub pat: P<Pat>,
  pub id: NodeId,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FunctionRetTy {
  /// Functions with return type `!`that always
  /// raise an error or exit (i.e. never return to the caller)
  None(Span),
  /// Return type is not specified.
  ///
  /// Functions default to `()` and
  /// closures default to inference. Span points to where return
  /// type would be inserted.
  Default(Span),
  /// Everything else
  Ty(P<Ty>),
}

impl FunctionRetTy {
  pub fn span(&self) -> Span {
    match *self {
      FunctionRetTy::None(span) => span,
      FunctionRetTy::Default(span) => span,
      FunctionRetTy::Ty(ref ty) => ty.span,
    }
  }
}

/// A delimited sequence of token trees
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Delimited {
  /// The type of delimiter
  pub delim: token::DelimToken,
  /// The span covering the opening delimiter
  pub open_span: Span,
  /// The delimited sequence of token trees
  pub tts: Vec<TokenTree>,
  /// The span covering the closing delimiter
  pub close_span: Span,
}

impl Delimited {
  /// Returns the opening delimiter as a token.
  pub fn open_token(&self) -> token::Token {
    token::OpenDelim(self.delim)
  }

  /// Returns the closing delimiter as a token.
  pub fn close_token(&self) -> token::Token {
    token::CloseDelim(self.delim)
  }

  /// Returns the opening delimiter as a token tree.
  pub fn open_tt(&self) -> TokenTree {
    TokenTree::Token(self.open_span, self.open_token())
  }

  /// Returns the closing delimiter as a token tree.
  pub fn close_tt(&self) -> TokenTree {
    TokenTree::Token(self.close_span, self.close_token())
  }
}

/// When the main rust parser encounters a syntax-extension invocation, it
/// parses the arguments to the invocation as a token-tree. This is a very
/// loose structure, such that all sorts of different AST-fragments can
/// be passed to syntax extensions using a uniform type.
///
/// If the syntax extension is an MBE macro, it will attempt to match its
/// LHS token tree against the provided token tree, and if it finds a
/// match, will transcribe the RHS token tree, splicing in any captured
/// macro_parser::matched_nonterminals into the `SubstNt`s it finds.
///
/// The RHS of an MBE macro is the only place `SubstNt`s are substituted.
/// Nothing special happens to misnamed or misplaced `SubstNt`s.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TokenTree {
  /// A single token
  Token(Span, token::Token),
  /// A delimited sequence of token trees
  Delimited(Span, Rc<Delimited>),
}

impl TokenTree {
  pub fn len(&self) -> usize {
    match *self {
      TokenTree::Token(_, token::DocComment(name)) => {
        match doc_comment_style(&name.as_str()) {
          AttrStyle::Outer => 2,
          AttrStyle::Inner => 3
        }
      }
      TokenTree::Delimited(_, ref delimed) => {
        delimed.tts.len() + 2
      }
      TokenTree::Token(..) => 0
    }
  }

  pub fn get_tt(&self, index: usize) -> TokenTree {
    match (self, index) {
      (&TokenTree::Token(sp, token::DocComment(_)), 0) => {
        TokenTree::Token(sp, token::Pound)
      }
      (&TokenTree::Token(sp, token::DocComment(name)), 1)
      if doc_comment_style(&name.as_str()) == AttrStyle::Inner => {
        TokenTree::Token(sp, token::Not)
      }
      (&TokenTree::Token(sp, token::DocComment(name)), _) => {
        let stripped = strip_doc_comment_decoration(&name.as_str());

        // Searches for the occurrences of `"#*` and returns the minimum number of `#`s
        // required to wrap the text.
        let num_of_hashes = stripped.chars().scan(0, |cnt, x| {
          *cnt = if x == '"' {
            1
          } else if *cnt != 0 && x == '#' {
            *cnt + 1
          } else {
            0
          };
          Some(*cnt)
        }).max().unwrap_or(0);

        TokenTree::Delimited(sp, Rc::new(Delimited {
          delim: token::Bracket,
          open_span: sp,
          tts: vec![TokenTree::Token(sp, token::Ident(token::str_to_ident("doc"))),
                              TokenTree::Token(sp, token::Eq),
                              TokenTree::Token(sp, token::Literal(
                                  token::StrRaw(token::intern(&stripped), num_of_hashes), None))],
          close_span: sp,
        }))
      }
      (&TokenTree::Delimited(_, ref delimed), _) => {
        if index == 0 {
          return delimed.open_tt();
        }
        if index == delimed.tts.len() + 1 {
          return delimed.close_tt();
        }
        delimed.tts[index - 1].clone()
      }
      _ => panic!("Cannot expand a token tree")
    }
  }

  /// Returns the `Span` corresponding to this token tree.
  pub fn get_span(&self) -> Span {
    match *self {
      TokenTree::Token(span, _)     => span,
      TokenTree::Delimited(span, _) => span,
    }
  }
}