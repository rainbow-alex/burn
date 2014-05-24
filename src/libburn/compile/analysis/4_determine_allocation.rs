use error::AnalysisError;
use parse::node;
use compile::analysis;
use compile::analysis::{ScopeAnalysis, VariableAnalysis, Binding};

pub struct DetermineAllocation {
	pub errors: Vec<AnalysisError>,
	n_local_variables: uint,
	n_shared_local_variables: uint,
	n_static_bound_variables: uint,
	n_shared_bound_variables: uint,
}

	impl DetermineAllocation {
		
		pub fn new() -> DetermineAllocation {
			DetermineAllocation {
				errors: Vec::new(),
				n_local_variables: 0,
				n_shared_local_variables: 0,
				n_static_bound_variables: 0,
				n_shared_bound_variables: 0,
			}
		}
		
		fn save_frame( &mut self ) -> (uint, uint, uint, uint) {
			
			let state = (
				self.n_local_variables,
				self.n_shared_local_variables,
				self.n_static_bound_variables,
				self.n_shared_bound_variables
			);
			
			self.n_local_variables = 0;
			self.n_shared_local_variables = 0;
			self.n_static_bound_variables = 0;
			self.n_shared_bound_variables = 0;
			
			state
		}
		
		fn restore_frame( &mut self, state: (uint, uint, uint, uint) ) {
			
			let (s0, s1, s2, s3) = state;
			
			self.n_local_variables = s0;
			self.n_shared_local_variables = s1;
			self.n_static_bound_variables = s2;
			self.n_shared_bound_variables = s3;
		}
		
		pub fn analyze_root( &mut self, root: &mut node::Root ) {
			
			self.analyze_scope( &mut root.scope );
			
			for statement in root.statements.mut_iter() {
				self.analyze_statement( *statement );
			}
			
			root.frame.n_local_variables = self.n_local_variables;
			root.frame.n_shared_local_variables = self.n_shared_local_variables;
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
		
		fn analyze_declared_variable( &mut self, variable: &mut VariableAnalysis ) {
			match variable.local_storage_type {
				analysis::LocalStorage => {
					variable.local_storage_index = self.n_local_variables;
					self.n_local_variables += 1;
				}
				analysis::SharedLocalStorage => {
					variable.local_storage_index = self.n_shared_local_variables;
					self.n_shared_local_variables += 1;
				}
			}
		}
		
		fn analyze_function( &mut self, function: &mut node::Expression ) {
			match *function {
				
				node::Function {
					parameters: _,
					closure: ref mut closure,
					scope: ref mut scope,
					block: ref mut block,
				} => {
					
					let state = self.save_frame();
					
					for binding in closure.bound.mut_iter() {
						self.analyze_bound_variable( binding );
					}
					
					self.analyze_scope( scope );
					
					for statement in block.mut_iter() {
						self.analyze_statement( *statement );
					}
					
					closure.frame.n_local_variables = self.n_local_variables;
					closure.frame.n_shared_local_variables = self.n_shared_local_variables;
					closure.n_static_bound_variables = self.n_static_bound_variables;
					closure.n_shared_bound_variables = self.n_shared_bound_variables;
					
					self.restore_frame( state );
				}
				
				_ => fail!(),
			}
		}
		
		fn analyze_bound_variable( &mut self, binding: &mut Binding ) {
			match binding.variable.get().bound_storage_type {
				analysis::StaticBoundStorage => {
					binding.storage_index = self.n_static_bound_variables;
					self.n_static_bound_variables += 1;
				}
				analysis::SharedBoundStorage => {
					binding.storage_index = self.n_shared_bound_variables;
					self.n_shared_bound_variables += 1;
				}
			}
		}
	}
