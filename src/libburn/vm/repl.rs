use std::collections::HashMap;
use mem::rc::Rc;
use lang::value;
use lang::identifier::Identifier;

/// Persists declared variables and their values for a read-eval-print loop.
pub struct State {
	#[doc(hidden)]
	pub variables: HashMap<Identifier, Rc<value::Value>>,
}

	impl State {
		
		/// Create a new, empty `State` instance.
		pub fn new() -> State {
			State {
				variables: HashMap::new(),
			}
		}
		
		#[doc(hidden)]
		pub fn declare_variable( &mut self, name: Identifier ) {
			self.variables.insert( name, Rc::new( value::Nothing ) );
		}
	}
