use error::AnalysisError;
use parse::node;
use compile::analysis;
use compile::analysis::{ScopeAnalysis, VariableAnalysis};

pub struct DetermineVariableLifetimeAndStorageType {
	pub errors: Vec<AnalysisError>,
}

	impl DetermineVariableLifetimeAndStorageType {
		
		pub fn new() -> DetermineVariableLifetimeAndStorageType {
			DetermineVariableLifetimeAndStorageType {
				errors: Vec::new(),
			}
		}
		
		pub fn analyze_root( &mut self, root: &mut node::Root ) {
			
			self.analyze_scope( &mut root.scope );
			
			for statement in root.statements.mut_iter() {
				self.analyze_statement( *statement );
			}
		}
		
		fn analyze_statement( &mut self, statement: &mut node::Statement ) {
			match *statement {
				
				node::If {
					test: _,
					scope: ref mut if_scope,
					block: ref mut if_block,
					else_if_clauses: ref mut else_if_clauses,
					else_clause: ref mut else_clause,
				} => {
					
					self.analyze_scope( if_scope );
					for statement in if_block.mut_iter() {
						self.analyze_statement( *statement );
					}
					
					for else_if_clause in else_if_clauses.mut_iter() {
						self.analyze_scope( &mut else_if_clause.scope );
						for statement in else_if_clause.block.mut_iter() {
							self.analyze_statement( *statement );
						}
					}
					
					match *else_clause {
						Some( ref mut else_clause ) => {
							self.analyze_scope( &mut else_clause.scope );
							for statement in else_clause.block.mut_iter() {
								self.analyze_statement( *statement );
							}
						}
						None => {}
					}
				}
				
				node::Try {
					scope: ref mut try_scope,
					block: ref mut try_block,
					catch_clauses: ref mut catch_clauses,
					else_clause: ref mut else_clause,
					finally_clause: ref mut finally_clause,
				} => {
					
					self.analyze_scope( try_scope );
					for statement in try_block.mut_iter() {
						self.analyze_statement( *statement );
					}
					
					for catch_clause in catch_clauses.mut_iter() {
						self.analyze_scope( &mut catch_clause.scope );
						for statement in catch_clause.block.mut_iter() {
							self.analyze_statement( *statement );
						}
					}
					
					match *else_clause {
						Some( ref mut else_clause ) => {
							self.analyze_scope( &mut else_clause.scope );
							for statement in else_clause.block.mut_iter() {
								self.analyze_statement( *statement );
							}
						}
						None => {}
					}
					
					match *finally_clause {
						Some( ref mut finally_clause ) => {
							self.analyze_scope( &mut finally_clause.scope );
							for statement in finally_clause.block.mut_iter() {
								self.analyze_statement( *statement );
							}
						}
						None => {}
					}
				}
				
				node::While {
					test: _,
					scope: ref mut while_scope,
					block: ref mut while_block,
					else_clause: ref mut else_clause,
				} => {
					
					self.analyze_scope( while_scope );
					for statement in while_block.mut_iter() {
						self.analyze_statement( *statement );
					}
					
					match *else_clause {
						Some( ref mut else_clause ) => {
							self.analyze_scope( &mut else_clause.scope );
							for statement in else_clause.block.mut_iter() {
								self.analyze_statement( *statement );
							}
						}
						None => {}
					}
				}
				
				node::Let {..}
				| node::Print {..}
				| node::Return {..}
				| node::Throw {..}
				| node::Assignment {..}
				| node::ExpressionStatement {..}
				=> {}
			}
		}
		
		fn analyze_scope( &mut self, scope: &mut ScopeAnalysis ) {
			
			for variable in scope.declared.mut_iter() {
				self.analyze_declared_variable( variable.get() );
			}
			
			for function in scope.functions.mut_iter() {
				self.analyze_function( function.get() );
			}
		}
		
		fn analyze_function( &mut self, function: &mut node::Expression ) {
			match *function {
				
				node::Function {
					parameters: _,
					closure: _,
					scope: ref mut scope,
					block: ref mut block,
				} => {
					
					self.analyze_scope( scope );
					
					for statement in block.mut_iter() {
						self.analyze_statement( *statement );
					}
				}
				
				_ => fail!(),
			}
		}
		
		fn analyze_declared_variable( &mut self, variable: &mut VariableAnalysis ) {
			
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
						// In the declaring scope, the variable is dead after binding.
						// The binding function mutates the variable, but since it is the only owner of the value, it can be static.
						variable.local_storage_type = analysis::LocalStorage;
						variable.bound_storage_type = analysis::StaticBoundStorage;
					}
					
					// In the declaring scope, the variable is dead after binding, so it can be local.
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
	}
