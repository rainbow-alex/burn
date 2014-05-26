use std::str::utf8_char_width;
use parse::token;

pub struct Lexer<'src> {
	source: &'src str,
	pub offset: uint,
}

impl<'src> Lexer<'src> {
	
	pub fn new( source: &'src str ) -> Lexer<'src> {
		Lexer {
			source: source,
			offset: 0,
		}
	}
	
	pub fn read( &mut self ) -> token::Token<'src> {
		loop {
			match self.peek_char( self.offset ) {
				None => {
					return token::Eof;
				},
				Some( ' ' ) | Some( '\t' ) => {
					self.offset += 1;
				},
				Some( '/' ) if self.peek_char( self.offset + 1 ) == Some( '*' ) => {
					self.offset += 2;
					
					let mut stars = 1;
					while self.peek_char( self.offset ) == Some( '*' ) {
						self.offset += 1;
						stars += 1;
					}
					
					let mut found_stars = 0;
					loop {
						match self.peek_char( self.offset ) {
							Some( '*' ) => {
								found_stars += 1;
								self.offset += 1;
							}
							Some( '/' ) if found_stars == stars => {
								self.offset += 1;
								break;
							}
							Some( _ ) => {
								found_stars = 0;
								self.offset += 1;
							}
							None => {
								break;
							}
						}
					}
				}
				Some( '/' ) if self.peek_char( self.offset + 1 ) == Some( '/' ) => {
					self.offset += 2;
					
					loop {
						match self.peek_char( self.offset ) {
							Some( '\n' ) => {
								self.offset += 1;
								break;
							}
							Some( _ ) => {
								self.offset += 1;
							}
							None => {
								break;
							}
						}
					}
				}
				Some( _ ) => {
					let (token, length) = self.match_token();
					self.offset += length;
					return token;
				}
			}
		}
	}
	
	fn peek_char( &self, offset: uint ) -> Option<char> {
		if offset < self.source.len() {
			Some( self.source[ offset ] as char )
		} else {
			None
		}
	}
	
	fn match_token( &self ) -> (token::Token<'src>, uint) {
		
		match self.source[ self.offset ] as char {
			
			'\n' => (token::Newline, 1),
			
			// symbols
			'{' => (token::LeftCurlyBracket, 1),
			'}' => (token::RightCurlyBracket, 1),
			'[' => (token::LeftSquareBracket, 1),
			']' => (token::RightSquareBracket, 1),
			'(' => (token::LeftParenthesis, 1),
			')' => (token::RightParenthesis, 1),
			'.' => (token::Dot, 1),
			',' => (token::Comma, 1),
			'=' => match self.peek_char( self.offset + 1 ) {
				Some( '=' ) => (token::EqualsEquals, 2),
				_ => (token::Equals, 1),
			},
			'<' => match self.peek_char( self.offset + 1 ) {
				Some( '=' ) => (token::LeftAngleBracketEquals, 2),
				_ => (token::LeftAngleBracket, 1),
			},
			'>' => match self.peek_char( self.offset + 1 ) {
				Some( '=' ) => (token::RightAngleBracketEquals, 2),
				_ => (token::RightAngleBracket, 1),
			},
			'!' => match self.peek_char( self.offset + 1 ) {
				Some( '=' ) => (token::BangEquals, 2),
				_ => (token::Error( "Unexpected `!`." ), 1),
			},
			
			'+' => match self.peek_char( self.offset + 1 ) {
				Some( '=' ) => (token::PlusEquals, 2),
				_ => (token::Plus, 1),
			},
			'-' => match self.peek_char( self.offset + 1 ) {
				Some( '0'..'9' ) => self.match_number_literal(),
				Some( '=' ) => (token::DashEquals, 2),
				Some( '>' ) => (token::Arrow, 2),
				_ => (token::Dash, 1),
			},
			'*' => match self.peek_char( self.offset + 1 ) {
				Some( '=' ) => (token::AsteriskEquals, 2),
				_ => (token::Asterisk, 1),
			},
			'/' => match self.peek_char( self.offset + 1 ) {
				Some( '=' ) => (token::SlashEquals, 2),
				_ => (token::Slash, 1),
			},
			'%' => match self.peek_char( self.offset + 1 ) {
				Some( '=' ) => (token::PercentEquals, 2),
				_ => (token::Percent, 1),
			},
			'|' => (token::VerticalBar, 1),
			
			// identifier
			'a'..'z' | 'A'..'Z' | '_' | ':' => {
				
				let mut length = 1;
				
				loop {
					match self.peek_char( self.offset + length ) {
						Some( 'a'..'z' ) | Some( 'A'..'Z' ) | Some( '0'..'9' ) | Some( '_' ) | Some( ':' ) => { length += 1; }
						_ => { break; }
					}
				}
				
				let sub = self.source.slice( self.offset, self.offset + length );
				
				let value = match sub {
					"and" => token::And,
					"catch" => token::Catch,
					"class" => token::Class,
					"else" => token::Else,
					"extends" => token::Extends,
					"false" => token::False,
					"finally" => token::Finally,
					"for" => token::For,
					"function" => token::Function,
					"if" => token::If,
					"import" => token::Import,
					"in" => token::In,
					"is" => token::Is,
					"let" => token::Let,
					"new" => token::New,
					"not" => token::Not,
					"nothing" => token::Nothing,
					"or" => token::Or,
					"print" => token::Print,
					"return" => token::Return,
					"this" => token::This,
					"throw" => token::Throw,
					"true" => token::True,
					"try" => token::Try,
					"while" => token::While,
					_ => token::Identifier( sub ),
				};
				
				(value, length)
			}
			
			// variable
			'$' => {
				
				let mut length = 0;
				
				loop {
					match self.peek_char( self.offset + 1 + length ) {
						Some( 'a'..'z' )
						| Some( 'A'..'Z' )
						| Some( '0'..'9' )
						| Some( '_' )
						| Some( ':' )
						| Some( '!' ) => { length += 1; },
						_ => { break; }
					}
				}
				
				if length == 0 {
					(token::Error( "Unexpected `$`." ), 0)
				} else {
					let sub = self.source.slice( self.offset + 1, self.offset + 1 + length );
					(token::Variable( sub ), 1 + length)
				}
			}
			
			// string literals
			'"' => {
				let mut length = 1;
				loop {
					match self.peek_char( self.offset + length ) {
						Some( '\\' ) => {
							length += 2;
						}
						Some( '"' ) => {
							length += 1;
							break;
						}
						Some( _ ) => {
							length += utf8_char_width( self.source[ self.offset + length ] );
						}
						None => {
							return (token::Error( "Unterminated string literal." ), 0);
						}
					}
				}
				let sub = self.source.slice( self.offset, self.offset + length );
				(token::String( sub ), length)
			}
			
			'0'..'9' => self.match_number_literal(),
			
			// invalid
			_ => (token::Error( "Unexpected character." ), 0)
		}
	}
	
	fn match_number_literal( &self ) -> (token::Token<'src>, uint) {
		
		let mut l = 0;
		let mut float = false;
		
		
		match self.peek_char( self.offset ) {
			Some( '-' ) => {
				l += 1;
			},
			_ => {},
		}
		
		match self.peek_char( self.offset + l ) {
			Some( '0' ) => {
				l += 1;
				match self.peek_char( self.offset + l ) {
					Some( '0'..'9' ) => {
						return (token::Error( "Invalid number literal." ), 0);
					},
					_ => {}
				}
			},
			Some( '1'..'9' ) => l += 1,
			_ => assert!( false )
		}
		
		loop {
			match self.peek_char( self.offset + l ) {
				Some( '0'..'9' ) => l += 1,
				Some( '.' ) if ! float => {
					match self.peek_char( self.offset + l + 1 ) {
						Some( '0'..'9' ) => l += 2,
						_ => break,
					}
					float = true;
				}
				_ => break
			}
		}
		
		let sub = self.source.slice( self.offset, self.offset + l );
		
		if float {
			(token::Float( sub ), l)
		} else {
			(token::Integer( sub ), l)
		}
	}
}

#[cfg(test)]
mod test {
	use std::vec::Vec;
	use parse::lexer::Lexer;
	use parse::token;
	
	fn lex<'src>( source: &'src str ) -> Vec<token::Token<'src>> {
		let mut lexer = Lexer::new( source );
		let mut tokens = Vec::new();
		loop {
			let token = lexer.read();
			
			if token == token::Eof {
				break;
			}
			
			tokens.push( token );
			
			match token {
				token::Error(..) => break,
				_ => {},
			}
		}
		tokens
	}
	
	#[test]
	fn test_edge_cases() {
		assert!( lex( "" ) == vec!() );
	}
	
	#[test]
	fn test_newline() {
		assert!( lex( "\n" ) == vec!( token::Newline ) );
	}
	
	#[test]
	fn test_whitespace() {
		assert!( lex( " " ) == vec!() );
		assert!( lex( "\t" ) == vec!() );
	}
	
	#[test]
	fn test_comments() {
		assert!( lex( "// foo" ) == vec!() );
		assert!( lex( "// foo\n" ) == vec!() );
		assert!( lex( "//\nfoo" ) == vec!( token::Identifier( "foo" ) ) );
		assert!( lex( "/* foo */" ) == vec!() );
		assert!( lex( "/* foo\nbar */" ) == vec!() );
		assert!( lex( "/*" ) == vec!() );
		assert!( lex( "/* foo */ bar" ) == vec!( token::Identifier( "bar" ) ) );
		assert!( lex( "/*** foo ***/ bar" ) == vec!( token::Identifier( "bar" ) ) );
		assert!( lex( "/*** foo */ bar" ) == vec!() );
		assert!( lex( "/* foo ***/ bar" ) == vec!() );
	}
	
	#[test]
	fn test_symbols() {
		assert!( lex( "{" ) == vec!( token::LeftCurlyBracket ) );
		assert!( lex( "}" ) == vec!( token::RightCurlyBracket ) );
		assert!( lex( "[" ) == vec!( token::LeftSquareBracket ) );
		assert!( lex( "]" ) == vec!( token::RightSquareBracket ) );
		assert!( lex( "(" ) == vec!( token::LeftParenthesis ) );
		assert!( lex( ")" ) == vec!( token::RightParenthesis ) );
		assert!( lex( "<" ) == vec!( token::LeftAngleBracket ) );
		assert!( lex( ">" ) == vec!( token::RightAngleBracket ) );
		assert!( lex( "." ) == vec!( token::Dot ) );
		assert!( lex( "," ) == vec!( token::Comma ) );
		assert!( lex( "=" ) == vec!( token::Equals ) );
		assert!( lex( "+" ) == vec!( token::Plus ) );
		assert!( lex( "-" ) == vec!( token::Dash ) );
		assert!( lex( "*" ) == vec!( token::Asterisk ) );
		assert!( lex( "/" ) == vec!( token::Slash ) );
		assert!( lex( "%" ) == vec!( token::Percent ) );
		assert!( lex( "|" ) == vec!( token::VerticalBar ) );
		assert!( lex( "<=" ) == vec!( token::LeftAngleBracketEquals ) );
		assert!( lex( ">=" ) == vec!( token::RightAngleBracketEquals ) );
		assert!( lex( "+=" ) == vec!( token::PlusEquals ) );
		assert!( lex( "-=" ) == vec!( token::DashEquals ) );
		assert!( lex( "*=" ) == vec!( token::AsteriskEquals ) );
		assert!( lex( "/=" ) == vec!( token::SlashEquals ) );
		assert!( lex( "%=" ) == vec!( token::PercentEquals ) );
		assert!( lex( "==" ) == vec!( token::EqualsEquals ) );
		assert!( lex( "!=" ) == vec!( token::BangEquals ) );
		assert!( lex( "->" ) == vec!( token::Arrow ) );
		
		assert!( lex( "!" ) == vec!( token::Error( "Unexpected `!`." ) ) );
	}
	
	#[test]
	fn test_keywords() {
		assert!( lex( "and" ) == vec!( token::And ) );
		assert!( lex( "catch" ) == vec!( token::Catch ) );
		assert!( lex( "class" ) == vec!( token::Class ) );
		assert!( lex( "else" ) == vec!( token::Else ) );
		assert!( lex( "extends" ) == vec!( token::Extends ) );
		assert!( lex( "false" ) == vec!( token::False ) );
		assert!( lex( "finally" ) == vec!( token::Finally ) );
		assert!( lex( "for" ) == vec!( token::For ) );
		assert!( lex( "function" ) == vec!( token::Function ) );
		assert!( lex( "if" ) == vec!( token::If ) );
		assert!( lex( "in" ) == vec!( token::In ) );
		assert!( lex( "is" ) == vec!( token::Is ) );
		assert!( lex( "let" ) == vec!( token::Let ) );
		assert!( lex( "new" ) == vec!( token::New ) );
		assert!( lex( "not" ) == vec!( token::Not ) );
		assert!( lex( "nothing" ) == vec!( token::Nothing ) );
		assert!( lex( "or" ) == vec!( token::Or ) );
		assert!( lex( "return" ) == vec!( token::Return ) );
		assert!( lex( "this" ) == vec!( token::This ) );
		assert!( lex( "throw" ) == vec!( token::Throw ) );
		assert!( lex( "true" ) == vec!( token::True ) );
		assert!( lex( "try" ) == vec!( token::Try ) );
		assert!( lex( "while" ) == vec!( token::While ) );
	}
	
	#[test]
	fn test_identifiers() {
		assert!( lex( "foo" ) == vec!( token::Identifier( "foo" ) ) );
		assert!( lex( "Foo" ) == vec!( token::Identifier( "Foo" ) ) );
		assert!( lex( "FOO" ) == vec!( token::Identifier( "FOO" ) ) );
		assert!( lex( "foo_bar" ) == vec!( token::Identifier( "foo_bar" ) ) );
		assert!( lex( "_foo" ) == vec!( token::Identifier( "_foo" ) ) );
		assert!( lex( "foo_" ) == vec!( token::Identifier( "foo_" ) ) );
		assert!( lex( "foo:bar" ) == vec!( token::Identifier( "foo:bar" ) ) );
		assert!( lex( ":foo" ) == vec!( token::Identifier( ":foo" ) ) );
		assert!( lex( "foo:" ) == vec!( token::Identifier( "foo:" ) ) );
		assert!( lex( "foo123" ) == vec!( token::Identifier( "foo123" ) ) );
		assert!( lex( "123foo" ) == vec!( token::Integer( "123" ), token::Identifier( "foo" ) ) );
	}
	
	#[test]
	fn test_variables() {
		assert!( lex( "$" ) == vec!( token::Error( "Unexpected `$`." ) ) );
		assert!( lex( "$foo" ) == vec!( token::Variable( "foo" ) ) );
		assert!( lex( "$foo_bar" ) == vec!( token::Variable( "foo_bar" ) ) );
		assert!( lex( "$_foo" ) == vec!( token::Variable( "_foo" ) ) );
		assert!( lex( "$foo_" ) == vec!( token::Variable( "foo_" ) ) );
		assert!( lex( "$foo:bar" ) == vec!( token::Variable( "foo:bar" ) ) );
		assert!( lex( "$:foo" ) == vec!( token::Variable( ":foo" ) ) );
		assert!( lex( "$foo:" ) == vec!( token::Variable( "foo:" ) ) );
		assert!( lex( "$foo123" ) == vec!( token::Variable( "foo123" ) ) );
		assert!( lex( "$123foo" ) == vec!( token::Variable( "123foo" ) ) );
	}
	
	#[test]
	fn test_literals() {
		assert!( lex( "0" ) == vec!( token::Integer( "0" ) ) );
		assert!( lex( "-0" ) == vec!( token::Integer( "-0" ) ) );
		assert!( lex( "3" ) == vec!( token::Integer( "3" ) ) );
		assert!( lex( "-10" ) == vec!( token::Integer( "-10" ) ) );
		assert!( lex( "03" ) == vec!( token::Error( "Invalid number literal." ) ) );
		
		assert!( lex( "0.1" ) == vec!( token::Float( "0.1" ) ) );
		assert!( lex( "1.1" ) == vec!( token::Float( "1.1" ) ) );
		assert!( lex( "12.34" ) == vec!( token::Float( "12.34" ) ) );
		assert!( lex( "1." ) == vec!( token::Integer( "1" ), token::Dot ) );
		assert!( lex( ".1" ) == vec!( token::Dot, token::Integer( "1" ) ) );
		
		assert!( lex( "\"\"" ) == vec!( token::String( "\"\"" ) ) );
		assert!( lex( "\"test\"" ) == vec!( token::String( "\"test\"" ) ) );
		assert!( lex( "\"" ) == vec!( token::Error( "Unterminated string literal." ) ) );
	}
	
	#[test]
	fn test_invalid() {
		assert!( lex( "#" ) == vec!( token::Error( "Unexpected character." ) ) );
		assert!( lex( "僯" ) == vec!( token::Error( "Unexpected character." ) ) );
	}
}
