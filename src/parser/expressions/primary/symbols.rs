// Symbol literal parsing: `:name`, `:@ivar`, `:@@cvar`, `:keyword`, `:[]`, `:+`, `:"..."`.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::{InterpolationPart, Lexer, Position, TokenKind};
use crate::parser::Parser;

/// Whether `kind` can follow a `:` to form a symbol literal. Keeps the
/// paren-less argument checks in step with what `parse_symbol_literal`
/// actually accepts, so `foo :alias, :meth` parses like `foo :a, :b`.
pub(crate) fn starts_symbol_literal(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Ident(_)
            | TokenKind::InstanceVar(_)
            | TokenKind::ClassVar(_)
            | TokenKind::GlobalVar(_)
            | TokenKind::String(_)
            | TokenKind::ByteString(_)
            | TokenKind::BinaryString(_)
            | TokenKind::FrozenByteString(_)
            | TokenKind::FrozenBinaryString(_)
            | TokenKind::FrozenString(_)
            | TokenKind::MutableString(_)
            | TokenKind::InterpolatedString(_)
            | TokenKind::Def
            | TokenKind::Class
            | TokenKind::If
            | TokenKind::Else
            | TokenKind::End
            | TokenKind::Do
            | TokenKind::Nil
            | TokenKind::True
            | TokenKind::False
            | TokenKind::Return
            | TokenKind::Retry
            | TokenKind::Redo
            | TokenKind::Begin
            | TokenKind::Rescue
            | TokenKind::Ensure
            | TokenKind::While
            | TokenKind::For
            | TokenKind::Case
            | TokenKind::When
            | TokenKind::Module
            | TokenKind::Include
            | TokenKind::Yield
            | TokenKind::Super
            | TokenKind::Lambda
            | TokenKind::Break
            | TokenKind::Continue
            | TokenKind::Raise
            | TokenKind::AttrReader
            | TokenKind::AttrWriter
            | TokenKind::AttrAccessor
            | TokenKind::Extend
            | TokenKind::Alias
            | TokenKind::Unless
            | TokenKind::Until
            | TokenKind::Then
            | TokenKind::Elsif
            | TokenKind::In
            | TokenKind::KeywordAnd
            | TokenKind::KeywordOr
            | TokenKind::NotKeyword
            | TokenKind::Defined
            | TokenKind::MagicFile
            | TokenKind::MagicLine
            | TokenKind::MagicDir
            | TokenKind::SourceEncoding(_)
    )
}

/// Whether a `:` glued to this token names an operator method, as `:/` and
/// `:<=>` do. Kept apart from the names above because only an adjacent colon
/// makes one: `condition ? value : -1` is a ternary, not the symbol `:-`.
pub(crate) fn starts_operator_symbol(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::StarStar
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::EqualEqual
            | TokenKind::TripleEqual
            | TokenKind::BangEqual
            | TokenKind::Less
            | TokenKind::Greater
            | TokenKind::LessEqual
            | TokenKind::GreaterEqual
            | TokenKind::Spaceship
            | TokenKind::Shovel
            | TokenKind::RightShift
            | TokenKind::Tilde
            | TokenKind::Caret
            | TokenKind::Match
            | TokenKind::NotMatch
            | TokenKind::Bang
            | TokenKind::Ampersand
            | TokenKind::Pipe
            | TokenKind::LBracket
    )
}

impl Parser<'_> {
    /// The symbol `name` stands for, as `:name=` where an `=` follows. It
    /// names a setter only when the `=` is written against the name, so
    /// `:a ==> 1` is `:a` followed by `==`.
    fn symbol_with_setter(&mut self, name: &str, symbol_position: Position) -> Expression {
        if self.check(&[TokenKind::Equal])
            && !self.peek().had_leading_space
            && !matches!(self.peek_ahead(1).kind, TokenKind::Equal)
        {
            self.advance();
            symbol(format!("{}=", name), symbol_position)
        } else {
            symbol(name, symbol_position)
        }
    }

    /// Parse a symbol literal after a leading `:` token has been consumed.
    pub(crate) fn parse_symbol_literal(
        &mut self,
        symbol_position: Position,
    ) -> Result<Expression, MetorexError> {
        let next = self.advance();
        match next.kind {
            TokenKind::Ident(name) => Ok(self.symbol_with_setter(&name, symbol_position)),
            TokenKind::InstanceVar(name) => Ok(symbol(format!("@{}", name), symbol_position)),
            TokenKind::ClassVar(name) => Ok(symbol(format!("@@{}", name), symbol_position)),
            // `:$name` is a symbol too, which is how `trace_var` names the
            // global it watches.
            TokenKind::GlobalVar(name) => Ok(symbol(format!("${}", name), symbol_position)),

            // Keyword names as symbols, `:next=` naming a setter as `:name=`
            // does.
            TokenKind::Def => Ok(self.symbol_with_setter("def", symbol_position)),
            TokenKind::Class => Ok(self.symbol_with_setter("class", symbol_position)),
            TokenKind::If => Ok(self.symbol_with_setter("if", symbol_position)),
            TokenKind::Else => Ok(self.symbol_with_setter("else", symbol_position)),
            TokenKind::End => Ok(self.symbol_with_setter("end", symbol_position)),
            TokenKind::Do => Ok(self.symbol_with_setter("do", symbol_position)),
            TokenKind::Nil => Ok(self.symbol_with_setter("nil", symbol_position)),
            TokenKind::True => Ok(self.symbol_with_setter("true", symbol_position)),
            TokenKind::False => Ok(self.symbol_with_setter("false", symbol_position)),
            TokenKind::Return => Ok(self.symbol_with_setter("return", symbol_position)),
            TokenKind::Retry => Ok(self.symbol_with_setter("retry", symbol_position)),
            TokenKind::Redo => Ok(self.symbol_with_setter("redo", symbol_position)),
            TokenKind::Begin => Ok(self.symbol_with_setter("begin", symbol_position)),
            TokenKind::Rescue => Ok(self.symbol_with_setter("rescue", symbol_position)),
            TokenKind::Ensure => Ok(self.symbol_with_setter("ensure", symbol_position)),
            TokenKind::While => Ok(self.symbol_with_setter("while", symbol_position)),
            TokenKind::For => Ok(self.symbol_with_setter("for", symbol_position)),
            TokenKind::Case => Ok(self.symbol_with_setter("case", symbol_position)),
            TokenKind::When => Ok(self.symbol_with_setter("when", symbol_position)),
            TokenKind::Module => Ok(self.symbol_with_setter("module", symbol_position)),
            TokenKind::Include => Ok(self.symbol_with_setter("include", symbol_position)),
            TokenKind::Yield => Ok(self.symbol_with_setter("yield", symbol_position)),
            TokenKind::Super => Ok(self.symbol_with_setter("super", symbol_position)),
            TokenKind::Lambda => Ok(self.symbol_with_setter("lambda", symbol_position)),
            TokenKind::Break => Ok(self.symbol_with_setter("break", symbol_position)),
            TokenKind::Continue => Ok(self.symbol_with_setter("next", symbol_position)),
            TokenKind::Raise => Ok(self.symbol_with_setter("raise", symbol_position)),
            TokenKind::AttrReader => Ok(self.symbol_with_setter("attr_reader", symbol_position)),
            TokenKind::AttrWriter => Ok(self.symbol_with_setter("attr_writer", symbol_position)),
            TokenKind::AttrAccessor => {
                Ok(self.symbol_with_setter("attr_accessor", symbol_position))
            }
            TokenKind::Extend => Ok(self.symbol_with_setter("extend", symbol_position)),
            TokenKind::Alias => Ok(self.symbol_with_setter("alias", symbol_position)),
            TokenKind::Unless => Ok(self.symbol_with_setter("unless", symbol_position)),
            TokenKind::Until => Ok(self.symbol_with_setter("until", symbol_position)),
            TokenKind::Then => Ok(self.symbol_with_setter("then", symbol_position)),
            TokenKind::Elsif => Ok(self.symbol_with_setter("elsif", symbol_position)),
            TokenKind::In => Ok(self.symbol_with_setter("in", symbol_position)),
            TokenKind::KeywordAnd => Ok(self.symbol_with_setter("and", symbol_position)),
            TokenKind::KeywordOr => Ok(self.symbol_with_setter("or", symbol_position)),
            TokenKind::NotKeyword => Ok(self.symbol_with_setter("not", symbol_position)),
            TokenKind::Defined => Ok(symbol("defined?", symbol_position)),
            TokenKind::MagicFile => Ok(symbol("__FILE__", symbol_position)),
            TokenKind::MagicLine => Ok(symbol("__LINE__", symbol_position)),
            TokenKind::MagicDir => Ok(symbol("__dir__", symbol_position)),
            TokenKind::SourceEncoding(_) => Ok(symbol("__ENCODING__", symbol_position)),

            // :[] and :[]= operator symbols
            TokenKind::LBracket => {
                self.expect(TokenKind::RBracket, "Expected ']' after '[' in symbol")?;
                if self.match_token(&[TokenKind::Equal]) {
                    Ok(symbol("[]=", symbol_position))
                } else {
                    Ok(symbol("[]", symbol_position))
                }
            }

            // Operator symbols. `-@` and `+@` name the unary forms, spelled
            // with the `@` the lexer reads as an instance variable with no
            // name of its own.
            TokenKind::Plus | TokenKind::Minus => {
                let operator = match next.kind {
                    TokenKind::Plus => "+",
                    _ => "-",
                };
                if matches!(&self.peek().kind, TokenKind::InstanceVar(name) if name.is_empty()) {
                    self.advance();
                    return Ok(symbol(format!("{}@", operator), symbol_position));
                }
                Ok(symbol(operator, symbol_position))
            }
            TokenKind::StarStar => Ok(symbol("**", symbol_position)),
            TokenKind::Bang => Ok(symbol("!", symbol_position)),
            TokenKind::Ampersand => Ok(symbol("&", symbol_position)),
            TokenKind::Pipe => Ok(symbol("|", symbol_position)),
            TokenKind::Star => Ok(symbol("*", symbol_position)),
            TokenKind::Slash => Ok(symbol("/", symbol_position)),
            TokenKind::Percent => Ok(symbol("%", symbol_position)),
            TokenKind::EqualEqual => Ok(symbol("==", symbol_position)),
            TokenKind::TripleEqual => Ok(symbol("===", symbol_position)),
            TokenKind::BangEqual => Ok(symbol("!=", symbol_position)),
            TokenKind::Less => Ok(symbol("<", symbol_position)),
            TokenKind::Greater => Ok(symbol(">", symbol_position)),
            TokenKind::LessEqual => Ok(symbol("<=", symbol_position)),
            TokenKind::GreaterEqual => Ok(symbol(">=", symbol_position)),
            TokenKind::Spaceship => Ok(symbol("<=>", symbol_position)),
            TokenKind::Shovel => Ok(symbol("<<", symbol_position)),
            TokenKind::RightShift => Ok(symbol(">>", symbol_position)),
            TokenKind::Tilde => Ok(symbol("~", symbol_position)),
            TokenKind::Caret => Ok(symbol("^", symbol_position)),
            TokenKind::Match => Ok(symbol("=~", symbol_position)),
            TokenKind::NotMatch => Ok(symbol("!~", symbol_position)),

            // :"string" syntax — symbol from string literal
            TokenKind::String(s) | TokenKind::FrozenString(s) | TokenKind::MutableString(s) => {
                Ok(symbol(s, symbol_position))
            }

            // A symbol written in escapes names the characters those bytes
            // spell. Bytes that spell nothing are refused while the source is
            // read, the way Ruby refuses them.
            TokenKind::ByteString(s)
            | TokenKind::BinaryString(s)
            | TokenKind::FrozenByteString(s)
            | TokenKind::FrozenBinaryString(s) => {
                let bytes: Vec<u8> = s.chars().map(|held| held as u32 as u8).collect();
                match String::from_utf8(bytes) {
                    Ok(spelled) => Ok(symbol(spelled, symbol_position)),
                    Err(_) => Err(self.error_at_previous("invalid symbol")),
                }
            }

            // `:"#{...}"` names a symbol built at run time, so the assembled
            // characters go through `to_sym`.
            TokenKind::InterpolatedString(parts) => {
                let mut ast_parts = Vec::new();
                for part in parts {
                    match part {
                        InterpolationPart::Text(text) => {
                            ast_parts.push(crate::ast::node::InterpolationPart::Text(text));
                        }
                        InterpolationPart::Expression(expr_str, _) => {
                            let expr_lexer = Lexer::new(&expr_str);
                            let expr_tokens = expr_lexer.tokenize();
                            let mut expr_parser = self.nested_parser(expr_tokens);
                            let expr = expr_parser.parse_expression()?;
                            ast_parts.push(crate::ast::node::InterpolationPart::Expression(
                                Box::new(expr),
                            ));
                        }
                    }
                }
                Ok(Expression::MethodCall {
                    receiver: Box::new(Expression::InterpolatedString {
                        parts: ast_parts,
                        position: symbol_position,
                    }),
                    method: "to_sym".to_string(),
                    arguments: Vec::new(),
                    trailing_block: None,
                    position: symbol_position,
                })
            }

            _ => Err(self.error_at_previous("Expected identifier after ':' for symbol")),
        }
    }
}

fn symbol(value: impl Into<String>, position: Position) -> Expression {
    // Reading a symbol in the source interns it, so it is among the names
    // `Symbol.all_symbols` reports even before the line runs.
    let value: String = value.into();
    crate::symbol_registry::record(&value);
    Expression::Symbol { value, position }
}
