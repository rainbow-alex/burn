use vm::error::AnalysisError;
use parse::node;
use vm::analysis;
use vm::analysis::{FrameAnalysis, VariableAnalysis, ClosureAnalysis, Binding};
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
		
		#[inline(always)]
		fn push_frame( &mut self ) {
			self.frames.push( Frame {
				n_local_variables: 0,
				n_shared_local_variables: 0,
				n_static_bound_variables: 0,
				n_shared_bound_variables: 0,
			} );
		}
		
		#[inline(always)]
		fn pop_frame( &mut self ) {
			self.frames.pop();
		}
		
		#[inline(always)]
		fn get_current_frame<'l>( &'l mut self ) -> &'l mut Frame {
			self.frames.mut_last().unwrap()
		}
		
		pub fn analyze_root( &mut self, root: &mut node::Root ) {
			self.analyze_frame( &mut root.frame, 0 );
		}
		
		pub fn analyze_repl_root( &mut self, root: &mut node::Root, repl_state: &mut repl::State ) {
			self.analyze_frame( &mut root.frame, repl_state.variables.len() );
		}
		
		fn analyze_frame( &mut self, frame: &mut FrameAnalysis, n_repl_vars: uint ) {
			
			self.push_frame();
			
			for variable in frame.declared.mut_iter().take( n_repl_vars ) {
				variable.local_storage_type = analysis::SharedLocalStorage;
				variable.bound_storage_type = analysis::SharedBoundStorage;
			}
			
			for variable in frame.declared.mut_iter().skip( n_repl_vars ) {
				self.determine_variable_storage_types( *variable );
			}
			
			for variable in frame.declared.mut_iter() {
				self.determine_declared_variable_storage_index( *variable );
			}
			
			for function in frame.functions.mut_iter() {
				match *function.get() {
					
					node::Function { frame: ref mut frame, .. } => {
						self.analyze_frame( frame, 0 );
					}
					
					_ => { fail!(); }
				}
			}
			
			frame.n_local_variables = self.get_current_frame().n_local_variables;
			frame.n_shared_local_variables = self.get_current_frame().n_shared_local_variables;
			
			if frame.closure.is_some() {
				self.analyze_closure( frame.closure.as_mut().unwrap() );
			}
			
			self.pop_frame();
		}
		
		fn analyze_closure( &mut self, closure: &mut ClosureAnalysis ) {
			
			for binding in closure.bindings.mut_iter() {
				self.determine_binding_storage_index( binding );
			}
			
			closure.n_static_bound_variables = self.get_current_frame().n_static_bound_variables;
			closure.n_shared_bound_variables = self.get_current_frame().n_shared_bound_variables;
		}
		
		fn determine_variable_storage_types( &mut self, variable: &mut VariableAnalysis ) {
			match variable.root_binds.len() {
				
				0 => {
					variable.local_storage_type = analysis::LocalStorage;
				}
				
				1 => {
					let only_bind = variable.root_binds.get(0);
					
					for write in variable.writes.iter() {
						if write.time > only_bind.time {
							// The variable is assigned to after binding.
							variable.local_storage_type = analysis::SharedLocalStorage;
							variable.bound_storage_type = analysis::SharedBoundStorage;
							return;
						}
					}
					
					if ! only_bind.mutable {
						// The variable is never assigned to after binding.
						// It is effectively immutable!
						variable.local_storage_type = analysis::LocalStorage;
						variable.bound_storage_type = analysis::StaticBoundStorage;
						return;
					}
					
					for read in variable.reads.iter() {
						if read.time > only_bind.time {
							// The variable is assigned to inside the binding function, but also read after binding.
							variable.local_storage_type = analysis::SharedLocalStorage;
							variable.bound_storage_type = analysis::SharedBoundStorage;
							return;
						}
					}
					
					if ! variable.n_binds == 1 {
						// In the declaring frame, the variable is dead after binding.
						// The binding function mutates the variable, but since it is the only owner of the value, it can be static.
						variable.local_storage_type = analysis::LocalStorage;
						variable.bound_storage_type = analysis::StaticBoundStorage;
					}
					
					// In the declaring frame, the variable is dead after binding, so it can be local.
					// The binding functions have to share, since they mutate the variable.
					variable.local_storage_type = analysis::LocalStorage;
					variable.bound_storage_type = analysis::SharedBoundStorage;
				}
				
				_ => {
					
					for bind in variable.root_binds.iter() {
						if bind.mutable {
							// Multiple bindings and at least one mutates.
							variable.local_storage_type = analysis::SharedLocalStorage;
							variable.bound_storage_type = analysis::SharedBoundStorage;
						}
					}
					
					let first_binding = variable.root_binds.get(0);
					
					for write in variable.writes.iter() {
						if write.time > first_binding.time {
							// The variable is assigned to after a binding.
							variable.local_storage_type = analysis::SharedLocalStorage;
							variable.bound_storage_type = analysis::SharedBoundStorage;
							return;
						}
					}
					
					// The variable is never assigned to after binding.
					// It is effectively immutable!
					variable.local_storage_type = analysis::LocalStorage;
					variable.bound_storage_type = analysis::StaticBoundStorage;
				}
			}
		}
		
		fn determine_declared_variable_storage_index( &mut self, variable: &mut VariableAnalysis ) {
			match variable.local_storage_type {
				
				analysis::LocalStorage => {
					variable.local_storage_index = self.get_current_frame().n_local_variables;
					self.get_current_frame().n_local_variables += 1;
				}
				
				analysis::SharedLocalStorage => {
					variable.local_storage_index = self.get_current_frame().n_shared_local_variables;
					self.get_current_frame().n_shared_local_variables += 1;
				}
			}
		}
		
		fn determine_binding_storage_index( &mut self, binding: &mut Binding ) {
			match binding.variable.get().bound_storage_type {
				
				analysis::StaticBoundStorage => {
					binding.storage_index = self.get_current_frame().n_static_bound_variables;
					self.get_current_frame().n_static_bound_variables += 1;
				}
				
				analysis::SharedBoundStorage => {
					binding.storage_index = self.get_current_frame().n_shared_bound_variables;
					self.get_current_frame().n_shared_bound_variables += 1;
				}
			}
		}
	}
