//! Expression AST types.

use core::fmt;

use crate::lexer::Span;

/// A literal value.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Integer literal.
    Integer(i64),
    /// Float literal.
    Float(f64),
    /// String literal.
    String(String),
    /// Blob literal.
    Blob(Vec<u8>),
    /// Boolean literal.
    Boolean(bool),
    /// NULL literal.
    Null,
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,

    // Logical
    And,
    Or,

    // String
    Concat,
    Like,

    // Bitwise
    BitAnd,
    BitOr,
    LeftShift,
    RightShift,
}

impl BinaryOp {
    /// Returns the SQL representation of the operator.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Mod => "%",
            Self::Eq => "=",
            Self::NotEq => "!=",
            Self::Lt => "<",
            Self::LtEq => "<=",
            Self::Gt => ">",
            Self::GtEq => ">=",
            Self::And => "AND",
            Self::Or => "OR",
            Self::Concat => "||",
            Self::Like => "LIKE",
            Self::BitAnd => "&",
            Self::BitOr => "|",
            Self::LeftShift => "<<",
            Self::RightShift => ">>",
        }
    }

    /// Returns the precedence of the operator (higher = binds tighter).
    #[must_use]
    pub const fn precedence(&self) -> u8 {
        match self {
            Self::Or => 1,
            Self::And => 2,
            Self::Eq | Self::NotEq | Self::Lt | Self::LtEq | Self::Gt | Self::GtEq => 3,
            Self::Like => 4,
            Self::BitOr => 5,
            Self::BitAnd => 6,
            Self::LeftShift | Self::RightShift => 7,
            Self::Add | Self::Sub | Self::Concat => 8,
            Self::Mul | Self::Div | Self::Mod => 9,
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(n) => write!(f, "{n}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::String(s) => {
                let escaped = s.replace('\'', "''");
                write!(f, "'{escaped}'")
            }
            Self::Blob(bytes) => {
                write!(f, "X'")?;
                for b in bytes {
                    write!(f, "{b:02X}")?;
                }
                write!(f, "'")
            }
            Self::Boolean(true) => write!(f, "TRUE"),
            Self::Boolean(false) => write!(f, "FALSE"),
            Self::Null => write!(f, "NULL"),
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// Negation (-)
    Neg,
    /// Logical NOT
    Not,
    /// Bitwise NOT (~)
    BitNot,
}

impl UnaryOp {
    /// Returns the SQL representation of the operator.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Neg => "-",
            Self::Not => "NOT",
            Self::BitNot => "~",
        }
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A function call expression.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    /// The function name.
    pub name: String,
    /// The arguments.
    pub args: Vec<Expr>,
    /// Whether DISTINCT was specified.
    pub distinct: bool,
}

/// An SQL expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A literal value.
    Literal(Literal),

    /// A column reference (optionally qualified with table name).
    Column {
        /// Table name or alias (optional).
        table: Option<String>,
        /// Column name.
        name: String,
        /// Source span.
        span: Span,
    },

    /// A binary expression.
    Binary {
        /// Left operand.
        left: Box<Expr>,
        /// Operator.
        op: BinaryOp,
        /// Right operand.
        right: Box<Expr>,
    },

    /// A unary expression.
    Unary {
        /// Operator.
        op: UnaryOp,
        /// Operand.
        operand: Box<Expr>,
    },

    /// A function call.
    Function(FunctionCall),

    /// A subquery.
    Subquery(Box<super::SelectStatement>),

    /// IS NULL expression.
    IsNull {
        /// The expression to check.
        expr: Box<Expr>,
        /// Whether this is IS NOT NULL.
        negated: bool,
    },

    /// IN expression.
    In {
        /// The expression to check.
        expr: Box<Expr>,
        /// The list of values or subquery.
        list: Vec<Expr>,
        /// Whether this is NOT IN.
        negated: bool,
    },

    /// BETWEEN expression.
    Between {
        /// The expression to check.
        expr: Box<Expr>,
        /// Lower bound.
        low: Box<Expr>,
        /// Upper bound.
        high: Box<Expr>,
        /// Whether this is NOT BETWEEN.
        negated: bool,
    },

    /// CASE expression.
    Case {
        /// The operand (if any).
        operand: Option<Box<Expr>>,
        /// WHEN/THEN clauses.
        when_clauses: Vec<(Expr, Expr)>,
        /// ELSE clause.
        else_clause: Option<Box<Expr>>,
    },

    /// CAST expression.
    Cast {
        /// Expression to cast.
        expr: Box<Expr>,
        /// Target type.
        data_type: super::DataType,
    },

    /// Parenthesized expression.
    Paren(Box<Expr>),

    /// A parameter placeholder (? or :name).
    Parameter {
        /// The parameter index or name.
        name: Option<String>,
        /// Position in the query (1-based for ? placeholders).
        position: usize,
    },

    /// Wildcard (*) in SELECT.
    Wildcard {
        /// Table qualifier (optional).
        table: Option<String>,
    },
}

impl fmt::Display for FunctionCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // EXISTS gets special handling: the subquery already
        // contains its own parentheses in the rendered form,
        // so we render `EXISTS(SELECT ...)` instead of
        // `EXISTS((SELECT ...))`.
        if self.name == "EXISTS" {
            if let [Expr::Subquery(q)] = self.args.as_slice() {
                return write!(f, "EXISTS({q})");
            }
        }
        write!(f, "{}(", self.name)?;
        if self.distinct {
            write!(f, "DISTINCT ")?;
        }
        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{arg}")?;
        }
        write!(f, ")")
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(lit) => write!(f, "{lit}"),
            Self::Column { table, name, .. } => {
                if let Some(t) = table {
                    write!(f, "{t}.{name}")
                } else {
                    write!(f, "{name}")
                }
            }
            Self::Binary { left, op, right } => {
                write!(f, "{left} {op} {right}")
            }
            Self::Unary { op, operand } => match op {
                UnaryOp::Not => write!(f, "NOT {operand}"),
                UnaryOp::Neg => write!(f, "-{operand}"),
                UnaryOp::BitNot => write!(f, "~{operand}"),
            },
            Self::Function(func) => write!(f, "{func}"),
            Self::Subquery(q) => write!(f, "({q})"),
            Self::IsNull { expr, negated } => {
                if *negated {
                    write!(f, "{expr} IS NOT NULL")
                } else {
                    write!(f, "{expr} IS NULL")
                }
            }
            Self::In {
                expr,
                list,
                negated,
            } => {
                write!(f, "{expr}")?;
                if *negated {
                    write!(f, " NOT IN (")?;
                } else {
                    write!(f, " IN (")?;
                }
                for (i, item) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, ")")
            }
            Self::Between {
                expr,
                low,
                high,
                negated,
            } => {
                if *negated {
                    write!(f, "{expr} NOT BETWEEN {low} AND {high}")
                } else {
                    write!(f, "{expr} BETWEEN {low} AND {high}")
                }
            }
            Self::Case {
                operand,
                when_clauses,
                else_clause,
            } => {
                write!(f, "CASE")?;
                if let Some(op) = operand {
                    write!(f, " {op}")?;
                }
                for (when, then) in when_clauses {
                    write!(f, " WHEN {when} THEN {then}")?;
                }
                if let Some(el) = else_clause {
                    write!(f, " ELSE {el}")?;
                }
                write!(f, " END")
            }
            Self::Cast { expr, data_type } => {
                write!(f, "CAST({expr} AS {data_type})")
            }
            Self::Paren(inner) => write!(f, "({inner})"),
            Self::Parameter { name, .. } => {
                if let Some(n) = name {
                    write!(f, ":{n}")
                } else {
                    write!(f, "?")
                }
            }
            Self::Wildcard { table } => {
                if let Some(t) = table {
                    write!(f, "{t}.*")
                } else {
                    write!(f, "*")
                }
            }
        }
    }
}

impl Expr {
    /// Creates a new column reference.
    #[must_use]
    pub fn column(name: impl Into<String>) -> Self {
        Self::Column {
            table: None,
            name: name.into(),
            span: Span::default(),
        }
    }

    /// Creates a new qualified column reference.
    #[must_use]
    pub fn qualified_column(table: impl Into<String>, name: impl Into<String>) -> Self {
        Self::Column {
            table: Some(table.into()),
            name: name.into(),
            span: Span::default(),
        }
    }

    /// Creates a new integer literal.
    #[must_use]
    pub const fn integer(value: i64) -> Self {
        Self::Literal(Literal::Integer(value))
    }

    /// Creates a new float literal.
    #[must_use]
    pub const fn float(value: f64) -> Self {
        Self::Literal(Literal::Float(value))
    }

    /// Creates a new string literal.
    #[must_use]
    pub fn string(value: impl Into<String>) -> Self {
        Self::Literal(Literal::String(value.into()))
    }

    /// Creates a new boolean literal.
    #[must_use]
    pub const fn boolean(value: bool) -> Self {
        Self::Literal(Literal::Boolean(value))
    }

    /// Creates a NULL literal.
    #[must_use]
    pub const fn null() -> Self {
        Self::Literal(Literal::Null)
    }

    /// Creates a binary expression.
    #[must_use]
    pub fn binary(self, op: BinaryOp, right: Self) -> Self {
        Self::Binary {
            left: Box::new(self),
            op,
            right: Box::new(right),
        }
    }

    /// Creates an equality expression.
    #[must_use]
    pub fn eq(self, right: Self) -> Self {
        self.binary(BinaryOp::Eq, right)
    }

    /// Creates an inequality expression.
    #[must_use]
    pub fn not_eq(self, right: Self) -> Self {
        self.binary(BinaryOp::NotEq, right)
    }

    /// Creates a less-than expression.
    #[must_use]
    pub fn lt(self, right: Self) -> Self {
        self.binary(BinaryOp::Lt, right)
    }

    /// Creates a less-than-or-equal expression.
    #[must_use]
    pub fn lt_eq(self, right: Self) -> Self {
        self.binary(BinaryOp::LtEq, right)
    }

    /// Creates a greater-than expression.
    #[must_use]
    pub fn gt(self, right: Self) -> Self {
        self.binary(BinaryOp::Gt, right)
    }

    /// Creates a greater-than-or-equal expression.
    #[must_use]
    pub fn gt_eq(self, right: Self) -> Self {
        self.binary(BinaryOp::GtEq, right)
    }

    /// Creates an AND expression.
    #[must_use]
    pub fn and(self, right: Self) -> Self {
        self.binary(BinaryOp::And, right)
    }

    /// Creates an OR expression.
    #[must_use]
    pub fn or(self, right: Self) -> Self {
        self.binary(BinaryOp::Or, right)
    }

    /// Creates an IS NULL expression.
    #[must_use]
    pub fn is_null(self) -> Self {
        Self::IsNull {
            expr: Box::new(self),
            negated: false,
        }
    }

    /// Creates an IS NOT NULL expression.
    #[must_use]
    pub fn is_not_null(self) -> Self {
        Self::IsNull {
            expr: Box::new(self),
            negated: true,
        }
    }

    /// Creates a BETWEEN expression.
    #[must_use]
    pub fn between(self, low: Self, high: Self) -> Self {
        Self::Between {
            expr: Box::new(self),
            low: Box::new(low),
            high: Box::new(high),
            negated: false,
        }
    }

    /// Creates a NOT BETWEEN expression.
    #[must_use]
    pub fn not_between(self, low: Self, high: Self) -> Self {
        Self::Between {
            expr: Box::new(self),
            low: Box::new(low),
            high: Box::new(high),
            negated: true,
        }
    }

    /// Creates an IN expression.
    #[must_use]
    pub fn in_list(self, list: Vec<Self>) -> Self {
        Self::In {
            expr: Box::new(self),
            list,
            negated: false,
        }
    }

    /// Creates a NOT IN expression.
    #[must_use]
    pub fn not_in_list(self, list: Vec<Self>) -> Self {
        Self::In {
            expr: Box::new(self),
            list,
            negated: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_op_precedence() {
        assert!(BinaryOp::Mul.precedence() > BinaryOp::Add.precedence());
        assert!(BinaryOp::And.precedence() > BinaryOp::Or.precedence());
        assert!(BinaryOp::Eq.precedence() > BinaryOp::And.precedence());
    }

    #[test]
    fn test_expr_builders() {
        let col = Expr::column("name");
        assert!(matches!(col, Expr::Column { name, .. } if name == "name"));

        let lit = Expr::integer(42);
        assert!(matches!(lit, Expr::Literal(Literal::Integer(42))));
    }

    #[test]
    fn test_expr_chaining() {
        let expr = Expr::column("age")
            .gt(Expr::integer(18))
            .and(Expr::column("status").eq(Expr::string("active")));

        assert!(matches!(
            expr,
            Expr::Binary {
                op: BinaryOp::And,
                ..
            }
        ));
    }
}
