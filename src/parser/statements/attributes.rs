// Attribute accessor parsing (attr_reader, attr_writer, attr_accessor)

use crate::ast::{Expression, Statement};
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// Parse attr_reader statement: attr_reader :name1, :name2, ...
    pub(crate) fn parse_attr_reader(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self
            .expect(TokenKind::AttrReader, "Expected 'attr_reader'")?
            .position;
        self.skip_whitespace();
        if self.attribute_call_is_parenthesized() || self.attribute_call_names_keywords() {
            return self.attribute_method_call("attr_reader", start_pos);
        }

        let attributes = self.parse_attribute_list()?;

        Ok(Statement::AttrReader {
            attributes,
            position: start_pos,
        })
    }

    /// Parse attr_writer statement: attr_writer :name1, :name2, ...
    pub(crate) fn parse_attr_writer(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self
            .expect(TokenKind::AttrWriter, "Expected 'attr_writer'")?
            .position;
        self.skip_whitespace();
        if self.attribute_call_is_parenthesized() || self.attribute_call_names_keywords() {
            return self.attribute_method_call("attr_writer", start_pos);
        }

        let attributes = self.parse_attribute_list()?;

        Ok(Statement::AttrWriter {
            attributes,
            position: start_pos,
        })
    }

    /// Parse attr_accessor statement: attr_accessor :name1, :name2, ...
    pub(crate) fn parse_attr_accessor(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self
            .expect(TokenKind::AttrAccessor, "Expected 'attr_accessor'")?
            .position;
        self.skip_whitespace();
        if self.attribute_call_is_parenthesized() || self.attribute_call_names_keywords() {
            return self.attribute_method_call("attr_accessor", start_pos);
        }

        let attributes = self.parse_attribute_list()?;

        Ok(Statement::AttrAccessor {
            attributes,
            position: start_pos,
        })
    }

    /// Whether the names on this line come with keyword arguments, as
    /// `attr_accessor :debug, type: :boolean` passes to a module that writes
    /// its own `attr_accessor`.
    fn attribute_call_names_keywords(&self) -> bool {
        let mut offset = 0;
        let mut depth = 0usize;
        loop {
            let token = self.peek_ahead(offset);
            match &token.kind {
                TokenKind::Newline | TokenKind::Semicolon | TokenKind::EOF => return false,
                TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => depth += 1,
                TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                }
                TokenKind::StarStar if depth == 0 => return true,
                TokenKind::Ident(_) if depth == 0 => {
                    let after = self.peek_ahead(offset + 1);
                    if matches!(after.kind, TokenKind::Colon) && !after.had_leading_space {
                        return true;
                    }
                }
                _ => {}
            }
            offset += 1;
        }
    }

    /// Whether the names are written in parentheses against the method's
    /// name, `attr_reader(:a, :b)`, which reads as any call does.
    fn attribute_call_is_parenthesized(&self) -> bool {
        self.check(&[TokenKind::LParen]) && !self.peek().had_leading_space
    }

    /// The `attr_*` call written as the method call it is, for a module that
    /// writes its own and takes keywords.
    fn attribute_method_call(
        &mut self,
        name: &str,
        position: crate::lexer::Position,
    ) -> Result<Statement, MetorexError> {
        let arguments = if self.match_token(&[TokenKind::LParen]) {
            self.parse_arguments()?
        } else {
            self.parse_arguments_without_parens()?
        };
        self.wrap_with_modifier(Statement::Expression {
            expression: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: name.to_string(),
                    position,
                }),
                arguments,
                trailing_block: None,
                position,
            },
            position,
        })
    }

    /// Parse a comma-separated list of attribute expressions. Each expression
    /// is evaluated at runtime; symbols and strings resolve directly, other
    /// values are coerced via #to_str.
    pub(crate) fn parse_attribute_list(&mut self) -> Result<Vec<Expression>, MetorexError> {
        let mut attributes = Vec::new();

        attributes.push(self.parse_attribute_expression()?);

        loop {
            self.skip_whitespace();

            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }

            self.skip_whitespace();
            attributes.push(self.parse_attribute_expression()?);
        }

        Ok(attributes)
    }

    /// Parse a single attribute name expression. Recognizes `:keyword` for
    /// reserved-word symbols (e.g. `:include`, `:class`); otherwise defers to
    /// the standard expression parser so strings, identifiers, and arbitrary
    /// expressions all work.
    pub(crate) fn parse_attribute_expression(&mut self) -> Result<Expression, MetorexError> {
        if matches!(self.peek().kind, TokenKind::Colon) {
            let next = self.peek_ahead(1).kind.clone();
            if let Some(name) = reserved_keyword_name(&next) {
                let colon_pos = self.advance().position;
                self.advance();
                return Ok(Expression::Symbol {
                    value: name,
                    position: colon_pos,
                });
            }
        }
        // A `rescue` after the names is a modifier on the whole statement.
        self.paren_less_arg_depth += 1;
        let parsed = self.parse_expression();
        self.paren_less_arg_depth -= 1;
        parsed
    }
}

pub(crate) fn reserved_keyword_name(kind: &TokenKind) -> Option<String> {
    Some(
        match kind {
            TokenKind::Include => "include",
            TokenKind::Extend => "extend",
            TokenKind::Class => "class",
            TokenKind::Module => "module",
            TokenKind::Def => "def",
            TokenKind::End => "end",
            TokenKind::If => "if",
            TokenKind::Else => "else",
            TokenKind::Do => "do",
            _ => return None,
        }
        .to_string(),
    )
}
