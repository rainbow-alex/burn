use std::vec::Vec;
use vm::analysis::{FrameAnalysis, VariableAnalysis};
use mem::raw::Raw;
use lang::identifier::Identifier;

pub struct Script {
	pub root: Root,
}

pub struct Repl {
	pub root: Root,
}

pub struct Root {
	pub statements: Vec<Box<Statement>>,
	pub frame: FrameAnalysis,
}

// STATEMENTS //////////////////////////////////////////////////////////////////////////////////////

pub enum Statement {
	
	ExpressionStatement {
		pub expression: Box<Expression>,
	},
	
	Assignment {
		pub lvalue: Box<Lvalue>,
		pub rvalue: Box<Expression>,
	},
	
	Import {
		pub path: Vec<Identifier>,
	},
	
	Let {
		pub variable_name: Identifier,
		pub variable: Raw<VariableAnalysis>,
		pub default: Option<Box<Expression>>,
		pub source_offset: uint,
	},
	
	Print {
		pub expression: Box<Expression>,
	},
	
	Return {
		pub expression: Option<Box<Expression>>,
	},
	
	Throw {
		pub expression: Box<Expression>,
	},
	
	If {
		pub test: Box<Expression>,
		pub block: Vec<Box<Statement>>,
		pub else_if_clauses: Vec<Box<ElseIf>>,
		pub else_clause: Option<Box<Else>>,
	},
	
	Try {
		pub block: Vec<Box<Statement>>,
		pub catch_clauses: Vec<Box<Catch>>,
		pub else_clause: Option<Box<Else>>,
		pub finally_clause: Option<Box<Finally>>,
	},
	
	While {
		pub test: Box<Expression>,
		pub block: Vec<Box<Statement>>,
		pub else_clause: Option<Box<Else>>,
	},
}

pub struct ElseIf {
	pub test: Box<Expression>,
	pub block: Vec<Box<Statement>>,
}

pub struct Else {
	pub block: Vec<Box<Statement>>,
}

pub struct Catch {
	pub type_: Option<Box<Expression>>,
	pub variable_name: Identifier,
	pub variable: Raw<VariableAnalysis>,
	pub block: Vec<Box<Statement>>,
}

pub struct Finally {
	pub block: Vec<Box<Statement>>,
}

// EXPRESSIONS /////////////////////////////////////////////////////////////////////////////////////

pub enum Expression {
	
	Function {
		pub parameters: Vec<FunctionParameter>,
		pub frame: FrameAnalysis,
		pub block: Vec<Box<Statement>>,
	},
	
	And {
		pub left: Box<Expression>,
		pub right: Box<Expression>,
	},
	Or {
		pub left: Box<Expression>,
		pub right: Box<Expression>,
	},
	Not {
		pub expression: Box<Expression>,
	},
	
	Is {
		pub left: Box<Expression>,
		pub right: Box<Expression>,
	},
	Eq {
		pub left: Box<Expression>,
		pub right: Box<Expression>,
	},
	Neq {
		pub left: Box<Expression>,
		pub right: Box<Expression>,
	},
	Lt {
		pub left: Box<Expression>,
		pub right: Box<Expression>,
	},
	Gt {
		pub left: Box<Expression>,
		pub right: Box<Expression>,
	},
	LtEq {
		pub left: Box<Expression>,
		pub right: Box<Expression>,
	},
	GtEq {
		pub left: Box<Expression>,
		pub right: Box<Expression>,
	},
	
	Union {
		pub left: Box<Expression>,
		pub right: Box<Expression>,
	},
	
	Addition {
		pub left: Box<Expression>,
		pub right: Box<Expression>,
	},
	Subtraction {
		pub left: Box<Expression>,
		pub right: Box<Expression>,
	},
	Multiplication {
		pub left: Box<Expression>,
		pub right: Box<Expression>,
	},
	Division {
		pub left: Box<Expression>,
		pub right: Box<Expression>,
	},
	
	DotAccess {
		pub expression: Box<Expression>,
		pub name: Identifier,
	},
	ItemAccess {
		pub expression: Box<Expression>,
		pub key_expression: Box<Expression>,
	},
	Call {
		pub expression: Box<Expression>,
		pub arguments: Vec<Box<Expression>>,
	},
	
	Variable {
		pub name: Identifier,
		pub analysis: Raw<VariableAnalysis>,
		pub source_offset: uint,
	},
	Name {
		pub identifier: Identifier,
	},
	
	String {
		pub value: ::std::string::String,
	},
	Integer {
		pub value: i64,
	},
	Float {
		pub value: f64,
	},
	Boolean {
		pub value: bool,
	},
	Nothing,
}

pub struct FunctionParameter {
	pub type_: Option<Box<Expression>>,
	pub default: Option<Box<Expression>>,
	pub variable_name: Identifier,
	pub variable: Raw<VariableAnalysis>,
}

// LVALUES /////////////////////////////////////////////////////////////////////////////////////////

pub enum Lvalue {
	
	VariableLvalue {
		pub name: Identifier,
		pub analysis: Raw<VariableAnalysis>,
	}
}
