use error::Error;
use lang::value;

pub enum Result {
	End,
	Errors( Vec<Box<Error>> ),
	Value( value::Value ),
	UncaughtThrowable( value::Value ),
}
