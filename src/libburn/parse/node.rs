use std::vec::Vec;
use compile::analysis::{FrameAnalysis, ClosureAnalysis, ScopeAnalysis, VariableAnalysis};
use mem::raw::Raw;
use lang::identifier::Identifier;

#[cfg(test)]
use std::fmt;

pub struct Script {
	pub root: Root,
}

pub struct Repl {
	pub root: Root,
}

pub struct Root {
	pub statements: Vec<Box<Statement>>,
	pub frame: FrameAnalysis,
	pub scope: ScopeAnalysis,
}

// STATEMENTS //////////////////////////////////////////////////////////////////////////////////////

pub enum Statement {
	
	If {
		pub test: Box<Expression>,
		pub scope: ScopeAnalysis,
		pub block: Vec<Box<Statement>>,
		pub else_if_clauses: Vec<Box<ElseIf>>,
		pub else_clause: Option<Box<Else>>,
	},
	
	Try {
		pub scope: ScopeAnalysis,
		pub block: Vec<Box<Statement>>,
		pub catch_clauses: Vec<Box<Catch>>,
		pub else_clause: Option<Box<Else>>,
		pub finally_clause: Option<Box<Finally>>,
	},
	
	While {
		pub test: Box<Expression>,
		pub scope: ScopeAnalysis,
		pub block: Vec<Box<Statement>>,
		pub else_clause: Option<Box<Else>>,
	},
	
	Let {
		pub variable: VariableAnalysis,
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
	
	Assignment {
		pub lvalue: Box<Lvalue>,
		pub rvalue: Box<Expression>,
	},
	
	ExpressionStatement {
		pub expression: Box<Expression>,
	},
}

pub struct ElseIf {
	pub test: Box<Expression>,
	pub scope: ScopeAnalysis,
	pub block: Vec<Box<Statement>>,
}

pub struct Else {
	pub scope: ScopeAnalysis,
	pub block: Vec<Box<Statement>>,
}

pub struct Catch {
	pub type_: Option<Box<Expression>>,
	pub scope: ScopeAnalysis,
	pub variable: VariableAnalysis,
	pub block: Vec<Box<Statement>>,
}

pub struct Finally {
	pub scope: ScopeAnalysis,
	pub block: Vec<Box<Statement>>,
}

// EXPRESSIONS /////////////////////////////////////////////////////////////////////////////////////

pub enum Expression {
	
	Function {
		pub parameters: Vec<FunctionParameter>,
		pub closure: ClosureAnalysis,
		pub scope: ScopeAnalysis,
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
		pub value: StrBuf,
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
	pub variable: VariableAnalysis,
}

// LVALUES /////////////////////////////////////////////////////////////////////////////////////////

pub enum Lvalue {
	
	VariableLvalue {
		pub name: Identifier,
		pub analysis: Raw<VariableAnalysis>,
	}
}

// PRINTING (test only) ////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
impl fmt::Show for Script {
	fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result {
		write!( f, "{}", self.statements )
	}
}

#[cfg(test)]
impl fmt::Show for Statement {
	fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result {
		match *self {
			
			Throw { expression: ref e } => write!( f, "throw({})", e ),
			ExpressionStatement { expression: ref e } => write!( f, "{}", e ),
			
			_ => write!( f, "<statement>" ),
		}
	}
}

#[cfg(test)]
impl fmt::Show for Expression {
	fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result {
		match *self {
			
			Function { parameters: ref p, block: ref b, .. } => write!( f, "function({})\\{{}\\}", p, b ),
			
			Addition { left: ref l, right: ref r } => write!( f, "({}+{})", l, r ),
			Multiplication { left: ref l, right: ref r } => write!( f, "({}*{})", l, r ),
			
			Integer { value: v } => write!( f, "{}", v ),
			Boolean { value: v } => write!( f, "{}", if v { "true" } else { "false" } ),
			Nothing => write!( f, "nothing" ),
			
			_ => write!( f, "<expression>" ),
		}
	}
}

#[cfg(test)]
impl fmt::Show for FunctionParameter {
	fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result {
		write!( f, "<parameter>" )
	}
}
