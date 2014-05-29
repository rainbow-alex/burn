use std::fmt;

#[deriving(Eq)]
pub enum Token<'src> {
	Whitespace,
	Newline,
	
	LeftCurlyBracket, // {
	RightCurlyBracket, // }
	LeftSquareBracket, // [
	RightSquareBracket, // ]
	LeftParenthesis, // (
	RightParenthesis, // )
	LeftAngleBracket, // <
	RightAngleBracket, // >
	Dot, // .
	Comma, // ,
	VerticalBar, // |
	Equals, // =
	Plus, // +
	Dash, // -
	Asterisk, // *
	Slash, // /
	Percent, // %
	
	LeftAngleBracketEquals, // <=
	RightAngleBracketEquals, // >=
	PlusEquals, // +=
	DashEquals, // -=
	AsteriskEquals, // *=
	SlashEquals, // /=
	PercentEquals, // %=
	EqualsEquals, // ==
	BangEquals, // !=
	
	Arrow, // ->
	
	And,
	Catch,
	Class,
	Else,
	False,
	Finally,
	For,
	Function,
	If,
	In,
	Is,
	Let,
	New,
	Not,
	Nothing,
	Or,
	Print,
	Return,
	This,
	Throw,
	True,
	Try,
	While,
	Use,
	
	Identifier( &'src str ), // e.g. foobar
	Variable( &'src str ), // e.g. $foobar (only foobar is stored)
	
	String( &'src str ),
	Integer( &'src str ),
	Float( &'src str ),
	
	Eof,
	
	Error( &'static str ),
}

	impl<'src> fmt::Show for Token<'src> {
		fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result {
			match *self {
				
				Whitespace => write!( f, "whitespace" ),
				Newline => write!( f, "newline" ),
				
				LeftCurlyBracket => write!( f, "`\\{`" ),
				RightCurlyBracket => write!( f, "`\\}`" ),
				LeftSquareBracket => write!( f, "`[`" ),
				RightSquareBracket => write!( f, "`]`" ),
				LeftParenthesis => write!( f, "`(`" ),
				RightParenthesis => write!( f, "`)`" ),
				LeftAngleBracket => write!( f, "`<`" ),
				RightAngleBracket => write!( f, "`>`" ),
				Dot => write!( f, "`.`" ),
				Comma => write!( f, "`,`" ),
				VerticalBar => write!( f, "`|`" ),
				Equals => write!( f, "`=`" ),
				Plus => write!( f, "`+`" ),
				Dash => write!( f, "`-`" ),
				Asterisk => write!( f, "`*`" ),
				Slash => write!( f, "`/`" ),
				Percent => write!( f, "`%`" ),
	
				LeftAngleBracketEquals => write!( f, "`<=`" ),
				RightAngleBracketEquals => write!( f, "`>=`" ),
				PlusEquals => write!( f, "`+=`" ),
				DashEquals => write!( f, "`-=`" ),
				AsteriskEquals => write!( f, "`*=`" ),
				SlashEquals => write!( f, "`/=`" ),
				PercentEquals => write!( f, "`%=`" ),
				EqualsEquals => write!( f, "`==`" ),
				BangEquals => write!( f, "`!=`" ),
				
				Arrow => write!( f, "`->`" ),
				
				And => write!( f, "and" ),
				Catch => write!( f, "catch" ),
				Class => write!( f, "class" ),
				Else => write!( f, "else" ),
				False => write!( f, "false" ),
				Finally => write!( f, "finally" ),
				For => write!( f, "for" ),
				Function => write!( f, "function" ),
				If => write!( f, "if" ),
				In => write!( f, "in" ),
				Is => write!( f, "is" ),
				Let => write!( f, "let" ),
				New => write!( f, "new" ),
				Not => write!( f, "not" ),
				Nothing => write!( f, "nothing" ),
				Or => write!( f, "or" ),
				Print => write!( f, "print" ),
				Return => write!( f, "return" ),
				This => write!( f, "this" ),
				Throw => write!( f, "throw" ),
				True => write!( f, "true" ),
				Try => write!( f, "try" ),
				While => write!( f, "while" ),
				Use => write!( f, "use" ),
				
				Identifier( v ) => write!( f, "IDENTIFIER({})", v ),
				Variable( v ) => write!( f, "VARIABLE(${})", v ),
				
				String( v ) => write!( f, "STRING({})", v ),
				Integer( v ) => write!( f, "INTEGER({})", v ),
				Float( v ) => write!( f, "FLOAT({})", v ),
				
				Eof => write!( f, "<EOF>" ),
				
				Error( m ) => write!( f, "ERROR({})", m ),
			}
		}
	}
