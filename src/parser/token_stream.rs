// Token stream management for the parser
// Handles token navigation, matching, and whitespace handling

use std::cell::UnsafeCell;
use std::collections::VecDeque;

use crate::lexer::{Lexer, Position, Token, TokenKind};

/// How many tokens a chunk of the stream holds, as a power of two.
const CHUNK_BITS: usize = 12;
const CHUNK_LENGTH: usize = 1 << CHUNK_BITS;

/// Encapsulates token navigation state and operations. The tokens are read
/// from their source as the parser reaches them, into chunks that never
/// move once made, and the chunks the parser has passed are released.
pub struct TokenStream<'source> {
    store: UnsafeCell<Store<'source>>,
    /// Current position in the token stream
    current: usize,
}

/// Where the tokens come from.
enum Source<'source> {
    Tokens(std::vec::IntoIter<Token>),
    Lexer(Box<Lexer<'source>>),
}

struct Store<'source> {
    /// The tokens read and not released, each chunk made with room for
    /// `CHUNK_LENGTH` and never grown past it, so a token stays where it
    /// was read until its chunk is released.
    chunks: VecDeque<Vec<Token>>,
    /// The number of the first chunk held, counting released ones.
    first_chunk: usize,
    /// How many tokens have been read.
    read: usize,
    /// Where the end-of-file token is, once it has been read.
    end: Option<usize>,
    /// A released chunk, emptied, which the next chunk is made in.
    spare: Option<Vec<Token>>,
    source: Source<'source>,
}

impl Store<'_> {
    /// Read tokens until the one at `index` or the end of the file has been.
    fn read_to(&mut self, index: usize) {
        while self.end.is_none() && self.read <= index {
            let token = match &mut self.source {
                Source::Tokens(tokens) => tokens.next(),
                Source::Lexer(lexer) => {
                    let _counted = crate::memory_category::attribute(
                        crate::memory_category::Category::SyntaxTree,
                    );
                    Some(lexer.next_token())
                }
            }
            .unwrap_or_else(|| Token::new(TokenKind::EOF, Position::default()));
            if token.kind == TokenKind::EOF {
                self.end = Some(self.read);
            }
            match self.chunks.back_mut() {
                Some(chunk) if chunk.len() < CHUNK_LENGTH => chunk.push(token),
                _ => {
                    let mut chunk = self
                        .spare
                        .take()
                        .unwrap_or_else(|| Vec::with_capacity(CHUNK_LENGTH));
                    chunk.push(token);
                    self.chunks.push_back(chunk);
                }
            }
            self.read += 1;
        }
    }
}

/// The tokens of a stream, read by where they are in it.
pub struct Tokens<'stream, 'source>(&'stream TokenStream<'source>);

impl<'stream, 'source> Tokens<'stream, 'source> {
    /// The token at `index`, if the file reaches that far.
    pub fn get(&self, index: usize) -> Option<&'stream Token> {
        self.0.token_within(index)
    }

    /// The tokens from `from` up to `to`.
    pub fn between(
        &self,
        from: usize,
        to: usize,
    ) -> impl Iterator<Item = &'stream Token> + use<'stream, 'source> {
        let stream = self.0;
        (from..to).map(move |index| stream.token_within(index).expect("a token within the file"))
    }
}

impl<'stream> std::ops::Index<usize> for Tokens<'stream, '_> {
    type Output = Token;

    fn index(&self, index: usize) -> &Token {
        self.0.token_within(index).expect("a token within the file")
    }
}

impl<'source> TokenStream<'source> {
    /// Create a new token stream from a vector of tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self::reading(Source::Tokens(tokens.into_iter()))
    }

    /// A token stream reading its tokens from `lexer` as they are reached.
    pub fn lexing(lexer: Lexer<'source>) -> Self {
        Self::reading(Source::Lexer(Box::new(lexer)))
    }

    fn reading(source: Source<'source>) -> Self {
        Self {
            store: UnsafeCell::new(Store {
                chunks: VecDeque::new(),
                first_chunk: 0,
                read: 0,
                end: None,
                spare: None,
                source,
            }),
            current: 0,
        }
    }

    /// The token at `index`, or None past the end of the file.
    fn token_within(&self, index: usize) -> Option<&Token> {
        // SAFETY: reading more tokens only pushes onto a chunk with room or
        // adds a chunk, so no token a reference was handed out for moves,
        // and chunks are released only through `&mut self`, when no such
        // reference is alive. The stream is not shared between threads.
        let store = unsafe { &mut *self.store.get() };
        store.read_to(index);
        if store.end.is_some_and(|end| index > end) {
            return None;
        }
        let chunk = (index >> CHUNK_BITS)
            .checked_sub(store.first_chunk)
            .expect("a token the parser has not released");
        let token: *const Token = &store.chunks[chunk][index & (CHUNK_LENGTH - 1)];
        // SAFETY: as above, the token stays where it is while `self` is
        // borrowed.
        Some(unsafe { &*token })
    }

    /// The token at `index`, or the end-of-file token past the end.
    fn token(&self, index: usize) -> &Token {
        match self.token_within(index) {
            Some(token) => token,
            None => {
                // SAFETY: only the end, which `token_within` set, is read.
                let end = unsafe { (*self.store.get()).end }.expect("the end was read");
                self.token_within(end).expect("the end-of-file token")
            }
        }
    }

    /// Release the chunks wholly before the token at `index`, which the
    /// parser does not read again.
    pub fn release_before(&mut self, index: usize) {
        let store = self.store.get_mut();
        while store.chunks.len() > 1 && (store.first_chunk + 1) << CHUNK_BITS <= index {
            let mut released = store.chunks.pop_front().expect("a chunk before the last");
            released.clear();
            store.spare = Some(released);
            store.first_chunk += 1;
        }
    }

    /// Get the current token without consuming it
    pub fn peek(&self) -> &Token {
        self.token(self.current)
    }

    /// Get the token at an offset from the current position
    #[allow(dead_code)]
    pub fn peek_ahead(&self, offset: usize) -> &Token {
        self.token(self.current + offset)
    }

    /// Get the previous token
    pub fn previous(&self) -> &Token {
        self.token(self.current.saturating_sub(1))
    }

    /// Check if we're at the end of the token stream
    pub fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::EOF)
    }

    /// Advance to the next token and return the previous one
    pub fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous().clone()
    }

    /// Check if the current token matches any of the given kinds
    pub fn check(&self, kinds: &[TokenKind]) -> bool {
        if self.is_at_end() {
            return false;
        }
        kinds.iter().any(|kind| self.match_kind(kind))
    }

    /// Check if the current token matches a specific kind (handles complex matching)
    pub fn match_kind(&self, kind: &TokenKind) -> bool {
        let current = &self.peek().kind;
        match (kind, current) {
            (TokenKind::Ident(_), TokenKind::Ident(_)) => true,
            (TokenKind::Int(_), TokenKind::Int(_)) => true,
            (TokenKind::BigInt(_), TokenKind::BigInt(_)) => true,
            (TokenKind::Float(_), TokenKind::Float(_)) => true,
            (TokenKind::String(_), TokenKind::String(_)) => true,
            (TokenKind::InterpolatedString(_), TokenKind::InterpolatedString(_)) => true,
            (TokenKind::InstanceVar(_), TokenKind::InstanceVar(_)) => true,
            (TokenKind::ClassVar(_), TokenKind::ClassVar(_)) => true,
            (TokenKind::Comment(_), TokenKind::Comment(_)) => true,
            _ => kind == current,
        }
    }

    /// Consume the current token if it matches any of the given kinds
    pub fn match_token(&mut self, kinds: &[TokenKind]) -> bool {
        if self.check(kinds) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Skip any newline tokens
    #[allow(dead_code)]
    pub fn skip_newlines(&mut self) {
        while self.match_token(&[TokenKind::Newline]) {
            // Keep consuming newlines
        }
    }

    /// Skip any comment tokens
    #[allow(dead_code)]
    pub fn skip_comments(&mut self) {
        while matches!(self.peek().kind, TokenKind::Comment(_)) {
            self.advance();
        }
    }

    /// Skip newlines and comments
    pub fn skip_whitespace(&mut self) {
        while let TokenKind::Newline | TokenKind::Comment(_) | TokenKind::Semicolon =
            &self.peek().kind
        {
            self.advance();
        }
    }

    /// The tokens, read by where they are in the stream.
    pub fn tokens(&self) -> Tokens<'_, 'source> {
        Tokens(self)
    }

    /// Get the current position in the token stream
    pub fn current_position(&self) -> usize {
        self.current
    }

    /// Restore the position in the token stream (for backtracking)
    pub fn restore_position(&mut self, position: usize) {
        self.current = position;
    }
}
