extern crate debug;

pub struct Line<'src> {
	pub no: uint,
	pub offset: uint,
	pub start: uint,
	pub end: uint,
}

pub fn find_line<'l>( source: &'l str, offset: uint ) -> Line<'l> {
	
	let mut chars = source.char_indices().enumerate();
	
	let mut line = Line {
		no: 1,
		offset: source.len(),
		start: 0,
		end: 0,
	};
	
	for (char_i, (byte_i, c)) in chars {
		
		if char_i == offset {
			
			line.offset = byte_i;
			
			if c == '\n' {
				line.end = byte_i;
				return line;
			}
			
			break;
		}
		
		if c == '\n' {
			line.start = byte_i+1;
			line.no += 1;
		}
	}
	
	for (_, (byte_i, c)) in chars {
		
		if c == '\n' {
			line.end = byte_i;
			return line;
		}
	}
	
	line.end = source.len();
	line
}

#[cfg(test)]
mod test {
	
	#[test]
	fn test() {
	
		let source = "foo僯\nbar\nbaz";
		let check = | i, s: &str | {
			let l = find_line( source, i );
			let f = format!( "{}|{}", source.slice( l.start, l.offset ), source.slice( l.offset, l.end ) );
			assert!( f == s.to_string() );
		};
		
		check( 0, "|foo僯" );
		check( 1, "f|oo僯" );
		check( 2, "fo|o僯" );
		check( 3, "foo|僯" );
		check( 4, "foo僯|" );
		check( 5, "|bar" );
		check( 6, "b|ar" );
		check( 7, "ba|r" );
		check( 8, "bar|" );
		check( 9, "|baz" );
		check( 10, "b|az" );
		check( 11, "ba|z" );
		check( 12, "baz|" );
		check( 13, "baz|" );
		check( 14, "baz|" );
	}
}
