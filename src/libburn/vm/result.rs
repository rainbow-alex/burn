use lang::value;
use vm::error::Error;

pub enum Result {
	Done,
	Fail( Vec<Box<Error>> ),
	UncaughtThrowable( value::Value ),
}
