// The parts of `crate::error` the lexer and parser use, for the build script,
// which parses without the rest of the crate. The real type holds runtime
// values the build script has none of.

#[derive(Debug, Clone, Copy)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize, _offset: usize) -> Self {
        SourceLocation { line, column }
    }
}

#[derive(Debug, Clone)]
pub enum MetorexError {
    SyntaxError {
        message: String,
        location: SourceLocation,
    },
    RuntimeError {
        message: String,
        location: SourceLocation,
    },
}

impl MetorexError {
    pub fn syntax_error(message: impl Into<String>, location: SourceLocation) -> Self {
        MetorexError::SyntaxError {
            message: message.into(),
            location,
        }
    }

    pub fn runtime_error(message: impl Into<String>, location: SourceLocation) -> Self {
        MetorexError::RuntimeError {
            message: message.into(),
            location,
        }
    }
}
