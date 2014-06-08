#![feature(macro_rules)]

extern crate burn;

use std::os;
use std::io;
use std::path::posix::Path;
use burn::lang::origin;
use burn::lang::Value;
use burn::vm::{VirtualMachine, Error, UncaughtThrowableHandler};
use burn::repl;
use burn::util;

macro_rules! errln(
	($($arg:tt)*) => ( let _ = ::std::io::stderr().write_line( format!($($arg)*).as_slice() ) )
)

static HELP: &'static str = 
"usage: burn [options...] <file> [args...]

Read and run burn program from file. Use - to read from stdin.

options:
-d | --debug    Print bytecode and instruction info.
-h | --help     Print this help message.";

fn main() {
	
	let mut burn = Burn {
		verbose: true,
	};
	
	enum Input {
		Stdin,
		File( Path ),
		Repl,
	}
	
	let mut input = Repl;
	let mut args = os::args().move_iter().skip(1);
	
	for arg in args {
		match arg.as_slice() {
			"-d" | "--debug" => {
				unsafe { burn::DEBUG = true; }
			}
			"-h" | "--help" => {
				let _ = io::stdout().write_line( HELP );
				return;
			}
			"-q" | "--quiet" => {
				burn.verbose = false;
			}
			"-" => {
				input = Stdin;
				break;
			}
			_ => {
				input = File( Path::new( arg ) );
				break;
			}
		}
	}
	
	let remaining_args = args.collect::<Vec<String>>();
	
	match input {
		Stdin => { burn.run_stdin( remaining_args ); }
		File( path ) => { burn.run_script( path, remaining_args ); }
		Repl => { burn.run_repl( remaining_args ); }
	}
}

struct Burn {
	verbose: bool,
}

	impl Burn {
		
		fn run_stdin( &self, args: Vec<String> ) {
			
			(args);
			
			let mut vm = VirtualMachine::new();
			vm.on_uncaught_throwable( box OsStatusUpdater as Box<UncaughtThrowableHandler> );
			vm.on_uncaught_throwable( box ErrorPrinter as Box<UncaughtThrowableHandler> );
			
			let origin = box origin::Stdin as Box<origin::Origin>;
			
			let source = match io::stdin().read_to_str() {
				Ok( source ) => source,
				Err( m ) => {
					errln!( "Error reading {}: {}", origin, m );
					os::set_exit_status( 1 );
					return;
				}
			};
			
			match vm.schedule_source( origin, None, source.as_slice() ) {
				
				Ok( () ) => {
					vm.run();
				}
				
				Err( errors ) => {
					os::set_exit_status( 1 );
					for error in errors.move_iter() {
						self.print_libburn_error( source.as_slice(), error );
					}
				}
			};
		}
		
		fn run_script( &self, path: Path, args: Vec<String> ) {
			
			(args);
			
			let mut vm = VirtualMachine::new();
			vm.on_uncaught_throwable( box OsStatusUpdater as Box<UncaughtThrowableHandler> );
			vm.on_uncaught_throwable( box ErrorPrinter as Box<UncaughtThrowableHandler> );
			
			let origin = box origin::Script { path: path } as Box<origin::Origin>;
			
			let mut file = match io::File::open( origin.get_path().unwrap() ) {
				Ok( f ) => f,
				Err( m ) => {
					errln!( "Error opening {}: {}", origin, m );
					os::set_exit_status( 1 );
					return;
				}
			};
			
			let source = match file.read_to_str() {
				Ok( source ) => source,
				Err( m ) => {
					errln!( "Error reading {}: {}", origin, m );
					os::set_exit_status( 1 );
					return;
				}
			};
			
			match vm.schedule_source( origin, None, source.as_slice() ) {
				
				Ok( () ) => {
					vm.run();
				}
				
				Err( errors ) => {
					os::set_exit_status( 1 );
					for error in errors.move_iter() {
						self.print_libburn_error( source.as_slice(), error );
					}
				}
			};
		}
		
		fn run_repl( &self, args: Vec<String> ) {
			
			(args);
			
			let mut vm = VirtualMachine::new();
			vm.on_uncaught_throwable( box ErrorPrinter as Box<UncaughtThrowableHandler> );
			let mut state = repl::State::new();
			
			loop {
				
				let mut input = String::new();
				loop {
					print!( "> " );
					let line = match io::stdin().read_line() {
						Ok( l ) => l,
						Err( e ) => {
							errln!( "Error reading stdin: {}", e );
							os::set_exit_status( 1 );
							return;
						}
					};
					
					if line.as_slice() == "\n" {
						break;
					} else {
						input.push_str( line.as_slice() );
					}
				}
				
				let origin = box origin::Stdin as Box<origin::Origin>;
				match vm.schedule_source( origin, Some( &mut state ), input.as_slice() ) {
					Ok( () ) => {
						vm.run();
					}
					Err( errors ) => {
						for error in errors.move_iter() {
							self.print_libburn_error( input.as_slice(), error );
						}
					}
				}
			}
		}
		
		fn print_libburn_error( &self, source: &str, error: Box<Error> ) {
			
			let line = util::source::find_line( source, error.get_source_offset() );
			
			errln!( "{}", error.get_message() );
			errln!( "in {} on line {}", error.get_origin().get_name(), line.no );
			
			if self.verbose {
				self.print_error_line_fragment( source, line );
			}
		}
		
		fn print_error_line_fragment( &self, source: &str, line: util::source::Line ) {
			errln!( "{}", source.slice( line.start, line.end ) );
			for c in source.slice( line.start, line.offset ).chars() {
				let _ = io::stderr().write_char( if c == '\t' { c } else { ' ' } );
			}
			errln!( "^" );
		}
	}

struct ErrorPrinter;

	impl UncaughtThrowableHandler for ErrorPrinter {
		fn handle_uncaught_throwable( &mut self, vm: &mut VirtualMachine, t: Value ) {
			errln!( "Uncaught throwable:\n{}", vm.to_string( t ).ok().unwrap() );
		}
	}

struct OsStatusUpdater;

	impl UncaughtThrowableHandler for OsStatusUpdater {
		fn handle_uncaught_throwable( &mut self, _: &mut VirtualMachine, _: Value ) {
			os::set_exit_status( 2 );
		}
	}
