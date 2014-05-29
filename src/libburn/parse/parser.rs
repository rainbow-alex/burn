use std::vec::Vec;
use vm::error::ParseError;
use parse::token;
use parse::lexer::Lexer;
use parse::node;
use parse::literal;
use vm::analysis::annotation;
use mem::raw::Raw;
use lang::identifier::Identifier;

type ParseResult<T> = Result<T,ParseError>;

pub fn parse_script<'src>( source: &'src str ) -> Result<Box<node::Script>,ParseError> {
	
	let mut parsing = Parsing {
		lexer: Lexer::new( source ),
		buffer: Vec::new(),
		newline_policy: HeedNewlines,
	};
	
	match parsing.parse_root() {
		Ok( root ) => Ok( box node::Script { root: root } ),
		Err( e ) => Err( e ),
	}
}

pub fn parse_repl( source: &str ) -> Result<Box<node::Repl>,ParseError>{
	
	let mut parsing = Parsing {
		lexer: Lexer::new( source ),
		buffer: Vec::new(),
		newline_policy: HeedNewlines,
	};
	
	match parsing.parse_root() {
		Ok( root ) => Ok( box node::Repl { root: root } ),
		Err( e ) => Err( e ),
	}
}

struct Parsing<'src> {
	lexer: Lexer<'src>,
	buffer: Vec<token::Token<'src>>,
	newline_policy: NewlinePolicy,
}

	#[deriving(Eq)]
	enum NewlinePolicy {
		IgnoreNewlines,
		HeedNewlines,
	}

	macro_rules! unwrap_or_return_err(
		($expr:expr) => {
			match $expr {
				Ok( v ) => v,
				Err( e ) => { return Err( e ); }
			}
		}
	)

	macro_rules! return_err(
		($expr:expr) => {
			match $expr {
				Ok(..) => {},
				Err( e ) => { return Err( e ); }
			}
		}
	)
	
	type Precedence = u8;
	static PRECEDENCE_MULTIPLICATIVE: Precedence = 31;
	static PRECEDENCE_ADDITIVE: Precedence = 30;
	static PRECEDENCE_UNION: Precedence = 25;
	static PRECEDENCE_COMPARE: Precedence = 20;
	static PRECEDENCE_NOT: Precedence = 11;
	static PRECEDENCE_BIN_LOGIC: Precedence = 10;
	static PRECEDENCE_ANY: Precedence = 0;
	
	impl<'src> Parsing<'src> {
		
		//
		// helpers
		//
		
		fn fill_buffer( &mut self, length: uint ) {
			while self.buffer.len() < length {
				self.buffer.push( self.lexer.read() );
			}
		}
		
		fn peek_n( &mut self, mut offset: uint ) -> token::Token<'src> {
			
			if self.newline_policy == HeedNewlines {
				
				self.fill_buffer( offset + 1 );
				return *self.buffer.get( offset )
				
			} else {
				
				let mut i = 0;
				loop {
					self.fill_buffer( i + 1 );
					if *self.buffer.get( i ) != token::Newline {
						if offset > 0 {
							offset -= 1;
						} else {
							return *self.buffer.get( i );
						}
					}
					i += 1;
				}
			}
		}
		
		fn peek( &mut self ) -> token::Token<'src> {
			self.peek_n( 0 )
		}
		
		fn read( &mut self ) -> token::Token<'src> {
			if self.newline_policy == IgnoreNewlines {
				loop {
					self.fill_buffer(1);
					if *self.buffer.get(0) == token::Newline {
						self.buffer.shift().unwrap();
					} else {
						break;
					}
				}
			}
			self.fill_buffer(1);
			self.buffer.shift().unwrap()
		}
		
		fn err( &self, message: String ) -> ParseError {
			ParseError {
				source_offset: self.lexer.offset,
				message: message,
			}
		}
		
		fn skip_newlines( &mut self ) {
			while self.peek() == token::Newline {
				self.read();
			}
		}
		
		//
		// Parsing logic
		//
		
		fn parse_root( &mut self ) -> ParseResult<node::Root> {
			
			let mut statements = Vec::<Box<node::Statement>>::new();
			
			self.skip_newlines();
			
			loop {
				if self.peek() == token::Eof {
					break;
				}
				
				let statement = unwrap_or_return_err!( self.parse_statement() );
				statements.push( statement );
				
				match self.peek() {
					token::Newline => self.skip_newlines(),
					token::Eof => break,
					_ => return Err( self.err( "Expected newline.".into_owned() ) )
				}
			}
			
			Ok( node::Root {
				statements: statements,
				frame: annotation::Frame::new(),
			} )
		}
		
		fn parse_block( &mut self ) -> ParseResult<Vec<Box<node::Statement>>> {
			
			let mut statements = Vec::<Box<node::Statement>>::new();
			
			match self.peek() {
				token::LeftCurlyBracket => { self.read(); }
				_ => return Err( self.err( "Expected `{`." .into_owned() ) )
			}
			
			let old_newline_policy = self.newline_policy;
			self.newline_policy = HeedNewlines;
			self.skip_newlines();
			
			loop {
				if self.peek() == token::RightCurlyBracket {
					break;
				}
				
				let statement = unwrap_or_return_err!( self.parse_statement() );
				statements.push( statement );
				
				match self.peek() {
					token::Newline => self.skip_newlines(),
					token::RightCurlyBracket => break,
					_ => return Err( self.err( "Expected newline.".to_string() ) )
				}
			}
			
			let closing = self.read();
			assert!( closing == token::RightCurlyBracket );
			
			self.newline_policy = old_newline_policy;
			
			Ok( statements )
		}
		
		fn parse_statement( &mut self ) -> ParseResult<Box<node::Statement>> {
			match self.peek() {
				
				token::Use => self.parse_use_statement(),
				
				token::Let => self.parse_let_statement(),
				token::Print => self.parse_print_statement(),
				token::Throw => self.parse_throw_statement(),
				
				token::If => self.parse_if_statement(),
				token::While => self.parse_while_statement(),
				token::Try => self.parse_try_statement(),
				
				_ => {
					
					let expression = unwrap_or_return_err!( self.parse_expression() );
					
					if self.peek() == token::Equals {
						
						let lvalue = unwrap_or_return_err!( self.to_lvalue( expression ) );
						self.read();
						let rvalue = unwrap_or_return_err!( self.parse_expression() );
						
						Ok( box node::Assignment {
							lvalue: lvalue,
							rvalue: rvalue,
						} )
						
					} else {
						
						Ok( box node::ExpressionStatement {
							expression: expression,
						} )
					}
				}
			}
		}
		
		fn parse_use_statement( &mut self ) -> ParseResult<Box<node::Statement>> {
			
			let keyword = self.read();
			assert!( keyword == token::Use );
			
			let path = unwrap_or_return_err!( self.parse_path() );
			let name = *path.last().unwrap();
			
			Ok( box node::Use {
				path: path,
				annotation: annotation::Use::new( name ),
			} )
		}
		
		fn parse_path( &mut self ) -> ParseResult<Vec<Identifier>> {
			
			let mut path = Vec::new();
			
			loop {
				
				match self.peek() {
					token::Identifier ( identifier ) => {
						path.push( Identifier::find_or_create( identifier ) );
						self.peek();
					}
					_ => {
						fail!();
					}
				}
				
				self.read();
				
				match self.peek() {
					token::Dot => {
						self.read();
					}
					_ => {
						break;
					}
				}
			}
			
			Ok( path )
		}
		
		fn parse_if_statement( &mut self ) -> ParseResult<Box<node::Statement>> {
			
			let keyword = self.read();
			assert!( keyword == token::If );
			
			let previous_newline_policy = self.newline_policy;
			self.newline_policy = IgnoreNewlines;
			
			let if_test = unwrap_or_return_err!( self.parse_expression() );
			let if_block = unwrap_or_return_err!( self.parse_block() );
			
			let mut else_if_clauses = Vec::new();
			let mut else_clause = None;
			
			while self.peek() == token::Else {
				
				self.read();
				
				if self.peek() == token::If {
					
					self.read();
					
					let else_if_test = unwrap_or_return_err!( self.parse_expression() );
					let else_if_block = unwrap_or_return_err!( self.parse_block() );
					else_if_clauses.push( box node::ElseIf {
						test: else_if_test,
						block: else_if_block,
					} );
					
				} else {
					
					let else_block = unwrap_or_return_err!( self.parse_block() );
					else_clause = Some( box node::Else {
						block: else_block,
					} );
					
					break;
				}
			}
			
			self.newline_policy = previous_newline_policy;
			
			Ok( box node::If {
				test: if_test,
				block: if_block,
				else_if_clauses: else_if_clauses,
				else_clause: else_clause,
			} )
		}
		
		fn parse_while_statement( &mut self ) -> ParseResult<Box<node::Statement>> {
			
			let keyword = self.read();
			assert!( keyword == token::While );
			
			let previous_newline_policy = self.newline_policy;
			self.newline_policy = IgnoreNewlines;
			
			let test = unwrap_or_return_err!( self.parse_expression() );
			let while_block = unwrap_or_return_err!( self.parse_block() );
			
			let mut else_clause = None;
			
			if self.peek() == token::Else {
				self.read();
				let else_block = unwrap_or_return_err!( self.parse_block() );
				else_clause = Some( box node::Else {
					block: else_block,
				} );
			}
			
			self.newline_policy = previous_newline_policy;
			
			Ok( box node::While {
				test: test,
				block: while_block,
				else_clause: else_clause,
			} )
		}
		
		fn parse_try_statement( &mut self ) -> ParseResult<Box<node::Statement>> {
			
			let keyword = self.read();
			assert!( keyword == token::Try );
			
			let previous_newline_policy = self.newline_policy;
			self.newline_policy = IgnoreNewlines;
			
			let try_block = unwrap_or_return_err!( self.parse_block() );
			
			let mut catch_clauses = Vec::<Box<node::Catch>>::new();
			
			while self.peek() == token::Catch {
				
				self.read();
				
				let type_ = match self.peek() {
					token::Variable(..) => match self.peek_n(1) {
						token::LeftCurlyBracket => None,
						_ => Some( unwrap_or_return_err!( self.parse_expression() ) ),
					},
					_ => Some( unwrap_or_return_err!( self.parse_expression() ) ),
				};
				
				let variable_name = match self.peek() {
					token::Variable( name ) => {
						self.read();
						Identifier::find_or_create( name )
					}
					_ => return Err( self.err( "Expected variable".to_string() ) )
				};
				
				let block = unwrap_or_return_err!( self.parse_block() );
				
				catch_clauses.push( box node::Catch {
					type_: type_,
					variable_name: variable_name,
					variable: Raw::null(),
					block: block,
				} );
			}
			
			let mut else_clause = None;
			
			if self.peek() == token::Else {
				
				self.read();
				
				let else_block = unwrap_or_return_err!( self.parse_block() );
				else_clause = Some( box node::Else {
					block: else_block,
				} );
			}
			
			let mut finally_clause = None;
			
			if self.peek() == token::Finally {
				
				self.read();
				
				let finally_block = unwrap_or_return_err!( self.parse_block() );
				finally_clause = Some( box node::Finally {
					block: finally_block,
				} );
			}
			
			self.newline_policy = previous_newline_policy;
			
			Ok( box node::Try {
				block: try_block,
				catch_clauses: catch_clauses,
				else_clause: else_clause,
				finally_clause: finally_clause,
			} )
		}
		
		fn parse_let_statement( &mut self ) -> ParseResult<Box<node::Statement>> {
			
			let source_offset = self.lexer.offset;
			
			let keyword = self.read();
			assert!( keyword == token::Let );
			
			let variable_name = match self.peek() {
				token::Variable( name ) => {
					self.read();
					Identifier::find_or_create( name )
				}
				_ => return Err( self.err( "Expected variable".to_string() ) )
			};
			
			let default = if self.peek() == token::Equals {
				self.read();
				Some( unwrap_or_return_err!( self.parse_expression() ) )
			} else {
				None
			};
			
			Ok( box node::Let {
				variable_name: variable_name,
				annotation: Raw::null(),
				default: default,
				source_offset: source_offset,
			} )
		}
		
		fn parse_print_statement( &mut self ) -> ParseResult<Box<node::Statement>> {
			
			let keyword = self.read();
			assert!( keyword == token::Print );
			
			let expression = unwrap_or_return_err!( self.parse_expression() );
			
			Ok( box node::Print {
				expression: expression,
			} )
		}
		
		fn parse_throw_statement( &mut self ) -> ParseResult<Box<node::Statement>> {
			
			let keyword = self.read();
			assert!( keyword == token::Throw );
			
			let expression = unwrap_or_return_err!( self.parse_expression() );
			
			Ok( box node::Throw {
				expression: expression,
			} )
		}
		
		fn parse_expression( &mut self ) -> ParseResult<Box<node::Expression>> {
			self.parse_op_expression( PRECEDENCE_ANY )
		}
		
		/// Parse binary and unary expressions.
		///
		/// The algorithm used is called "precedence climbing".
		/// Basically you keep greedily matching lower precedence operators. For every operator found,
		/// the right-hand side subexpression is parsed by recursing (with a limit on how low the precedence can then go).
		/// See http://en.wikipedia.org/wiki/Operator-precedence_parser
		/// Note that in this implementation the outer loop has been unrolled.
		fn parse_op_expression( &mut self, min_precedence: Precedence ) -> ParseResult<Box<node::Expression>> {
			
			//
			// Unary
			//
			
			if min_precedence <= PRECEDENCE_NOT && self.peek() == token::Not {
				self.read();
				let expression = unwrap_or_return_err!( self.parse_op_expression( PRECEDENCE_NOT + 1 ) );
				return Ok( box node::Not { expression: expression } );
			}
			
			//
			// Binary
			//
			
			let mut left = unwrap_or_return_err!( self.parse_access_expression() );
			
			if min_precedence > PRECEDENCE_MULTIPLICATIVE {
				return Ok( left );
			}
			
			loop {
				if self.peek() == token::Asterisk {
					self.read();
					self.skip_newlines();
					let right = unwrap_or_return_err!( self.parse_op_expression( PRECEDENCE_MULTIPLICATIVE + 1 ) );
					left = box node::Multiplication { left: left, right: right };
				} else if self.peek() == token::Slash {
					self.read();
					self.skip_newlines();
					let right = unwrap_or_return_err!( self.parse_op_expression( PRECEDENCE_MULTIPLICATIVE + 1 ) );
					left = box node::Division { left: left, right: right };
				} else {
					break;
				}
			}
			
			if min_precedence > PRECEDENCE_ADDITIVE {
				return Ok( left );
			}
			
			loop {
				if self.peek() == token::Plus {
					self.read();
					self.skip_newlines();
					let right = unwrap_or_return_err!( self.parse_op_expression( PRECEDENCE_ADDITIVE + 1 ) );
					left = box node::Addition { left: left, right: right };
				} else if self.peek() == token::Dash {
					self.read();
					self.skip_newlines();
					let right = unwrap_or_return_err!( self.parse_op_expression( PRECEDENCE_ADDITIVE + 1 ) );
					left = box node::Subtraction { left: left, right: right };
				} else {
					break;
				}
			}
			
			if min_precedence > PRECEDENCE_UNION {
				return Ok( left );
			}
			
			while self.peek() == token::VerticalBar {
				self.read();
				self.skip_newlines();
				let right = unwrap_or_return_err!( self.parse_op_expression( PRECEDENCE_UNION + 1 ) );
				left = box node::Union { left: left, right: right };
			}
			
			if min_precedence > PRECEDENCE_COMPARE {
				return Ok( left );
			}
			
			match self.peek() {
				
				token::Is => {
					self.read();
					self.skip_newlines();
					let right = unwrap_or_return_err!( self.parse_op_expression( PRECEDENCE_COMPARE + 1 ) );
					left = box node::Is { left: left, right: right };
				}
				
				token::EqualsEquals => {
					self.read();
					self.skip_newlines();
					let right = unwrap_or_return_err!( self.parse_op_expression( PRECEDENCE_COMPARE + 1 ) );
					left = box node::Eq { left: left, right: right };
				}
				
				token::BangEquals => {
					self.read();
					self.skip_newlines();
					let right = unwrap_or_return_err!( self.parse_op_expression( PRECEDENCE_COMPARE + 1 ) );
					left = box node::Neq { left: left, right: right };
				}
				
				token::LeftAngleBracket => {
					self.read();
					self.skip_newlines();
					let right = unwrap_or_return_err!( self.parse_op_expression( PRECEDENCE_COMPARE + 1 ) );
					left = box node::Lt { left: left, right: right };
				}
				
				token::RightAngleBracket => {
					self.read();
					self.skip_newlines();
					let right = unwrap_or_return_err!( self.parse_op_expression( PRECEDENCE_COMPARE + 1 ) );
					left = box node::Gt { left: left, right: right };
				}
				
				token::LeftAngleBracketEquals => {
					self.read();
					self.skip_newlines();
					let right = unwrap_or_return_err!( self.parse_op_expression( PRECEDENCE_COMPARE + 1 ) );
					left = box node::LtEq { left: left, right: right };
				}
				
				token::RightAngleBracketEquals => {
					self.read();
					self.skip_newlines();
					let right = unwrap_or_return_err!( self.parse_op_expression( PRECEDENCE_COMPARE + 1 ) );
					left = box node::GtEq { left: left, right: right };
				}
				
				_ => {}
			}
			
			if min_precedence > PRECEDENCE_BIN_LOGIC {
				return Ok( left );
			}
			
			if self.peek() == token::And {
				while self.peek() == token::And {
					self.read();
					self.skip_newlines();
					let right = unwrap_or_return_err!( self.parse_op_expression( PRECEDENCE_BIN_LOGIC + 1 ) );
					left = box node::And { left: left, right: right };
				}
			} else {
				while self.peek() == token::Or {
					self.read();
					self.skip_newlines();
					let right = unwrap_or_return_err!( self.parse_op_expression( PRECEDENCE_BIN_LOGIC + 1 ) );
					left = box node::Or { left: left, right: right };
				}
			}
			
			Ok( left )
		}
		
		fn parse_access_expression( &mut self ) -> ParseResult<Box<node::Expression>> {
			
			let mut expression = unwrap_or_return_err!( self.parse_atom_expression() );
			
			loop {
				match self.peek() {
					
					token::Dot => {
						self.read();
						
						let name = match self.peek() {
							token::Identifier( identifier ) => {
								self.read();
								identifier
							}
							_ => { fail!(); } // TODO
						};
						
						expression = box node::DotAccess {
							expression: expression,
							name: Identifier::find_or_create( name ),
						};
					}
					
					token::LeftParenthesis => {
						self.read();
						self.read(); //TODO!
						expression = box node::Call {
							expression: expression,
							arguments: Vec::new(),
						};
					}
					
					_ => break
				}
			}
			
			Ok( expression )
		}
		
		fn parse_atom_expression( &mut self ) -> ParseResult<Box<node::Expression>> {
			
			match self.peek() {
				
				token::Function => self.parse_function(),
				
				token::LeftParenthesis => {
					
					let left = self.read();
					assert!( left == token::LeftParenthesis );
					
					let old_newline_policy = self.newline_policy;
					self.newline_policy = IgnoreNewlines;
					
					let expr = unwrap_or_return_err!( self.parse_expression() );
					
					if self.peek() != token::RightParenthesis {
						return Err( self.err( format!( "Expected {}.", token::RightParenthesis ) ) );
					}
					self.read(); // )
					
					self.newline_policy = old_newline_policy;
					
					Ok( expr )
				}
				
				token::Identifier( identifier ) => {
					self.read();
					Ok( box node::Name {
						identifier: Identifier::find_or_create( identifier ),
						annotation: annotation::Name::new(),
					} )
				}
				token::Variable( name ) => {
					let source_offset = self.lexer.offset;
					self.read();
					Ok( box node::Variable {
						name: Identifier::find_or_create( name ),
						source_offset: source_offset,
						annotation: Raw::null(),
					} )
				}
				
				token::String( source ) => {
					self.read();
					match literal::parse_string( source ) {
						Ok( value ) => Ok( box node::String { value: value } ),
						Err( (message, _) ) => Err( self.err( message ) ),
					}
				}
				token::Integer( source ) => {
					self.read();
					match literal::parse_int( source ) {
						Ok( value ) => Ok( box node::Integer { value: value } ),
						Err( e ) => Err( self.err( e ) ),
					}
				}
				token::Float( source ) => {
					self.read();
					match literal::parse_float( source ) {
						Ok( value ) => Ok( box node::Float { value: value } ),
						Err( e ) => Err( self.err( e ) ),
					}
				}
				token::True => {
					self.read();
					Ok( box node::Boolean { value: true } )
				}
				token::False => {
					self.read();
					Ok( box node::Boolean { value: false } )
				}
				token::Nothing => {
					self.read();
					Ok( box node::Nothing )
				}
				
				t @ _ => {
					Err( self.err( format!( "Unexpected {}.", t ) ) )
				}
			}
		}
		
		fn parse_function( &mut self ) -> ParseResult<Box<node::Expression>> {
			
			let keyword = self.read();
			assert!( keyword == token::Function );
			
			let previous_newline_policy = self.newline_policy;
			self.newline_policy = IgnoreNewlines;
			
			if self.peek() != token::LeftParenthesis {
				return Err( self.err( "Expected `(`".to_string() ) );
			}
			self.read();
			
			let parameters = unwrap_or_return_err!( self.parse_function_parameters() );
			
			if self.peek() != token::RightParenthesis {
				return Err( self.err( "Expected `)`".to_string() ) );
			}
			self.read();
			
			let block = unwrap_or_return_err!( self.parse_block() );
			
			self.newline_policy = previous_newline_policy;
			
			Ok( box node::Function {
				parameters: parameters,
				block: block,
				frame: annotation::Frame::new_with_closure(),
			} )
		}
		
		fn parse_function_parameters( &mut self ) -> ParseResult<Vec<node::FunctionParameter>> {
			
			let mut parameters = Vec::new();
			
			if self.peek() == token::RightParenthesis {
				return Ok( parameters );
			}
			
			loop {
				let type_ = match self.peek() {
					token::Variable(..) => match self.peek_n(1) {
						token::Equals | token::Comma => None,
						_ => Some( unwrap_or_return_err!( self.parse_expression() ) ),
					},
					_ => Some( unwrap_or_return_err!( self.parse_expression() ) ),
				};
				
				let variable_name = match self.peek() {
					token::Variable( name ) => {
						self.read();
						Identifier::find_or_create( name )
					}
					_ => return Err( self.err( "Expected variable".to_string() ) )
				};
				
				let default = if self.peek() == token::Equals {
					self.read();
					Some( unwrap_or_return_err!( self.parse_expression() ) )
				} else {
					None
				};
				
				parameters.push( node::FunctionParameter {
					type_: type_,
					default: default,
					variable_name: variable_name,
					variable: Raw::null(),
				} );
				
				match self.peek() {
					token::Comma => {
						self.read();
					}
					token::RightParenthesis => break,
					_ => return Err( self.err( "Expected `)` or `,`".to_string() ) )
				};
			}
			
			Ok( parameters )
		}
		
		fn to_lvalue( &self, expression: Box<node::Expression> ) -> ParseResult<Box<node::Lvalue>> {
			match *expression {
				
				node::Variable {
					name: name,
					annotation: annotation,
					source_offset: _,
				} => {
					Ok( box node::VariableLvalue {
						name: name,
						annotation: annotation,
					} )
				}
				
				node::DotAccess {
					expression: expression,
					name: name,
				} => {
					Ok( box node::DotAccessLvalue {
						expression: expression,
						name: name,
					} )
				}
				
				_ => Err( self.err( "Invalid lvalue".to_string() ) )
			}
		}
	}
