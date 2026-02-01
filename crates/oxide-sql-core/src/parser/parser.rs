//! SQL Parser implementation.

use super::error::ParseError;
use super::pratt::{
    infix_binding_power, prefix_binding_power, token_to_binary_op, token_to_unary_op,
};
use crate::ast::{
    DataType, DeleteStatement, Expr, FunctionCall, InsertSource, InsertStatement, JoinClause,
    JoinType, Literal, OrderBy, OrderDirection, SelectColumn, SelectStatement, Statement, TableRef,
    UpdateAssignment, UpdateStatement,
};
use crate::lexer::{Keyword, Lexer, Span, Token, TokenKind};

/// SQL Parser.
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
    previous: Token,
    /// Parameter counter for ? placeholders.
    param_counter: usize,
}

impl<'a> Parser<'a> {
    /// Creates a new parser for the given input.
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer::new(input);
        let current = lexer.next_token();
        Self {
            lexer,
            current,
            previous: Token::new(TokenKind::Eof, Span::new(0, 0)),
            param_counter: 0,
        }
    }

    /// Parses a single SQL statement.
    ///
    /// # Errors
    ///
    /// Returns a `ParseError` if the input is not a valid SQL statement.
    pub fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match &self.current.kind {
            TokenKind::Keyword(Keyword::Select) => {
                Ok(Statement::Select(self.parse_select_statement()?))
            }
            TokenKind::Keyword(Keyword::Insert) => {
                Ok(Statement::Insert(self.parse_insert_statement()?))
            }
            TokenKind::Keyword(Keyword::Update) => {
                Ok(Statement::Update(self.parse_update_statement()?))
            }
            TokenKind::Keyword(Keyword::Delete) => {
                Ok(Statement::Delete(self.parse_delete_statement()?))
            }
            _ => Err(ParseError::unexpected(
                "SELECT, INSERT, UPDATE, or DELETE",
                self.current.kind.clone(),
                self.current.span,
            )),
        }
    }

    /// Parses a SELECT statement.
    fn parse_select_statement(&mut self) -> Result<SelectStatement, ParseError> {
        self.expect_keyword(Keyword::Select)?;

        // DISTINCT or ALL
        let distinct = if self.check_keyword(Keyword::Distinct) {
            self.advance();
            true
        } else if self.check_keyword(Keyword::All) {
            self.advance();
            false
        } else {
            false
        };

        // SELECT columns
        let columns = self.parse_select_columns()?;

        // FROM clause (optional for expressions like SELECT 1+1)
        let from = if self.check_keyword(Keyword::From) {
            self.advance();
            Some(self.parse_table_ref()?)
        } else {
            None
        };

        // WHERE clause
        let where_clause = if self.check_keyword(Keyword::Where) {
            self.advance();
            Some(self.parse_expression(0)?)
        } else {
            None
        };

        // GROUP BY clause
        let group_by = if self.check_keyword(Keyword::Group) {
            self.advance();
            self.expect_keyword(Keyword::By)?;
            self.parse_expression_list()?
        } else {
            vec![]
        };

        // HAVING clause
        let having = if self.check_keyword(Keyword::Having) {
            self.advance();
            Some(self.parse_expression(0)?)
        } else {
            None
        };

        // ORDER BY clause
        let order_by = if self.check_keyword(Keyword::Order) {
            self.advance();
            self.expect_keyword(Keyword::By)?;
            self.parse_order_by_list()?
        } else {
            vec![]
        };

        // LIMIT clause
        let limit = if self.check_keyword(Keyword::Limit) {
            self.advance();
            Some(self.parse_expression(0)?)
        } else {
            None
        };

        // OFFSET clause
        let offset = if self.check_keyword(Keyword::Offset) {
            self.advance();
            Some(self.parse_expression(0)?)
        } else {
            None
        };

        Ok(SelectStatement {
            distinct,
            columns,
            from,
            where_clause,
            group_by,
            having,
            order_by,
            limit,
            offset,
        })
    }

    /// Parses SELECT columns.
    fn parse_select_columns(&mut self) -> Result<Vec<SelectColumn>, ParseError> {
        let mut columns = vec![];

        loop {
            let expr = self.parse_expression(0)?;

            // Check for alias (AS name or just name)
            let alias = if self.check_keyword(Keyword::As) {
                self.advance();
                Some(self.expect_identifier()?)
            } else if matches!(&self.current.kind, TokenKind::Identifier(_)) {
                Some(self.expect_identifier()?)
            } else {
                None
            };

            columns.push(SelectColumn { expr, alias });

            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        Ok(columns)
    }

    /// Parses a table reference.
    fn parse_table_ref(&mut self) -> Result<TableRef, ParseError> {
        let mut table_ref = if self.check(&TokenKind::LeftParen) {
            // Subquery or grouped table ref
            self.advance();
            if self.check_keyword(Keyword::Select) {
                let query = self.parse_select_statement()?;
                self.expect(&TokenKind::RightParen)?;
                let alias = self.parse_optional_alias()?;
                TableRef::Subquery {
                    query: Box::new(query),
                    alias: alias.unwrap_or_else(|| String::from("subquery")),
                }
            } else {
                let inner = self.parse_table_ref()?;
                self.expect(&TokenKind::RightParen)?;
                inner
            }
        } else {
            // Simple table name
            let first = self.expect_identifier()?;
            let (schema, name) = if self.check(&TokenKind::Dot) {
                self.advance();
                let table_name = self.expect_identifier()?;
                (Some(first), table_name)
            } else {
                (None, first)
            };

            let alias = self.parse_optional_alias()?;

            TableRef::Table {
                schema,
                name,
                alias,
            }
        };

        // Parse joins
        while self.is_join_keyword() {
            let join_type = self.parse_join_type()?;
            let right = self.parse_simple_table_ref()?;

            let (on, using) = if join_type == JoinType::Cross {
                (None, vec![])
            } else if self.check_keyword(Keyword::On) {
                self.advance();
                (Some(self.parse_expression(0)?), vec![])
            } else if self.check_keyword(Keyword::Using) {
                self.advance();
                self.expect(&TokenKind::LeftParen)?;
                let cols = self.parse_identifier_list()?;
                self.expect(&TokenKind::RightParen)?;
                (None, cols)
            } else {
                return Err(ParseError::new(
                    "Expected ON or USING clause",
                    self.current.span,
                ));
            };

            table_ref = TableRef::Join {
                left: Box::new(table_ref),
                join: Box::new(JoinClause {
                    join_type,
                    table: right,
                    on,
                    using,
                }),
            };
        }

        Ok(table_ref)
    }

    /// Parses a simple table reference (no joins).
    fn parse_simple_table_ref(&mut self) -> Result<TableRef, ParseError> {
        let first = self.expect_identifier()?;
        let (schema, name) = if self.check(&TokenKind::Dot) {
            self.advance();
            let table_name = self.expect_identifier()?;
            (Some(first), table_name)
        } else {
            (None, first)
        };

        let alias = self.parse_optional_alias()?;

        Ok(TableRef::Table {
            schema,
            name,
            alias,
        })
    }

    /// Checks if current token is a join keyword.
    fn is_join_keyword(&self) -> bool {
        matches!(
            &self.current.kind,
            TokenKind::Keyword(
                Keyword::Join
                    | Keyword::Inner
                    | Keyword::Left
                    | Keyword::Right
                    | Keyword::Full
                    | Keyword::Cross
            )
        )
    }

    /// Parses a join type.
    fn parse_join_type(&mut self) -> Result<JoinType, ParseError> {
        let join_type = match &self.current.kind {
            TokenKind::Keyword(Keyword::Join) => {
                self.advance();
                JoinType::Inner
            }
            TokenKind::Keyword(Keyword::Inner) => {
                self.advance();
                self.expect_keyword(Keyword::Join)?;
                JoinType::Inner
            }
            TokenKind::Keyword(Keyword::Left) => {
                self.advance();
                if self.check_keyword(Keyword::Outer) {
                    self.advance();
                }
                self.expect_keyword(Keyword::Join)?;
                JoinType::Left
            }
            TokenKind::Keyword(Keyword::Right) => {
                self.advance();
                if self.check_keyword(Keyword::Outer) {
                    self.advance();
                }
                self.expect_keyword(Keyword::Join)?;
                JoinType::Right
            }
            TokenKind::Keyword(Keyword::Full) => {
                self.advance();
                if self.check_keyword(Keyword::Outer) {
                    self.advance();
                }
                self.expect_keyword(Keyword::Join)?;
                JoinType::Full
            }
            TokenKind::Keyword(Keyword::Cross) => {
                self.advance();
                self.expect_keyword(Keyword::Join)?;
                JoinType::Cross
            }
            _ => {
                return Err(ParseError::unexpected(
                    "JOIN keyword",
                    self.current.kind.clone(),
                    self.current.span,
                ));
            }
        };
        Ok(join_type)
    }

    /// Parses an optional table alias.
    fn parse_optional_alias(&mut self) -> Result<Option<String>, ParseError> {
        if self.check_keyword(Keyword::As) {
            self.advance();
            Ok(Some(self.expect_identifier()?))
        } else if matches!(&self.current.kind, TokenKind::Identifier(_)) && !self.is_reserved_word()
        {
            Ok(Some(self.expect_identifier()?))
        } else {
            Ok(None)
        }
    }

    /// Checks if current identifier is a reserved word.
    fn is_reserved_word(&self) -> bool {
        matches!(
            &self.current.kind,
            TokenKind::Keyword(
                Keyword::Where
                    | Keyword::Order
                    | Keyword::Group
                    | Keyword::Having
                    | Keyword::Limit
                    | Keyword::Offset
                    | Keyword::Join
                    | Keyword::Inner
                    | Keyword::Left
                    | Keyword::Right
                    | Keyword::Full
                    | Keyword::Cross
                    | Keyword::On
                    | Keyword::Using
                    | Keyword::Union
                    | Keyword::Intersect
                    | Keyword::Except
            )
        )
    }

    /// Parses an INSERT statement.
    fn parse_insert_statement(&mut self) -> Result<InsertStatement, ParseError> {
        self.expect_keyword(Keyword::Insert)?;
        self.expect_keyword(Keyword::Into)?;

        let first = self.expect_identifier()?;
        let (schema, table) = if self.check(&TokenKind::Dot) {
            self.advance();
            let table_name = self.expect_identifier()?;
            (Some(first), table_name)
        } else {
            (None, first)
        };

        // Column list (optional)
        let columns = if self.check(&TokenKind::LeftParen) {
            self.advance();
            let cols = self.parse_identifier_list()?;
            self.expect(&TokenKind::RightParen)?;
            cols
        } else {
            vec![]
        };

        // VALUES, SELECT, or DEFAULT VALUES
        let values = if self.check_keyword(Keyword::Values) {
            self.advance();
            let mut rows = vec![];
            loop {
                self.expect(&TokenKind::LeftParen)?;
                let row = self.parse_expression_list()?;
                self.expect(&TokenKind::RightParen)?;
                rows.push(row);
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
            InsertSource::Values(rows)
        } else if self.check_keyword(Keyword::Select) {
            InsertSource::Query(Box::new(self.parse_select_statement()?))
        } else if self.check_keyword(Keyword::Default) {
            self.advance();
            self.expect_keyword(Keyword::Values)?;
            InsertSource::DefaultValues
        } else {
            return Err(ParseError::unexpected(
                "VALUES, SELECT, or DEFAULT VALUES",
                self.current.kind.clone(),
                self.current.span,
            ));
        };

        Ok(InsertStatement {
            schema,
            table,
            columns,
            values,
            on_conflict: None,
        })
    }

    /// Parses an UPDATE statement.
    fn parse_update_statement(&mut self) -> Result<UpdateStatement, ParseError> {
        self.expect_keyword(Keyword::Update)?;

        let first = self.expect_identifier()?;
        let (schema, table) = if self.check(&TokenKind::Dot) {
            self.advance();
            let table_name = self.expect_identifier()?;
            (Some(first), table_name)
        } else {
            (None, first)
        };

        let alias = self.parse_optional_alias()?;

        self.expect_keyword(Keyword::Set)?;

        // Parse SET assignments
        let mut assignments = vec![];
        loop {
            let column = self.expect_identifier()?;
            self.expect(&TokenKind::Eq)?;
            let value = self.parse_expression(0)?;
            assignments.push(UpdateAssignment { column, value });

            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        // FROM clause (optional, for joins)
        let from = if self.check_keyword(Keyword::From) {
            self.advance();
            Some(self.parse_table_ref()?)
        } else {
            None
        };

        // WHERE clause
        let where_clause = if self.check_keyword(Keyword::Where) {
            self.advance();
            Some(self.parse_expression(0)?)
        } else {
            None
        };

        Ok(UpdateStatement {
            schema,
            table,
            alias,
            assignments,
            from,
            where_clause,
        })
    }

    /// Parses a DELETE statement.
    fn parse_delete_statement(&mut self) -> Result<DeleteStatement, ParseError> {
        self.expect_keyword(Keyword::Delete)?;
        self.expect_keyword(Keyword::From)?;

        let first = self.expect_identifier()?;
        let (schema, table) = if self.check(&TokenKind::Dot) {
            self.advance();
            let table_name = self.expect_identifier()?;
            (Some(first), table_name)
        } else {
            (None, first)
        };

        let alias = self.parse_optional_alias()?;

        // WHERE clause
        let where_clause = if self.check_keyword(Keyword::Where) {
            self.advance();
            Some(self.parse_expression(0)?)
        } else {
            None
        };

        Ok(DeleteStatement {
            schema,
            table,
            alias,
            where_clause,
        })
    }

    /// Parses an ORDER BY list.
    fn parse_order_by_list(&mut self) -> Result<Vec<OrderBy>, ParseError> {
        let mut items = vec![];
        loop {
            let expr = self.parse_expression(0)?;
            let direction = if self.check_keyword(Keyword::Desc) {
                self.advance();
                OrderDirection::Desc
            } else if self.check_keyword(Keyword::Asc) {
                self.advance();
                OrderDirection::Asc
            } else {
                OrderDirection::Asc
            };

            items.push(OrderBy {
                expr,
                direction,
                nulls: None,
            });

            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        Ok(items)
    }

    /// Parses an expression using Pratt parsing.
    #[allow(clippy::while_let_loop)]
    fn parse_expression(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        // Parse prefix (primary expression or unary operator)
        let mut lhs = self.parse_prefix()?;

        // Parse infix operators
        loop {
            // Check if current token is an infix operator
            let (l_bp, r_bp) = match infix_binding_power(&self.current.kind) {
                Some(bp) => bp,
                None => break,
            };

            if l_bp < min_bp {
                break;
            }

            // Handle special infix operators
            match &self.current.kind {
                TokenKind::Keyword(Keyword::Is) => {
                    self.advance();
                    let negated = if self.check_keyword(Keyword::Not) {
                        self.advance();
                        true
                    } else {
                        false
                    };
                    self.expect_keyword(Keyword::Null)?;
                    lhs = Expr::IsNull {
                        expr: Box::new(lhs),
                        negated,
                    };
                }
                TokenKind::Keyword(Keyword::In) => {
                    self.advance();
                    self.expect(&TokenKind::LeftParen)?;
                    let list = self.parse_expression_list()?;
                    self.expect(&TokenKind::RightParen)?;
                    lhs = Expr::In {
                        expr: Box::new(lhs),
                        list,
                        negated: false,
                    };
                }
                TokenKind::Keyword(Keyword::Between) => {
                    self.advance();
                    let low = self.parse_expression(r_bp)?;
                    self.expect_keyword(Keyword::And)?;
                    let high = self.parse_expression(r_bp)?;
                    lhs = Expr::Between {
                        expr: Box::new(lhs),
                        low: Box::new(low),
                        high: Box::new(high),
                        negated: false,
                    };
                }
                _ => {
                    // Standard binary operator
                    if let Some(op) = token_to_binary_op(&self.current.kind) {
                        self.advance();
                        let rhs = self.parse_expression(r_bp)?;
                        lhs = Expr::Binary {
                            left: Box::new(lhs),
                            op,
                            right: Box::new(rhs),
                        };
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(lhs)
    }

    /// Parses a prefix expression.
    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        // Check for unary operators
        if let Some(op) = token_to_unary_op(&self.current.kind) {
            let bp = prefix_binding_power(&self.current.kind).unwrap_or(15);
            self.advance();
            let operand = self.parse_expression(bp)?;
            return Ok(Expr::Unary {
                op,
                operand: Box::new(operand),
            });
        }

        self.parse_primary()
    }

    /// Parses a primary expression.
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.current.clone();

        match &token.kind {
            // Literals
            TokenKind::Integer(n) => {
                self.advance();
                Ok(Expr::Literal(Literal::Integer(*n)))
            }
            TokenKind::Float(f) => {
                self.advance();
                Ok(Expr::Literal(Literal::Float(*f)))
            }
            TokenKind::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(Expr::Literal(Literal::String(value)))
            }
            TokenKind::Blob(b) => {
                let value = b.clone();
                self.advance();
                Ok(Expr::Literal(Literal::Blob(value)))
            }
            TokenKind::Keyword(Keyword::True) => {
                self.advance();
                Ok(Expr::Literal(Literal::Boolean(true)))
            }
            TokenKind::Keyword(Keyword::False) => {
                self.advance();
                Ok(Expr::Literal(Literal::Boolean(false)))
            }
            TokenKind::Keyword(Keyword::Null) => {
                self.advance();
                Ok(Expr::Literal(Literal::Null))
            }

            // Parameter placeholders
            TokenKind::Question => {
                self.param_counter += 1;
                let position = self.param_counter;
                self.advance();
                Ok(Expr::Parameter {
                    name: None,
                    position,
                })
            }
            TokenKind::Colon => {
                self.advance();
                let name = self.expect_identifier()?;
                Ok(Expr::Parameter {
                    name: Some(name),
                    position: 0,
                })
            }

            // Wildcard
            TokenKind::Star => {
                self.advance();
                Ok(Expr::Wildcard { table: None })
            }

            // Parenthesized expression or subquery
            TokenKind::LeftParen => {
                self.advance();
                if self.check_keyword(Keyword::Select) {
                    let subquery = self.parse_select_statement()?;
                    self.expect(&TokenKind::RightParen)?;
                    Ok(Expr::Subquery(Box::new(subquery)))
                } else {
                    let expr = self.parse_expression(0)?;
                    self.expect(&TokenKind::RightParen)?;
                    Ok(Expr::Paren(Box::new(expr)))
                }
            }

            // Aggregate functions
            TokenKind::Keyword(
                kw @ (Keyword::Count | Keyword::Sum | Keyword::Avg | Keyword::Min | Keyword::Max),
            ) => {
                let name = kw.as_str().to_string();
                self.advance();
                self.parse_function_call(name)
            }

            // Other functions
            TokenKind::Keyword(kw @ (Keyword::Coalesce | Keyword::Nullif | Keyword::Cast)) => {
                let name = kw.as_str().to_string();
                self.advance();
                if matches!(kw, Keyword::Cast) {
                    self.parse_cast_expression()
                } else {
                    self.parse_function_call(name)
                }
            }

            // CASE expression
            TokenKind::Keyword(Keyword::Case) => self.parse_case_expression(),

            // EXISTS
            TokenKind::Keyword(Keyword::Exists) => {
                self.advance();
                self.expect(&TokenKind::LeftParen)?;
                let subquery = self.parse_select_statement()?;
                self.expect(&TokenKind::RightParen)?;
                Ok(Expr::Function(FunctionCall {
                    name: String::from("EXISTS"),
                    args: vec![Expr::Subquery(Box::new(subquery))],
                    distinct: false,
                }))
            }

            // Identifier (column reference or function call)
            TokenKind::Identifier(name) => {
                let name = name.clone();
                let span = token.span;
                self.advance();

                // Check for function call
                if self.check(&TokenKind::LeftParen) {
                    return self.parse_function_call(name);
                }

                // Check for qualified name (table.column or table.*)
                if self.check(&TokenKind::Dot) {
                    self.advance();
                    if self.check(&TokenKind::Star) {
                        self.advance();
                        return Ok(Expr::Wildcard { table: Some(name) });
                    }
                    let column = self.expect_identifier()?;
                    return Ok(Expr::Column {
                        table: Some(name),
                        name: column,
                        span,
                    });
                }

                Ok(Expr::Column {
                    table: None,
                    name,
                    span,
                })
            }

            _ => Err(ParseError::unexpected(
                "expression",
                self.current.kind.clone(),
                self.current.span,
            )),
        }
    }

    /// Parses a function call.
    fn parse_function_call(&mut self, name: String) -> Result<Expr, ParseError> {
        self.expect(&TokenKind::LeftParen)?;

        let distinct = if self.check_keyword(Keyword::Distinct) {
            self.advance();
            true
        } else {
            false
        };

        let args = if self.check(&TokenKind::RightParen) {
            vec![]
        } else if self.check(&TokenKind::Star) {
            self.advance();
            vec![Expr::Wildcard { table: None }]
        } else {
            self.parse_expression_list()?
        };

        self.expect(&TokenKind::RightParen)?;

        Ok(Expr::Function(FunctionCall {
            name,
            args,
            distinct,
        }))
    }

    /// Parses a CAST expression.
    fn parse_cast_expression(&mut self) -> Result<Expr, ParseError> {
        self.expect(&TokenKind::LeftParen)?;
        let expr = self.parse_expression(0)?;
        self.expect_keyword(Keyword::As)?;
        let data_type = self.parse_data_type()?;
        self.expect(&TokenKind::RightParen)?;

        Ok(Expr::Cast {
            expr: Box::new(expr),
            data_type,
        })
    }

    /// Parses a CASE expression.
    fn parse_case_expression(&mut self) -> Result<Expr, ParseError> {
        self.expect_keyword(Keyword::Case)?;

        // Check for simple CASE (CASE expr WHEN ...)
        let operand = if !self.check_keyword(Keyword::When) {
            Some(Box::new(self.parse_expression(0)?))
        } else {
            None
        };

        // Parse WHEN/THEN clauses
        let mut when_clauses = vec![];
        while self.check_keyword(Keyword::When) {
            self.advance();
            let when_expr = self.parse_expression(0)?;
            self.expect_keyword(Keyword::Then)?;
            let then_expr = self.parse_expression(0)?;
            when_clauses.push((when_expr, then_expr));
        }

        // Parse ELSE clause
        let else_clause = if self.check_keyword(Keyword::Else) {
            self.advance();
            Some(Box::new(self.parse_expression(0)?))
        } else {
            None
        };

        self.expect_keyword(Keyword::End)?;

        Ok(Expr::Case {
            operand,
            when_clauses,
            else_clause,
        })
    }

    /// Parses a data type.
    fn parse_data_type(&mut self) -> Result<DataType, ParseError> {
        let data_type = match &self.current.kind {
            TokenKind::Keyword(Keyword::Int | Keyword::Integer) => {
                self.advance();
                DataType::Integer
            }
            TokenKind::Keyword(Keyword::Smallint) => {
                self.advance();
                DataType::Smallint
            }
            TokenKind::Keyword(Keyword::Bigint) => {
                self.advance();
                DataType::Bigint
            }
            TokenKind::Keyword(Keyword::Real) => {
                self.advance();
                DataType::Real
            }
            TokenKind::Keyword(Keyword::Double) => {
                self.advance();
                DataType::Double
            }
            TokenKind::Keyword(Keyword::Float) => {
                self.advance();
                DataType::Double
            }
            TokenKind::Keyword(Keyword::Decimal) => {
                self.advance();
                let (precision, scale) = self.parse_optional_precision_scale()?;
                DataType::Decimal { precision, scale }
            }
            TokenKind::Keyword(Keyword::Numeric) => {
                self.advance();
                let (precision, scale) = self.parse_optional_precision_scale()?;
                DataType::Numeric { precision, scale }
            }
            TokenKind::Keyword(Keyword::Char) => {
                self.advance();
                let len = self.parse_optional_length()?;
                DataType::Char(len)
            }
            TokenKind::Keyword(Keyword::Varchar) => {
                self.advance();
                let len = self.parse_optional_length()?;
                DataType::Varchar(len)
            }
            TokenKind::Keyword(Keyword::Text) => {
                self.advance();
                DataType::Text
            }
            TokenKind::Keyword(Keyword::Blob) => {
                self.advance();
                DataType::Blob
            }
            TokenKind::Keyword(Keyword::Boolean) => {
                self.advance();
                DataType::Boolean
            }
            TokenKind::Keyword(Keyword::Date) => {
                self.advance();
                DataType::Date
            }
            TokenKind::Keyword(Keyword::Time) => {
                self.advance();
                DataType::Time
            }
            TokenKind::Keyword(Keyword::Timestamp) => {
                self.advance();
                DataType::Timestamp
            }
            TokenKind::Keyword(Keyword::Datetime) => {
                self.advance();
                DataType::Datetime
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                DataType::Custom(name)
            }
            _ => {
                return Err(ParseError::unexpected(
                    "data type",
                    self.current.kind.clone(),
                    self.current.span,
                ));
            }
        };

        Ok(data_type)
    }

    /// Parses optional precision and scale (for DECIMAL/NUMERIC).
    fn parse_optional_precision_scale(&mut self) -> Result<(Option<u16>, Option<u16>), ParseError> {
        if !self.check(&TokenKind::LeftParen) {
            return Ok((None, None));
        }
        self.advance();

        let precision = match &self.current.kind {
            TokenKind::Integer(n) => {
                let p = u16::try_from(*n)
                    .map_err(|_| ParseError::new("Precision too large", self.current.span))?;
                self.advance();
                Some(p)
            }
            _ => {
                return Err(ParseError::unexpected(
                    "integer",
                    self.current.kind.clone(),
                    self.current.span,
                ));
            }
        };

        let scale = if self.check(&TokenKind::Comma) {
            self.advance();
            match &self.current.kind {
                TokenKind::Integer(n) => {
                    let s = u16::try_from(*n)
                        .map_err(|_| ParseError::new("Scale too large", self.current.span))?;
                    self.advance();
                    Some(s)
                }
                _ => {
                    return Err(ParseError::unexpected(
                        "integer",
                        self.current.kind.clone(),
                        self.current.span,
                    ));
                }
            }
        } else {
            None
        };

        self.expect(&TokenKind::RightParen)?;
        Ok((precision, scale))
    }

    /// Parses optional length (for CHAR/VARCHAR).
    fn parse_optional_length(&mut self) -> Result<Option<u32>, ParseError> {
        if !self.check(&TokenKind::LeftParen) {
            return Ok(None);
        }
        self.advance();

        let length = match &self.current.kind {
            TokenKind::Integer(n) => {
                let len = u32::try_from(*n)
                    .map_err(|_| ParseError::new("Length too large", self.current.span))?;
                self.advance();
                len
            }
            _ => {
                return Err(ParseError::unexpected(
                    "integer",
                    self.current.kind.clone(),
                    self.current.span,
                ));
            }
        };

        self.expect(&TokenKind::RightParen)?;
        Ok(Some(length))
    }

    /// Parses a comma-separated list of expressions.
    fn parse_expression_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut exprs = vec![];
        loop {
            exprs.push(self.parse_expression(0)?);
            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        Ok(exprs)
    }

    /// Parses a comma-separated list of identifiers.
    fn parse_identifier_list(&mut self) -> Result<Vec<String>, ParseError> {
        let mut idents = vec![];
        loop {
            idents.push(self.expect_identifier()?);
            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        Ok(idents)
    }

    // --- Helper methods ---

    /// Advances to the next token.
    fn advance(&mut self) {
        self.previous = core::mem::replace(&mut self.current, self.lexer.next_token());
    }

    /// Checks if the current token matches the given kind.
    fn check(&self, kind: &TokenKind) -> bool {
        core::mem::discriminant(&self.current.kind) == core::mem::discriminant(kind)
    }

    /// Checks if the current token is the given keyword.
    fn check_keyword(&self, keyword: Keyword) -> bool {
        matches!(&self.current.kind, TokenKind::Keyword(kw) if *kw == keyword)
    }

    /// Expects the current token to be the given kind.
    fn expect(&mut self, kind: &TokenKind) -> Result<(), ParseError> {
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::unexpected(
                format!("{kind:?}"),
                self.current.kind.clone(),
                self.current.span,
            ))
        }
    }

    /// Expects the current token to be the given keyword.
    fn expect_keyword(&mut self, keyword: Keyword) -> Result<(), ParseError> {
        if self.check_keyword(keyword) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::unexpected(
                keyword.as_str(),
                self.current.kind.clone(),
                self.current.span,
            ))
        }
    }

    /// Expects and returns an identifier.
    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        match &self.current.kind {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(ParseError::unexpected(
                "identifier",
                self.current.kind.clone(),
                self.current.span,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::BinaryOp;

    fn parse(sql: &str) -> Result<Statement, ParseError> {
        Parser::new(sql).parse_statement()
    }

    #[test]
    fn test_simple_select() {
        let stmt = parse("SELECT id, name FROM users").unwrap();
        assert!(matches!(stmt, Statement::Select(_)));
    }

    #[test]
    fn test_select_with_where() {
        let stmt = parse("SELECT * FROM users WHERE id = 1").unwrap();
        if let Statement::Select(select) = stmt {
            assert!(select.where_clause.is_some());
        } else {
            panic!("Expected SELECT statement");
        }
    }

    #[test]
    fn test_select_with_join() {
        let stmt =
            parse("SELECT u.id, o.amount FROM users u JOIN orders o ON u.id = o.user_id").unwrap();
        assert!(matches!(stmt, Statement::Select(_)));
    }

    #[test]
    fn test_expression_precedence() {
        // 1 + 2 * 3 should be parsed as 1 + (2 * 3)
        let stmt = parse("SELECT 1 + 2 * 3").unwrap();
        if let Statement::Select(select) = stmt {
            if let Expr::Binary { op, right, .. } = &select.columns[0].expr {
                assert_eq!(*op, BinaryOp::Add);
                assert!(matches!(
                    right.as_ref(),
                    Expr::Binary {
                        op: BinaryOp::Mul,
                        ..
                    }
                ));
            } else {
                panic!("Expected binary expression");
            }
        } else {
            panic!("Expected SELECT statement");
        }
    }

    #[test]
    fn test_insert_values() {
        let stmt =
            parse("INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com')").unwrap();
        if let Statement::Insert(insert) = stmt {
            assert_eq!(insert.table, "users");
            assert_eq!(insert.columns.len(), 2);
            assert!(matches!(insert.values, InsertSource::Values(_)));
        } else {
            panic!("Expected INSERT statement");
        }
    }

    #[test]
    fn test_update() {
        let stmt = parse("UPDATE users SET name = 'Bob' WHERE id = 1").unwrap();
        if let Statement::Update(update) = stmt {
            assert_eq!(update.table, "users");
            assert_eq!(update.assignments.len(), 1);
            assert!(update.where_clause.is_some());
        } else {
            panic!("Expected UPDATE statement");
        }
    }

    #[test]
    fn test_delete() {
        let stmt = parse("DELETE FROM users WHERE id = 1").unwrap();
        if let Statement::Delete(delete) = stmt {
            assert_eq!(delete.table, "users");
            assert!(delete.where_clause.is_some());
        } else {
            panic!("Expected DELETE statement");
        }
    }

    #[test]
    fn test_parameter_placeholders() {
        let stmt = parse("SELECT * FROM users WHERE id = ? AND name = :name").unwrap();
        let Statement::Select(select) = stmt else {
            panic!("Expected SELECT statement");
        };
        let Some(Expr::Binary { left, right, .. }) = &select.where_clause else {
            panic!("Expected Binary expression in WHERE clause");
        };
        // First condition: id = ?
        if let Expr::Binary { right: param1, .. } = left.as_ref() {
            assert!(matches!(
                param1.as_ref(),
                Expr::Parameter {
                    name: None,
                    position: 1
                }
            ));
        }
        // Second condition: name = :name
        if let Expr::Binary { right: param2, .. } = right.as_ref() {
            assert!(matches!(
                param2.as_ref(),
                Expr::Parameter { name: Some(n), .. } if n == "name"
            ));
        }
    }

    #[test]
    fn test_case_expression() {
        let stmt =
            parse("SELECT CASE WHEN status = 1 THEN 'active' ELSE 'inactive' END FROM users")
                .unwrap();
        if let Statement::Select(select) = stmt {
            assert!(matches!(select.columns[0].expr, Expr::Case { .. }));
        }
    }

    #[test]
    fn test_aggregate_functions() {
        let stmt = parse("SELECT COUNT(*), SUM(amount), AVG(price) FROM orders").unwrap();
        if let Statement::Select(select) = stmt {
            assert_eq!(select.columns.len(), 3);
            assert!(matches!(select.columns[0].expr, Expr::Function(_)));
        }
    }
}
