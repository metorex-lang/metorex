// The source a failure message quotes: the lines of the statement a
// backtrace line falls in, and the text of the value handed to `expect`.
// Both read the file's tokens, which the lexer produces far faster than a
// parse tree written in Ruby could be built.

use crate::error::MetorexError;
use crate::lexer::{Lexer, Token, TokenKind};
use crate::object::Object;
use crate::object::{OptionView, View};
use crate::vm::VirtualMachine;

/// One token with how deeply nested it sits: the depth before it and after
/// it, counting brackets and the keywords an `end` closes.
struct Placed<'a> {
    token: &'a Token,
    depth_before: usize,
    depth_after: usize,
}

/// What a token on the nesting stack opened.
#[derive(Clone, Copy, PartialEq)]
enum Opening {
    Bracket,
    Keyword,
}

/// Whether a statement can start right after `previous`, which is what makes
/// `if`, `unless`, `while`, and `until` open a block rather than modify the
/// statement before them.
fn begins_statement(previous: Option<&TokenKind>) -> bool {
    use TokenKind::*;
    matches!(
        previous,
        None | Some(
            Newline
                | Semicolon
                | LParen
                | LBracket
                | LBrace
                | Comma
                | Equal
                | PlusEqual
                | MinusEqual
                | StarEqual
                | SlashEqual
                | PercentEqual
                | StarStarEqual
                | PipeEqual
                | AmpersandEqual
                | CaretEqual
                | ShovelEqual
                | RightShiftEqual
                | LogicalOrAssign
                | LogicalAndAssign
                | LogicalAnd
                | LogicalOr
                | KeywordAnd
                | KeywordOr
                | NotKeyword
                | Bang
                | Return
                | Then
                | Else
                | Do
                | Pipe
                | Question
                | Colon
                | FatArrow
                | Begin
                | Ensure
        )
    )
}

/// Whether a line ending in `kind` goes on to the next line, as an operator
/// or a comma waiting for what follows does.
fn continues_on_next_line(kind: &TokenKind) -> bool {
    use TokenKind::*;
    matches!(
        kind,
        Comma
            | Dot
            | SafeDot
            | Plus
            | Minus
            | Star
            | Slash
            | Percent
            | Caret
            | Equal
            | EqualEqual
            | TripleEqual
            | BangEqual
            | Match
            | NotMatch
            | StarStar
            | Less
            | Greater
            | LessEqual
            | GreaterEqual
            | Spaceship
            | Shovel
            | RightShift
            | Question
            | Colon
            | FatArrow
            | LogicalAnd
            | LogicalOr
            | KeywordAnd
            | KeywordOr
            | PlusEqual
            | MinusEqual
            | StarEqual
            | SlashEqual
            | PercentEqual
            | StarStarEqual
            | PipeEqual
            | AmpersandEqual
            | CaretEqual
            | ShovelEqual
            | RightShiftEqual
            | LogicalOrAssign
            | LogicalAndAssign
    )
}

/// The tokens of `source` without comments, each with its nesting depth.
fn placed_tokens(tokens: &[Token]) -> Vec<Placed<'_>> {
    let mut stack: Vec<Opening> = Vec::new();
    let mut placed = Vec::new();
    let mut previous: Option<&TokenKind> = None;
    // A `while`, `until`, or `for` header may end in `do`, which belongs to
    // the loop rather than opening a block of its own.
    let mut loop_header_open = false;
    let significant: Vec<&Token> = tokens
        .iter()
        .filter(|token| !matches!(token.kind, TokenKind::Comment(_)))
        .collect();
    for (at, token) in significant.iter().enumerate() {
        let depth_before = stack.len();
        let after_dot = matches!(
            previous,
            Some(TokenKind::Dot | TokenKind::SafeDot | TokenKind::ColonColon)
        );
        match &token.kind {
            TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => {
                stack.push(Opening::Bracket)
            }
            TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
                if stack.last() == Some(&Opening::Bracket) {
                    stack.pop();
                }
            }
            TokenKind::End if !after_dot => {
                if stack.last() == Some(&Opening::Keyword) {
                    stack.pop();
                }
            }
            TokenKind::Def if !after_dot => {
                if !is_endless_definition(&significant[at + 1..]) {
                    stack.push(Opening::Keyword);
                }
            }
            TokenKind::Class | TokenKind::Module | TokenKind::Begin | TokenKind::Case
                if !after_dot =>
            {
                stack.push(Opening::Keyword)
            }
            TokenKind::For if !after_dot => {
                stack.push(Opening::Keyword);
                loop_header_open = true;
            }
            TokenKind::While | TokenKind::Until if !after_dot && begins_statement(previous) => {
                stack.push(Opening::Keyword);
                loop_header_open = true;
            }
            TokenKind::If | TokenKind::Unless if !after_dot && begins_statement(previous) => {
                stack.push(Opening::Keyword)
            }
            TokenKind::Do if !after_dot => {
                if loop_header_open {
                    loop_header_open = false;
                } else {
                    stack.push(Opening::Keyword);
                }
            }
            TokenKind::Newline | TokenKind::Semicolon => loop_header_open = false,
            _ => {}
        }
        placed.push(Placed {
            token,
            depth_before,
            depth_after: stack.len(),
        });
        previous = Some(&token.kind);
    }
    placed
}

/// Whether the `def` before `rest` is written `def name(...) = value`, which
/// has no `end`.
fn is_endless_definition(rest: &[&Token]) -> bool {
    let mut depth = 0usize;
    for token in rest {
        match token.kind {
            TokenKind::LParen => depth += 1,
            TokenKind::RParen => depth = depth.saturating_sub(1),
            TokenKind::Equal if depth == 0 => return true,
            TokenKind::Newline | TokenKind::Semicolon if depth == 0 => return false,
            TokenKind::EOF => return false,
            _ => {}
        }
    }
    false
}

/// Whether the newline at `at` ends a statement: the line before it does
/// not end in an operator, and the line after it does not open with `.`.
fn ends_statement(placed: &[Placed], at: usize) -> bool {
    let before = placed[..at]
        .iter()
        .rev()
        .find(|item| item.token.kind != TokenKind::Newline);
    if before.is_some_and(|item| continues_on_next_line(&item.token.kind)) {
        return false;
    }
    let after = placed[at + 1..]
        .iter()
        .find(|item| item.token.kind != TokenKind::Newline);
    !after.is_some_and(|item| matches!(item.token.kind, TokenKind::Dot | TokenKind::SafeDot))
}

/// The first and last line of the statement that `line` falls in, or `None`
/// when no token is written on that line.
pub(crate) fn statement_lines(source: &str, line: usize) -> Option<(usize, usize)> {
    let tokens = Lexer::new(source).tokenize();
    let placed = placed_tokens(&tokens);
    let on_line: Vec<usize> = placed
        .iter()
        .enumerate()
        .filter(|(_, item)| {
            item.token.position.line == line
                && !matches!(item.token.kind, TokenKind::Newline | TokenKind::EOF)
        })
        .map(|(at, _)| at)
        .collect();
    let first = *on_line.first()?;
    let depth = on_line
        .iter()
        .map(|&at| placed[at].depth_before.min(placed[at].depth_after))
        .min()?;

    // Back to the token after the boundary the statement opens on: a
    // newline that ends a statement at this depth, or the bracket or keyword
    // whose body the statement is in.
    let mut start = first;
    while start > 0 {
        let item = &placed[start - 1];
        let boundary = match item.token.kind {
            TokenKind::Newline | TokenKind::Semicolon => {
                item.depth_after == depth && ends_statement(&placed, start - 1)
            }
            _ => item.depth_after == depth && item.depth_before < depth,
        };
        if boundary || item.depth_after < depth {
            break;
        }
        start -= 1;
    }
    while start < first && placed[start].token.kind == TokenKind::Newline {
        start += 1;
    }

    // On to the newline that ends it, or the closing token that leaves the
    // depth it is written at.
    let mut end = on_line[on_line.len() - 1];
    let mut last_line = placed[end].token.position.line;
    while end + 1 < placed.len() {
        let item = &placed[end + 1];
        match item.token.kind {
            TokenKind::EOF => break,
            TokenKind::Newline | TokenKind::Semicolon
                if item.depth_after == depth && ends_statement(&placed, end + 1) =>
            {
                last_line = last_line.max(item.token.position.line);
                break;
            }
            _ if item.depth_after < depth => break,
            TokenKind::Newline => {}
            _ => last_line = last_line.max(item.token.position.line),
        }
        end += 1;
    }
    Some((placed[start].token.position.line, last_line))
}

/// The text of the one value `expect(...)` is handed on `line`, or `None`
/// when the line holds no such call or more than one.
pub(crate) fn expect_argument(source: &str, line: usize) -> Option<String> {
    let tokens = Lexer::new(source).tokenize();
    let mut found = None;
    for (at, token) in tokens.iter().enumerate() {
        if token.position.line != line
            || !matches!(&token.kind, TokenKind::Ident(name) if name == "expect")
        {
            continue;
        }
        let opening = tokens.get(at + 1)?;
        if opening.kind != TokenKind::LParen || opening.had_leading_space {
            continue;
        }
        let mut depth = 0usize;
        let mut close = None;
        for (offset, candidate) in tokens[at + 1..].iter().enumerate() {
            match candidate.kind {
                TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => depth += 1,
                TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(at + 1 + offset);
                        break;
                    }
                }
                TokenKind::Comma if depth == 1 => return None,
                TokenKind::EOF => return None,
                _ => {}
            }
        }
        let close = close?;
        if found.is_some() {
            return None;
        }
        let text = source.get(opening.position.offset + 1..tokens[close].position.offset)?;
        found = Some(text.trim().to_string());
    }
    found.filter(|text| !text.is_empty())
}

impl VirtualMachine {
    /// `Rex::Source.__statement_lines__(path, line)` and
    /// `Rex::Source.__expect_argument__(path, line)`.
    pub(crate) fn call_rex_source(
        &mut self,
        method_name: &str,
        arguments: &[Object],
    ) -> Result<Option<Object>, MetorexError> {
        let (Some(View::String(path)), Some(View::Int(line))) =
            (arguments.first().view(), arguments.get(1).view())
        else {
            return Ok(Some(Object::Nil));
        };
        let Ok(source) = std::fs::read_to_string(path.as_str().to_string()) else {
            return Ok(Some(Object::Nil));
        };
        let line = line as usize;
        Ok(Some(match method_name {
            "__statement_lines__" => {
                statement_lines(&source, line).map_or(Object::Nil, |(first, last)| {
                    Object::array(vec![Object::Int(first as i64), Object::Int(last as i64)])
                })
            }
            "__expect_argument__" => {
                expect_argument(&source, line).map_or(Object::Nil, Object::string)
            }
            _ => return Ok(None),
        }))
    }
}
