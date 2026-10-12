#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Statement {
    Assignment {
        name: Expression,
        value: Expression,
    },

    Expression(Expression),

    If {
        condition: Expression,
        body: Vec<Statement>,
    },

    While {
        condition: Expression,
        body: Vec<Statement>,
    },

    Function {
        name: String,
        parameters: Vec<String>,
        body: Vec<Statement>,
    },

    Class {
        name: String,
        body: Vec<Statement>,
    },

    Return(Expression),
    Pass,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expression {
    None,

    Integer(i64),
    Str(String),
    List(Vec<Expression>),

    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },

    Variable(String),

    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },

    Attribute {
        object: Box<Expression>,
        name: String,
    },

    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,

    Greater,
    Less,
    Equal,
}
