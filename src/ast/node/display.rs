// How the nodes read back as text, and the questions they answer
// about themselves.

use super::*;

// Implement Display for BinaryOp
impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Subtract => write!(f, "-"),
            BinaryOp::Multiply => write!(f, "*"),
            BinaryOp::Divide => write!(f, "/"),
            BinaryOp::Modulo => write!(f, "%"),
            BinaryOp::Power => write!(f, "**"),
            BinaryOp::Equal => write!(f, "=="),
            BinaryOp::CaseEqual => write!(f, "==="),
            BinaryOp::NotEqual => write!(f, "!="),
            BinaryOp::Less => write!(f, "<"),
            BinaryOp::Greater => write!(f, ">"),
            BinaryOp::LessEqual => write!(f, "<="),
            BinaryOp::GreaterEqual => write!(f, ">="),
            BinaryOp::Assign => write!(f, "="),
            BinaryOp::AddAssign => write!(f, "+="),
            BinaryOp::SubtractAssign => write!(f, "-="),
            BinaryOp::MultiplyAssign => write!(f, "*="),
            BinaryOp::DivideAssign => write!(f, "/="),
            BinaryOp::And => write!(f, "&&"),
            BinaryOp::Or => write!(f, "||"),
            BinaryOp::Spaceship => write!(f, "<=>"),
            BinaryOp::BitwiseAnd => write!(f, "&"),
            BinaryOp::BitwiseOr => write!(f, "|"),
            BinaryOp::Xor => write!(f, "^"),
        }
    }
}

// Implement Display for UnaryOp
impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Plus => write!(f, "+"),
            UnaryOp::Minus => write!(f, "-"),
            UnaryOp::Not => write!(f, "!"),
        }
    }
}

// Implement helper methods for Expression
impl Expression {
    /// Get the position of this expression
    pub fn position(&self) -> Position {
        match self {
            Expression::IntLiteral { position, .. }
            | Expression::BigIntLiteral { position, .. }
            | Expression::FloatLiteral { position, .. }
            | Expression::StringLiteral { position, .. }
            | Expression::RegexLiteral { position, .. }
            | Expression::PatternTest { position, .. }
            | Expression::Symbol { position, .. }
            | Expression::InterpolatedString { position, .. }
            | Expression::BoolLiteral { position, .. }
            | Expression::NilLiteral { position, .. }
            | Expression::Identifier { position, .. }
            | Expression::InstanceVariable { position, .. }
            | Expression::ClassVariable { position, .. }
            | Expression::GlobalVariable { position, .. }
            | Expression::MagicFile { position, .. }
            | Expression::MagicLine { position, .. }
            | Expression::MagicDir { position, .. }
            | Expression::BinaryOp { position, .. }
            | Expression::UnaryOp { position, .. }
            | Expression::Call { position, .. }
            | Expression::MethodCall { position, .. }
            | Expression::Array { position, .. }
            | Expression::Index { position, .. }
            | Expression::Dictionary { position, .. }
            | Expression::Lambda { position, .. }
            | Expression::Grouped { position, .. }
            | Expression::SelfExpr { position, .. }
            | Expression::SingletonClass { position, .. }
            | Expression::Super { position, .. }
            | Expression::Splat { position, .. }
            | Expression::KeywordSplat { position, .. }
            | Expression::TopLevelConstant { position, .. }
            | Expression::BlockArg { position, .. }
            | Expression::BeginRescue { position, .. }
            | Expression::Defined { position, .. }
            | Expression::Yield { position, .. }
            | Expression::Range { position, .. }
            | Expression::Case { position, .. }
            | Expression::ScopeResolution { position, .. }
            | Expression::If { position, .. }
            | Expression::Unless { position, .. } => *position,
        }
    }

    /// Check if this expression is a literal
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Expression::IntLiteral { .. }
                | Expression::FloatLiteral { .. }
                | Expression::StringLiteral { .. }
                | Expression::InterpolatedString { .. }
                | Expression::BoolLiteral { .. }
                | Expression::NilLiteral { .. }
        )
    }

    /// Check if this expression is an identifier or variable
    pub fn is_identifier(&self) -> bool {
        matches!(
            self,
            Expression::Identifier { .. }
                | Expression::InstanceVariable { .. }
                | Expression::ClassVariable { .. }
        )
    }
}

// Implement helper methods for Statement
impl Statement {
    /// Get the position of this statement
    pub fn position(&self) -> Position {
        match self {
            Statement::Expression { position, .. }
            | Statement::Assignment { position, .. }
            | Statement::FunctionDef { position, .. }
            | Statement::MethodDef { position, .. }
            | Statement::ClassDef { position, .. }
            | Statement::If { position, .. }
            | Statement::Unless { position, .. }
            | Statement::While { position, .. }
            | Statement::For { position, .. }
            | Statement::Match { position, .. }
            | Statement::CaseIn { position, .. }
            | Statement::Return { position, .. }
            | Statement::Break { position, .. }
            | Statement::Continue { position, .. }
            | Statement::Redo { position }
            | Statement::Retry { position }
            | Statement::Block { position, .. }
            | Statement::Begin { position, .. }
            | Statement::Raise { position, .. }
            | Statement::AttrReader { position, .. }
            | Statement::AttrWriter { position, .. }
            | Statement::AttrAccessor { position, .. }
            | Statement::ModuleDef { position, .. }
            | Statement::Include { position, .. }
            | Statement::Extend { position, .. }
            | Statement::Alias { position, .. }
            | Statement::DoWhile { position, .. }
            | Statement::DeclareLocals { position, .. }
            | Statement::BeginBlock { position, .. }
            | Statement::MultipleAssignment { position, .. } => *position,
        }
    }

    /// Check if this statement is a definition (function, method, class, or module)
    pub fn is_definition(&self) -> bool {
        matches!(
            self,
            Statement::FunctionDef { .. }
                | Statement::MethodDef { .. }
                | Statement::ClassDef { .. }
                | Statement::ModuleDef { .. }
        )
    }

    /// Check if this statement is a control flow statement
    pub fn is_control_flow(&self) -> bool {
        matches!(
            self,
            Statement::If { .. }
                | Statement::While { .. }
                | Statement::For { .. }
                | Statement::Match { .. }
                | Statement::CaseIn { .. }
                | Statement::Return { .. }
                | Statement::Break { .. }
                | Statement::Continue { .. }
                | Statement::Redo { .. }
                | Statement::Retry { .. }
                | Statement::Begin { .. }
                | Statement::Raise { .. }
        )
    }
}

impl Expression {
    /// The name of this kind of expression, as `METOREX_COUNTERS` reports it.
    pub fn kind_name(&self) -> &'static str {
        match self {
            Expression::IntLiteral { .. } => "IntLiteral",
            Expression::FloatLiteral { .. } => "FloatLiteral",
            Expression::StringLiteral { .. } => "StringLiteral",
            Expression::RegexLiteral { .. } => "RegexLiteral",
            Expression::PatternTest { .. } => "PatternTest",
            Expression::InterpolatedString { .. } => "InterpolatedString",
            Expression::BoolLiteral { .. } => "BoolLiteral",
            Expression::NilLiteral { .. } => "NilLiteral",
            Expression::Symbol { .. } => "Symbol",
            Expression::Identifier { .. } => "Identifier",
            Expression::InstanceVariable { .. } => "InstanceVariable",
            Expression::ClassVariable { .. } => "ClassVariable",
            Expression::GlobalVariable { .. } => "GlobalVariable",
            Expression::MagicFile { .. } => "MagicFile",
            Expression::MagicLine { .. } => "MagicLine",
            Expression::MagicDir { .. } => "MagicDir",
            Expression::BinaryOp { .. } => "BinaryOp",
            Expression::UnaryOp { .. } => "UnaryOp",
            Expression::Call { .. } => "Call",
            Expression::MethodCall { .. } => "MethodCall",
            Expression::Array { .. } => "Array",
            Expression::Index { .. } => "Index",
            Expression::Dictionary { .. } => "Dictionary",
            Expression::Lambda { .. } => "Lambda",
            Expression::Grouped { .. } => "Grouped",
            Expression::SelfExpr { .. } => "SelfExpr",
            Expression::SingletonClass { .. } => "SingletonClass",
            Expression::Super { .. } => "Super",
            Expression::BigIntLiteral { .. } => "BigIntLiteral",
            Expression::Splat { .. } => "Splat",
            Expression::KeywordSplat { .. } => "KeywordSplat",
            Expression::TopLevelConstant { .. } => "TopLevelConstant",
            Expression::BlockArg { .. } => "BlockArg",
            Expression::BeginRescue { .. } => "BeginRescue",
            Expression::Defined { .. } => "Defined",
            Expression::Yield { .. } => "Yield",
            Expression::Range { .. } => "Range",
            Expression::Case { .. } => "Case",
            Expression::If { .. } => "If",
            Expression::Unless { .. } => "Unless",
            Expression::ScopeResolution { .. } => "ScopeResolution",
        }
    }
}

impl Statement {
    /// The name of this kind of statement, as `METOREX_COUNTERS` reports it.
    pub fn kind_name(&self) -> &'static str {
        match self {
            Statement::Expression { .. } => "Expression",
            Statement::Assignment { .. } => "Assignment",
            Statement::DoWhile { .. } => "DoWhile",
            Statement::DeclareLocals { .. } => "DeclareLocals",
            Statement::BeginBlock { .. } => "BeginBlock",
            Statement::MultipleAssignment { .. } => "MultipleAssignment",
            Statement::FunctionDef { .. } => "FunctionDef",
            Statement::MethodDef { .. } => "MethodDef",
            Statement::ClassDef { .. } => "ClassDef",
            Statement::ModuleDef { .. } => "ModuleDef",
            Statement::Include { .. } => "Include",
            Statement::Extend { .. } => "Extend",
            Statement::Alias { .. } => "Alias",
            Statement::If { .. } => "If",
            Statement::Unless { .. } => "Unless",
            Statement::While { .. } => "While",
            Statement::For { .. } => "For",
            Statement::Match { .. } => "Match",
            Statement::CaseIn { .. } => "CaseIn",
            Statement::Return { .. } => "Return",
            Statement::Break { .. } => "Break",
            Statement::Continue { .. } => "Continue",
            Statement::Redo { .. } => "Redo",
            Statement::Retry { .. } => "Retry",
            Statement::Block { .. } => "Block",
            Statement::Begin { .. } => "Begin",
            Statement::Raise { .. } => "Raise",
            Statement::AttrReader { .. } => "AttrReader",
            Statement::AttrWriter { .. } => "AttrWriter",
            Statement::AttrAccessor { .. } => "AttrAccessor",
        }
    }
}
