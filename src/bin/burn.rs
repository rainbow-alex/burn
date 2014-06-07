#![feature(macro_rules)]

extern crate burn;
extern crate debug;

use std::os;
use std::io;
use std::path::posix::Path;
use burn::lang::Value;
use burn::vm::{VirtualMachine, UncaughtThrowableHandler};
use burn::repl;

enum Input {
	Stdin,
	File( String ),
}

fn main() {
	
	let mut input = None::<Input>;
	
	let mut args = os::args().move_iter().skip(1);
	for arg in args {
		match arg.as_slice() {
			"-d" | "--debug" => {
				unsafe { burn::DEBUG = true; }
			}
			"-h" | "--help" => {
				help();
				return;
			}
			"-" => {
				input = Some( Stdin );
				break;
			}
			_ => {
				input = Some( File( arg ) );
				break;
			}
		}
	}
	
	let remaining_args = args.collect::<Vec<String>>();
	
	match input {
		Some( i ) => { process_input( i, remaining_args ); }
		None => { repl(); }
	}
}

fn help() {
	println!(
"usage: burn [options...] <file> [args...]

Read and run burn program from file. Use - to read from stdin.

options:
-d | --debug    Print bytecode and instruction info.
-h | --help     Print this help message."
	);
}

fn process_input( input: Input, args: Vec<String> ) {
	
	(args); // todo!
	
	let source = match input {
		
		Stdin => {
			match io::stdin().read_to_str() {
				Ok( s ) => s,
				Err( e ) => {
					let _ = writeln!( io::stderr(), "Error reading stdin: {}", e );
					os::set_exit_status( 1 );
					return;
				}
			}
		}
		
		File( name ) => {
			
			let path = Path::new( name.clone() );
			let mut file = match io::File::open( &path ) {
				Ok( f ) => f,
				Err( e ) => {
					let _ = writeln!( io::stderr(), "Error opening \"{}\": {}", name, e );
					os::set_exit_status( 1 );
					return;
				}
			};
			
			match file.read_to_str() {
				Ok( s ) => s,
				Err( e ) => {
					let _ = writeln!( io::stderr(), "Error reading \"{}\": {}", name, e );
					os::set_exit_status( 1 );
					return;
				}
			}
		}
	};
	
	let mut vm = VirtualMachine::new();
	vm.on_uncaught_throwable( box ErrorPrinter as Box<UncaughtThrowableHandler> );
	vm.run_script( source.as_slice() );
	vm.run();
}

fn repl() {
	
	let mut vm = VirtualMachine::new();
	let mut state = repl::State::new();
	// TODO error handling
	
	loop {
		
		let mut input = String::new();
		loop {
			print!( "> " );
			let line = match io::stdin().read_line() {
				Ok( l ) => l,
				Err( e ) => {
					let _ = writeln!( io::stderr(), "Error reading stdin: {}", e );
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
		
		vm.on_uncaught_throwable( box ErrorPrinter as Box<UncaughtThrowableHandler> );
		vm.run_repl( &mut state, input.as_slice() );
		vm.run();
	}
}

struct ErrorPrinter;

	impl UncaughtThrowableHandler for ErrorPrinter {
		fn handle_uncaught_throwable( &mut self, vm: &mut VirtualMachine, t: Value ) {
			let _ = writeln!( io::stderr(), "Uncaught throwable:" );
			let _ = writeln!( io::stderr(), "{}", vm.to_string( t ).ok().unwrap() ); // todo! handle err
			// todo! os::set_exit_status( 2 );
		}
	}
