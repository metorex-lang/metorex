//! The spellings a parameter list records for what is not a plain name.

/// Placeholder parameter recorded for a block written `{ |a,| }`. The
/// trailing comma is what tells Ruby to destructure a single array argument,
/// so it has to survive parsing; it never binds a name.
pub const TRAILING_COMMA_PARAM: &str = ",";

/// Marks a `|**nil|` declaration, which says the block takes no keyword
/// arguments at all. It names nothing and binds nothing.
pub const NO_KEYWORDS_PARAM: &str = "**nil";

/// Marks a `|(a, b)|` group, whose names follow separated by commas. One
/// array argument spreads across them.
pub const DESTRUCTURED_GROUP_PREFIX: &str = "(";

/// Marks a name written after the `;` in a block's parameter list. It is a
/// local of the block rather than a parameter, so it starts as nil and takes
/// no argument.
pub const BLOCK_LOCAL_PREFIX: &str = ";";

/// Marks a `|name:|` keyword parameter, which takes its value from the
/// keyword arguments rather than by position.
pub const KEYWORD_PARAM_PREFIX: &str = ":";

/// The parameter a block takes because its body reads a bare `it`. It is
/// spelled apart from an ordinary `it` so the block can tell the implicit
/// parameter from a name of its own: the implicit one is not a local the
/// block declared, and it gives way to an `it` already in scope.
pub const IMPLICIT_IT_PARAM: &str = "~it";
