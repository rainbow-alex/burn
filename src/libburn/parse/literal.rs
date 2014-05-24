use std::strbuf::StrBuf;
use std::str::utf8_char_width;

pub fn parse_int( source: &str ) -> Result<i64,StrBuf> {
	match from_str::<i64>( source ) {
		Some( i ) => Ok( i ),
		None => Err( "Integer literal is out of range.".to_owned() ),
	}
}

pub fn parse_float( source: &str ) -> Result<f64,StrBuf> {
	match from_str::<f64>( source ) {
		Some( f ) => Ok( f ),
		None => Err( "Float literal is out of range.".to_owned() ),
	}
}

pub fn parse_string( source: &str ) -> Result<StrBuf,(StrBuf,uint)> {
	let mut buf = StrBuf::new();
	let raw: bool;
	let delimiter: char;
	let mut i: uint;
	
	if source[0] as char == 'r' {
		raw = true;
		delimiter = source[1] as char;
		i = 2;
	} else {
		raw = false;
		delimiter = source[0] as char;
		i = 1;
	}
	
	loop {
		match source[i] as char {
			'\\' => {
				match source[i+1] as char {
					'\\' => {
						buf.push_char( '\\' );
						i += 2;
					}
					c @ _ if c == delimiter => {
						buf.push_char( delimiter );
						i += 2;
					}
					_ if raw => {
						buf.push_char( '\\' );
						i += 1;
					}
					'n' => {
						buf.push_char( '\n' );
						i += 2;
					}
					't' => {
						buf.push_char( '\t' );
						i += 2;
					}
					_ => {
						return Err( ("Invalid escape sequence".to_owned(), i) );
					}
				}
			},
			c @ _ if c == delimiter => break,
			_ => {
				buf.push_char( source.char_at( i ) );
				i += utf8_char_width( source[ i ] );
			}
		}
	}
	
	Ok( buf.into_owned() )
}

#[cfg(test)]
mod test {
	
	use super::{parse_int, parse_float, parse_string};
	
	#[test]
	fn test_parse_int() {
		assert!( parse_int( "0" ) == Ok( 0 ) );
		assert!( parse_int( "3" ) == Ok( 3 ) );
		assert!( parse_int( "123456789" ) == Ok( 123456789 ) );
		assert!( parse_int( "-10" ) == Ok( -10 ) );
		assert!( parse_int( "99999999999999999999999999999999999" ) == Err( "Integer literal is out of range.".to_owned() ) );
	}
	
	#[test]
	fn test_parse_float() {
		assert!( parse_float( "3.1" ) == Ok( 3.1 ) );
	}
	
	#[test]
	fn test_parse_string() {
		assert!( parse_string( r#""test""# ) == Ok( "test".to_owned() ) );
	}
}
