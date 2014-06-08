use vm::error::AnalysisError;
use parse::node;
use vm::analysis::annotation;
use vm::repl;

struct Frame {
	n_local_variables: uint,
	n_shared_local_variables: uint,
	n_static_bound_variables: uint,
	n_shared_bound_variables: uint,
}

pub struct AnalyzeAllocation {
	frames: Vec<Frame>,
	pub errors: Vec<AnalysisError>,
}

	impl AnalyzeAllocation {
		
		pub fn new() -> AnalyzeAllocation {
			AnalyzeAllocation {
				frames: Vec::new(),
				errors: Vec::new(),
			}
		}
		
		fn push_frame( &mut self ) {
			self.frames.push( Frame {
				n_local_variables: 0,
				n_shared_local_variables: 0,
				n_static_bound_variables: 0,
				n_shared_bound_variables: 0,
			} );
		}
		
		fn pop_frame( &mut self ) {
			self.frames.pop();
		}
		
		fn get_current_frame<'l>( &'l mut self ) -> &'l mut Frame {
			self.frames.mut_last().unwrap()
		}
		
		pub fn analyze_root( &mut self, root: &mut node::Root ) {
			self.analyze_frame( &mut root.frame, 0 );
		}
		
		pub fn analyze_repl_root(
			&mut self,
			root: &mut node::Root,
			repl_state: &mut repl::State
		) {
			self.analyze_frame( &mut root.frame, repl_state.variables.len() );
		}
		
		fn analyze_frame(
			&mut self,
			frame: &mut annotation::Frame,
			n_repl_vars: uint
		) {
			self.push_frame();
			
			for variable in frame.declared_variables.mut_iter().take( n_repl_vars ) {
				variable.local_storage_type = annotation::storage::SharedLocal;
				variable.bound_storage_type = annotation::storage::SharedBound;
			}
			
			for variable in frame.declared_variables.mut_iter().skip( n_repl_vars ) {
				self.determine_variable_storage_types( *variable );
			}
			
			for variable in frame.declared_variables.mut_iter() {
				self.determine_declared_variable_storage_index( *variable );
			}
			
			for &mut function in frame.functions.iter() {
				match_enum!( *function.deref_mut() to node::Function {
					frame: ref mut frame,
					..
				} => {
					self.analyze_frame( frame, 0 );
				} );
			}
			
			frame.n_local_variables = self.get_current_frame().n_local_variables;
			frame.n_shared_local_variables = self.get_current_frame().n_shared_local_variables;
			
			if frame.closure.is_some() {
				self.analyze_closure( frame.closure.as_mut().unwrap() );
			}
			
			self.pop_frame();
		}
		
		fn analyze_closure( &mut self, closure: &mut annotation::Closure ) {
			
			for binding in closure.bindings.mut_iter() {
				self.determine_binding_storage_index( binding );
			}
			
			closure.n_static_bound_variables = self.get_current_frame().n_static_bound_variables;
			closure.n_shared_bound_variables = self.get_current_frame().n_shared_bound_variables;
		}
		
		fn determine_variable_storage_types( &mut self, variable: &mut annotation::Variable ) {
			match variable.root_binds.len() {
				
				0 => {
					variable.local_storage_type = annotation::storage::Local;
				}
				
				1 => {
					let only_bind = variable.root_binds.get(0);
					
					for write in variable.writes.iter() {
						if write.time > only_bind.time {
							// The variable is assigned to after binding.
							variable.local_storage_type = annotation::storage::SharedLocal;
							variable.bound_storage_type = annotation::storage::SharedBound;
							return;
						}
					}
					
					if ! only_bind.mutable {
						// The variable is never assigned to after binding.
						// It is effectively immutable!
						variable.local_storage_type = annotation::storage::Local;
						variable.bound_storage_type = annotation::storage::StaticBound;
						return;
					}
					
					for read in variable.reads.iter() {
						if read.time > only_bind.time {
							// The variable is assigned to inside the binding function,
							// but also read after binding.
							variable.local_storage_type = annotation::storage::SharedLocal;
							variable.bound_storage_type = annotation::storage::SharedBound;
							return;
						}
					}
					
					if ! variable.n_binds == 1 {
						// In the declaring frame, the variable is dead after binding.
						// The binding function mutates the variable,
						// but since it is the only owner of the value, it can be static.
						variable.local_storage_type = annotation::storage::Local;
						variable.bound_storage_type = annotation::storage::StaticBound;
					}
					
					// In the declaring frame, the variable is dead after binding, so it can be local.
					// The binding functions have to share, since they mutate the variable.
					variable.local_storage_type = annotation::storage::Local;
					variable.bound_storage_type = annotation::storage::SharedBound;
				}
				
				_ => {
					
					for bind in variable.root_binds.iter() {
						if bind.mutable {
							// Multiple bindings and at least one mutates.
							variable.local_storage_type = annotation::storage::SharedLocal;
							variable.bound_storage_type = annotation::storage::SharedBound;
						}
					}
					
					let first_binding = variable.root_binds.get(0);
					
					for write in variable.writes.iter() {
						if write.time > first_binding.time {
							// The variable is assigned to after a binding.
							variable.local_storage_type = annotation::storage::SharedLocal;
							variable.bound_storage_type = annotation::storage::SharedBound;
							return;
						}
					}
					
					// The variable is never assigned to after binding.
					// It is effectively immutable!
					variable.local_storage_type = annotation::storage::Local;
					variable.bound_storage_type = annotation::storage::StaticBound;
				}
			}
		}
		
		fn determine_declared_variable_storage_index( &mut self, variable: &mut annotation::Variable ) {
			match variable.local_storage_type {
				
				annotation::storage::Local => {
					variable.local_storage_index = self.get_current_frame().n_local_variables;
					self.get_current_frame().n_local_variables += 1;
				}
				
				annotation::storage::SharedLocal => {
					variable.local_storage_index = self.get_current_frame().n_shared_local_variables;
					self.get_current_frame().n_shared_local_variables += 1;
				}
			}
		}
		
		fn determine_binding_storage_index( &mut self, binding: &mut annotation::Binding ) {
			match binding.variable.bound_storage_type {
				
				annotation::storage::StaticBound => {
					binding.storage_index = self.get_current_frame().n_static_bound_variables;
					self.get_current_frame().n_static_bound_variables += 1;
				}
				
				annotation::storage::SharedBound => {
					binding.storage_index = self.get_current_frame().n_shared_bound_variables;
					self.get_current_frame().n_shared_bound_variables += 1;
				}
			}
		}
	}
