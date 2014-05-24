use error::AnalysisError;
use compile::analysis::{Time, FrameAnalysis, ClosureAnalysis, ScopeAnalysis, VariableAnalysis, Binding, ReadVariable, WriteVariable, BindVariable};
use parse::node;
use mem::raw::Raw;
use lang::identifier::Identifier;

pub struct ResolveVariablesAndSetTimes {
	closures: Vec<Raw<ClosureAnalysis>>,
	scopes: Vec<Raw<ScopeAnalysis>>,
	frame: Raw<FrameAnalysis>,
	time: Time,
	pub errors: Vec<AnalysisError>,
}

	impl ResolveVariablesAndSetTimes {
		
		pub fn new() -> ResolveVariablesAndSetTimes {
			ResolveVariablesAndSetTimes {
				closures: Vec::new(),
				scopes: Vec::new(),
				frame: Raw::null(),
				time: 0,
				errors: Vec::new(),
			}
		}
		
		fn save_frame_state( &mut self ) -> (Raw<FrameAnalysis>, Time) {
			let state = (self.frame, self.time);
			self.frame = Raw::null();
			self.time = 0;
			state
		}
		
		fn restore_frame_state( &mut self, state: (Raw<FrameAnalysis>, Time) ) {
			let (frame, time) = state;
			self.frame = frame;
			self.time = time;
		}
		
		fn tick( &mut self ) -> Time {
			let moment = self.time;
			self.time += 1;
			moment
		}
		
		pub fn analyze_root( &mut self, root: &mut node::Root ) {
			
			self.frame = Raw::new( &root.frame );
			self.scopes.push( Raw::new( &root.scope ) );
			
			for statement in root.statements.mut_iter() {
				self.analyze_statement( *statement );
			}
		}
		
		fn analyze_statements(
			&mut self,
			scope: &mut ScopeAnalysis,
			statements: &mut Vec<Box<node::Statement>>
		) {
			self.scopes.push( Raw::new( scope ) );
			
			for statement in statements.mut_iter() {
				self.analyze_statement( *statement );
			}
			
			self.scopes.pop();
		}
		
		fn analyze_statement( &mut self, statement: &mut node::Statement ) {
			match *statement {
				
				node::If {
					test: ref mut test,
					scope: ref mut if_scope,
					block: ref mut if_block,
					else_if_clauses: ref mut else_if_clauses,
					else_clause: ref mut else_clause,
				} => {
					
					self.analyze_expression( *test );
					
					if_scope.start_at = self.tick();
					self.analyze_statements( if_scope, if_block );
					if_scope.end_at = self.tick();
					
					for else_if_clause in else_if_clauses.mut_iter() {
						
						self.analyze_expression( else_if_clause.test );
						
						else_if_clause.scope.start_at = self.tick();
						self.analyze_statements( &mut else_if_clause.scope, &mut else_if_clause.block );
						else_if_clause.scope.end_at = self.tick();
					}
					
					match *else_clause {
						Some( ref mut else_clause ) => {
							else_clause.scope.start_at = self.tick();
							self.analyze_statements( &mut else_clause.scope, &mut else_clause.block );
							else_clause.scope.end_at = self.tick();
						}
						None => {}
					};
				}
				
				node::While {
					test: ref mut test,
					scope: ref mut while_scope,
					block: ref mut while_block,
					else_clause: ref mut else_clause,
				} => {
					
					self.analyze_expression( *test );
					
					while_scope.is_loop = true;
					while_scope.start_at = self.tick();
					self.analyze_statements( while_scope, while_block );
					while_scope.end_at = self.tick();
					
					// Each operation of the frame's in-scope variables is duplicated at the end of the loop.
					// You can think of these as the repetition of the loop.
					for &scope in self.scopes.iter().rev() {
						let scope = scope.get();
						
						// Only variables declared in the current frame need to be corrected.
						// Usage and assignments of variables in other frames are summarized into bindings at the frame creation point.
						if scope.frame != self.frame {
							break;
						}
						
						for variable in scope.declared.iter() {
							let variable = variable.get();
							
							if variable.declared_in.get().frame == self.frame {
								
								let mut is_read = false;
								for read in variable.reads.iter() {
									if while_scope.start_at < read.time && read.time < while_scope.end_at {
										is_read = true;
										break;
									}
								}
								
								if is_read {
									variable.reads.push( ReadVariable {
										time: while_scope.end_at,
									} );
								}
								
								let mut is_written = false;
								for write in variable.writes.iter() {
									if while_scope.start_at < write.time && write.time < while_scope.end_at {
										is_written = true;
										break;
									}
								}
								
								if is_written {
									variable.writes.push( WriteVariable {
										time: while_scope.end_at,
									} );
								}
								
								let mut is_bound = false;
								let mut mutable = false;
								for bind in variable.root_binds.iter() {
									if while_scope.start_at < bind.time && bind.time < while_scope.end_at {
										is_bound = true;
										if bind.mutable {
											mutable = true;
											break;
										}
									}
								}
								
								if is_bound {
									variable.root_binds.push( BindVariable {
										time: while_scope.end_at,
										mutable: mutable,
									} );
									variable.n_binds += 1;
								}
							}
						}
					}
					
					match *else_clause {
						Some( ref mut else_clause ) => {
							else_clause.scope.start_at = self.tick();
							self.analyze_statements( &mut else_clause.scope, &mut else_clause.block );
							else_clause.scope.end_at = self.tick();
						}
						None => {}
					};
				}
				
				node::Try {
					scope: ref mut try_scope,
					block: ref mut try_block,
					catch_clauses: ref mut catch_clauses,
					else_clause: ref mut else_clause,
					finally_clause: ref mut finally_clause,
				} => {
					
					try_scope.start_at = self.tick();
					self.analyze_statements( try_scope, try_block );
					try_scope.end_at = self.tick();
					
					for catch_clause in catch_clauses.mut_iter() {
						
						match catch_clause.type_ {
							Some( ref mut expression ) => self.analyze_expression( *expression ),
							None => {},
						};
						
						catch_clause.scope.start_at = self.tick();
						catch_clause.variable.writes.push( WriteVariable { time: self.tick() } );
						self.analyze_statements( &mut catch_clause.scope, &mut catch_clause.block );
						catch_clause.scope.end_at = self.tick();
					}
					
					match *else_clause {
						Some( ref mut else_clause ) => {
							else_clause.scope.start_at = self.tick();
							self.analyze_statements( &mut else_clause.scope, &mut else_clause.block );
							else_clause.scope.end_at = self.tick();
						}
						None => {}
					};
					
					match *finally_clause {
						Some( ref mut finally_clause ) => {
							finally_clause.scope.start_at = self.tick();
							self.analyze_statements( &mut finally_clause.scope, &mut finally_clause.block );
							finally_clause.scope.end_at = self.tick();
						}
						None => {}
					};
				}
				
				node::Let {
					variable: ref mut variable,
					default: ref mut default,
					source_offset: _,
				} => {
					match *default {
						Some( ref mut expression ) => {
							self.analyze_expression( *expression );
							self.bind_variable( variable, true );
						},
						None => {},
					};
				}
				
				node::Print {
					expression: ref mut expression,
				} => {
					self.analyze_expression( *expression );
				}
				
				node::Return {
					expression: ref mut expression,
				} => {
					match *expression {
						Some( ref mut expression ) => {
							self.analyze_expression( *expression );
						}
						None => {}
					}
				}
				
				node::Throw {
					expression: ref mut expression,
				} => {
					self.analyze_expression( *expression );
				}
				
				node::Assignment {
					lvalue: ref mut lvalue,
					rvalue: ref mut rvalue,
				} => {
					self.analyze_lvalue_subexpressions( *lvalue );
					self.analyze_expression( *rvalue );
					self.analyze_lvalue( *lvalue );
				}
				
				node::ExpressionStatement {
					expression: ref mut expression,
				} => {
					self.analyze_expression( *expression );
				}
			}
		}
		
		fn analyze_expression( &mut self, expression: &mut node::Expression ) {
			match *expression {
				
				node::And { left: ref mut left, right: ref mut right }
				| node::Or { left: ref mut left, right: ref mut right }
				| node::Is { left: ref mut left, right: ref mut right }
				| node::Union { left: ref mut left, right: ref mut right }
				| node::Addition { left: ref mut left, right: ref mut right }
				| node::Subtraction { left: ref mut left, right: ref mut right }
				| node::Multiplication { left: ref mut left, right: ref mut right }
				| node::Division { left: ref mut left, right: ref mut right }
				=> {
					self.analyze_expression( *left );
					self.analyze_expression( *right );
				}
				
				node::Not { expression: ref mut expression }
				=> {
					self.analyze_expression( *expression );
				}
				
				node::DotAccess {
					expression: ref mut expression,
					..
				} => {
					self.analyze_expression( *expression );
				}
				
				node::ItemAccess {
					expression: ref mut expression,
					key_expression: ref mut key_expression,
				} => {
					self.analyze_expression( *expression );
					self.analyze_expression( *key_expression );
				}
				
				node::Call {
					expression: ref mut expression,
					arguments: ref mut arguments,
				} => {
					self.analyze_expression( *expression );
					for argument in arguments.mut_iter() {
						self.analyze_expression( *argument );
					}
				}
				
				node::Function {
					parameters: ref mut parameters,
					closure: ref mut closure,
					scope: ref mut scope,
					block: ref mut block,
					..
				} => {
					
					let state = self.save_frame_state();
					self.closures.push( Raw::new( closure ) );
					
					for parameter in parameters.mut_iter() {
						match parameter.type_ {
							Some( ref mut expression ) => self.analyze_expression( *expression ),
							None => {},
						}
						match parameter.default {
							Some( ref mut expression ) => self.analyze_expression( *expression ),
							None => {},
						}
					}
					
					self.scopes.push( Raw::new( scope ) );
					
					scope.start_at = self.tick();
					for statement in block.mut_iter() {
						self.analyze_statement( *statement );
					}
					scope.end_at = self.tick();
					
					self.scopes.pop();
					
					self.closures.pop();
					self.restore_frame_state( state );
				}
				
				node::Variable {
					name: name,
					source_offset: source_offset,
					analysis: ref mut analysis,
				} => {
					
					let variable = match self.find_variable( name ) {
						Some( variable_analysis ) => variable_analysis,
						None => {
							self.errors.push( AnalysisError {
								message: format!( "Variable not found: `${}`.", name ),
								source_offset: source_offset,
							} );
							return;
						}
					};
					
					*analysis = variable;
					self.bind_variable( variable.get(), false );
				}
				
				node::Name {..}
				| node::String {..}
				| node::Integer {..}
				| node::Float {..}
				| node::Boolean {..}
				| node::Nothing
				=> {}
			}
		}
		
		fn analyze_lvalue_subexpressions( &mut self, lvalue: &mut node::Lvalue ) {
			match *lvalue {
				
				node::VariableLvalue {..} => {}
				
			}
		}
		
		fn analyze_lvalue( &mut self, lvalue: &mut node::Lvalue ) {
			match *lvalue {
				
				node::VariableLvalue {
					name: name,
					analysis: ref mut analysis,
				} => {
					
					let variable = match self.find_variable( name ) {
						Some( variable_analysis ) => variable_analysis,
						None => {
							self.errors.push( AnalysisError {
								message: format!( "Variable not found: ${}.", name ),
								source_offset: 0, // TODO
							} );
							return;
						}
					};
					
					*analysis = variable;
					self.bind_variable( analysis.get(), true );
				}
				
			}
		}
		
		fn find_variable( &self, name: Identifier ) -> Option<Raw<VariableAnalysis>> {
			
			for scope in self.scopes.iter().rev() {
				for &variable in scope.get().declared.iter() {
					
					if variable.get().name == name {
						return Some( variable );
					}
				}
			}
			
			None
		}
		
		fn bind_variable( &mut self, variable: &mut VariableAnalysis, mutable: bool ) {
			
			let declaring_frame = variable.declared_in.get().frame.get();
			
			if declaring_frame == self.frame.get() {
				
				if mutable {
					variable.reads.push( ReadVariable { time: self.tick() } );
				} else {
					variable.writes.push( WriteVariable { time: self.tick() } );
				}
				
			} else {
				
				let mut time = 0;
				
				'closure_loop: for &closure in self.closures.iter().rev() {
				
					let closure = closure.get();
					
					if &closure.frame == declaring_frame {
						break;
					}
					
					time = closure.created_at;
					
					for binding in closure.bound.mut_iter() {
						
						if binding.variable.get() == variable {
						
							if mutable {
								return;
							} else {
								binding.mutable = true;
								continue 'closure_loop;
							}
						
						} else {
							continue;
						}
					}
					
					closure.bound.push( Binding {
						variable: Raw::new( variable ),
						mutable: mutable,
						storage_index: 0,
					} );
					variable.n_binds += 1;
				}
				
				for binding in variable.root_binds.mut_iter() {
					if binding.time == time {
						if ! binding.mutable {
							binding.mutable = mutable;
						}
						return;
					}
				}
				
				variable.root_binds.push( BindVariable {
					time: time,
					mutable: mutable,
				} );
				variable.n_binds += 1;
			}
		}
	}
