use error::AnalysisError;
use compile::analysis::{
	FrameAnalysis,
	VariableAnalysis,
	ReadVariable,
	WriteVariable,
	BindVariable,
	Binding,
	Time,
};
use parse::node;
use mem::raw::Raw;
use lang::identifier::Identifier;
use vm::repl::ReplState;

struct Scope {
	declared: Vec<Raw<VariableAnalysis>>,
}

pub struct AnalyzeVariables {
	frames: Vec<Raw<FrameAnalysis>>,
	scopes: Vec<Scope>,
	time: Time,
	pub errors: Vec<AnalysisError>,
}

	impl AnalyzeVariables {
		
		pub fn new() -> AnalyzeVariables {
			AnalyzeVariables {
				frames: Vec::new(),
				scopes: Vec::new(),
				time: 0,
				errors: Vec::new(),
			}
		}
		
		#[inline(always)]
		fn tick( &mut self ) -> Time {
			self.time += 1;
			self.time
		}
		
		#[inline(always)]
		fn push_frame( &mut self, frame: &mut FrameAnalysis ) {
			self.frames.push( Raw::new( frame ) );
		}
		
		#[inline(always)]
		fn pop_frame( &mut self ) {
			self.frames.pop();
		}
		
		#[inline(always)]
		fn push_scope( &mut self ) {
			self.scopes.push( Scope {
				declared: Vec::new(),
			} );
		}
		
		#[inline(always)]
		fn pop_scope( &mut self ) {
			self.scopes.pop();
		}
		
		#[inline(always)]
		fn get_current_scope<'l>( &'l mut self ) -> &'l mut Scope {
			self.scopes.mut_last().unwrap()
		}
		
		fn find_variable( &mut self, name: Identifier ) -> Result<Raw<VariableAnalysis>,()> {
			
			for scope in self.scopes.iter().rev() {
				for &variable in scope.declared.iter() {
					if variable.get().name == name {
						return Ok( variable );
					}
				}
			}
			
			Err(())
		}
		
		pub fn analyze_root( &mut self, root: &mut node::Root ) {
			self.push_frame( &mut root.frame );
			self.push_scope();
			self.find_declarations_in_block( &mut root.statements );
			self.analyze_block( &mut root.statements );
			self.pop_scope();
			self.pop_frame();
		}
		
		pub fn analyze_repl_root( &mut self, root: &mut node::Root, repl_state: &mut ReplState ) {
			
			self.push_frame( &mut root.frame );
			self.push_scope();
			
			// put repl_state vars into the root scope
			for &name in repl_state.variables.keys() {
				let mut var = box VariableAnalysis::new( name );
				var.declared_in = Raw::new( &root.frame );
				self.get_current_scope().declared.push( Raw::new( var ) );
				root.frame.declared.push( var );
			}
			
			self.find_declarations_in_block( &mut root.statements );
			self.analyze_block( &mut root.statements );
			
			// put any new vars into repl_state
			for var in self.get_current_scope().declared.iter().skip( repl_state.variables.len() ) {
				repl_state.declare_variable( var.get().name );
			}
			
			self.pop_scope();
			self.pop_frame();
		}
		
		fn find_declarations_in_block( &mut self, block: &mut Vec<Box<node::Statement>> ) {
			for statement in block.mut_iter() {
				self.find_declarations_in_statement( *statement );
			}
		}
		
		fn find_declarations_in_statement( &mut self, statement: &mut node::Statement ) {
			match *statement {
				
				node::Let {
					variable_name: name,
					variable: ref mut let_variable,
					default: _,
					source_offset: _,
				} => {
					
					let current_scope = self.scopes.mut_last().unwrap();
					
					for variable in current_scope.declared.iter() {
						if name == variable.get().name {
							fail!( "Double declaration" ); // TODO
						}
					}
					
					let mut variable = box VariableAnalysis::new( name );
					
					*let_variable = Raw::new( variable );
					
					current_scope.declared.push( Raw::new( variable ) );
					
					let current_frame = self.frames.mut_last().unwrap().get();
					variable.declared_in = Raw::new( current_frame );
					current_frame.declared.push( variable );
				}
				
				_ => {}
			}
		}
		
		fn analyze_block( &mut self, block: &mut Vec<Box<node::Statement>> ) {
			for statement in block.mut_iter() {
				self.analyze_statement( *statement );
			}
		}
		
		fn analyze_statement( &mut self, statement: &mut node::Statement ) {
			match *statement {
				
				node::ExpressionStatement { expression: ref mut expression }
				| node::Print { expression: ref mut expression }
				=> {
					self.analyze_expression( *expression );
				}
				
				node::Return { expression: ref mut optional_expression }
				=> {
					match *optional_expression {
						Some( ref mut expression ) => {
							self.analyze_expression( *expression );
						}
						None => {}
					}
				}
				
				node::Let {
					variable_name: _,
					variable: ref mut variable,
					default: ref mut default,
					source_offset: _,
				} => {
					match *default {
						Some( ref mut expression ) => {
							self.analyze_expression( *expression );
							self.write_variable( variable.get() );
						}
						None => {}
					};
				}
				
				node::Assignment {
					lvalue: ref mut lvalue,
					rvalue: ref mut rvalue,
				} => {
					self.analyze_lvalue_preparation( *lvalue );
					self.analyze_expression( *rvalue );
					self.analyze_lvalue_write( *lvalue );
				}
				
				node::If {
					test: ref mut if_test,
					block: ref mut if_block,
					else_if_clauses: ref mut else_if_clauses,
					else_clause: ref mut else_clause,
				} => {
					
					self.analyze_expression( *if_test );
					self.push_scope();
					self.find_declarations_in_block( if_block );
					self.analyze_block( if_block );
					self.pop_scope();
					
					for else_if_clause in else_if_clauses.mut_iter() {
						self.analyze_expression( else_if_clause.test );
						self.push_scope();
						self.find_declarations_in_block( &mut else_if_clause.block );
						self.analyze_block( &mut else_if_clause.block );
						self.pop_scope();
					}
					
					match *else_clause {
						Some( ref mut else_clause ) => {
							self.push_scope();
							self.find_declarations_in_block( &mut else_clause.block );
							self.analyze_block( &mut else_clause.block );
							self.pop_scope();
						}
						None => {}
					}
				}
				
				node::While {
					test: ref mut while_test,
					block: ref mut while_block,
					else_clause: ref mut else_clause,
				} => {
					
					self.analyze_expression( *while_test );
					self.push_scope();
					self.find_declarations_in_block( while_block );
					self.analyze_block( while_block );
					self.pop_scope();
					
					match *else_clause {
						Some( ref mut else_clause ) => {
							self.push_scope();
							self.find_declarations_in_block( &mut else_clause.block );
							self.analyze_block( &mut else_clause.block );
							self.pop_scope();
						}
						None => {}
					}
				}
				
				node::Try {
					block: ref mut try_block,
					catch_clauses: ref mut catch_clauses,
					else_clause: ref mut else_clause,
					finally_clause: ref mut finally_clause,
				} => {
					
					self.push_scope();
					self.find_declarations_in_block( try_block );
					self.analyze_block( try_block );
					self.pop_scope();
					
					for catch_clause in catch_clauses.mut_iter() {
						
						match catch_clause.type_ {
							Some( ref mut expression ) => {
								self.analyze_expression( *expression );
							}
							None => {}
						}
						
						self.push_scope();
						
						let mut variable = box VariableAnalysis::new( catch_clause.variable_name );
						variable.declared_in = *self.frames.last().unwrap();
						
						catch_clause.variable = Raw::new( variable );
						self.scopes.mut_last().unwrap().declared.push( Raw::new( variable ) );
						self.frames.mut_last().unwrap().get().declared.push( variable );
						
						self.find_declarations_in_block( &mut catch_clause.block );
						self.analyze_block( &mut catch_clause.block );
						
						self.pop_scope();
					}
					
					match *else_clause {
						Some( ref mut else_clause ) => {
							self.push_scope();
							self.find_declarations_in_block( &mut else_clause.block );
							self.analyze_block( &mut else_clause.block );
							self.pop_scope();
						}
						None => {}
					}
					
					match *finally_clause {
						Some( ref mut finally_clause ) => {
							self.push_scope();
							self.find_declarations_in_block( &mut finally_clause.block );
							self.analyze_block( &mut finally_clause.block );
							self.pop_scope();
						}
						None => {}
					}
				}
				
				_ => { fail!(); }
			}
		}
		
		fn analyze_expression( &mut self, expression: &mut node::Expression ) {
			match *expression {
				
				node::Nothing
				| node::Boolean {..}
				| node::Integer {..}
				| node::Float {..}
				| node::String {..}
				=> {}
				
				node::Variable {
					name: name,
					analysis: ref mut analysis,
					source_offset: source_offset,
				} => {
					match self.find_variable( name ) {
						Ok( variable_analysis ) => {
							*analysis = variable_analysis;
							self.read_variable( variable_analysis.get() );
						}
						Err(..) => {
							self.errors.push( AnalysisError {
								message: format!( "Variable not found: ${}", name ),
								source_offset: source_offset,
							} );
						}
					};
				}
				
				node::Name {..} => {}
				
				node::ItemAccess {
					expression: ref mut expression,
					key_expression: ref mut key_expression,
				} => {
					self.analyze_expression( *expression );
					self.analyze_expression( *key_expression );
				}
				
				node::DotAccess {
					expression: ref mut expression,
					name: _,
				} => {
					self.analyze_expression( *expression );
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
				
				node::Multiplication { left: ref mut left, right: ref mut right }
				| node::Division { left: ref mut left, right: ref mut right }
				| node::Addition { left: ref mut left, right: ref mut right }
				| node::Subtraction { left: ref mut left, right: ref mut right }
				| node::Union { left: ref mut left, right: ref mut right }
				| node::Is { left: ref mut left, right: ref mut right }
				| node::And { left: ref mut left, right: ref mut right }
				| node::Or { left: ref mut left, right: ref mut right }
				=> {
					self.analyze_expression( *left );
					self.analyze_expression( *right );
				}
				
				node::Not {
					expression: ref mut expression,
				} => {
					self.analyze_expression( *expression );
				}
				
				node::Function {
					parameters: ref mut parameters,
					frame: ref mut frame,
					block: ref mut block,
				} => {
					
					self.push_frame( frame );
					
					for parameter in parameters.mut_iter() {
						match parameter.type_ {
							Some( ref mut expression ) => {
								self.analyze_expression( *expression );
							}
							None => {},
						};
						match parameter.default {
							Some( ref mut expression ) => {
								self.analyze_expression( *expression );
							}
							None => {},
						};
					}
					
					self.push_scope();
					self.find_declarations_in_block( block );
					self.analyze_block( block );
					self.pop_scope();
					self.pop_frame();
				}
			}
		}
		
		fn analyze_lvalue_preparation( &mut self, lvalue: &mut node::Lvalue ) {
			match *lvalue {
				
				node::VariableLvalue {
					name: name,
					analysis: ref mut analysis,
				} => {
					match self.find_variable( name ) {
						Ok( variable_analysis ) => {
							*analysis = variable_analysis;
						}
						Err(..) => {
							self.errors.push( AnalysisError {
								message: format!( "Variable not found: ${}.", name ),
								source_offset: 0, // TODO
							} );
						}
					}
				}
			}
		}
		
		fn analyze_lvalue_write( &mut self, lvalue: &mut node::Lvalue ) {
			match *lvalue {
				
				node::VariableLvalue {
					name: _,
					analysis: ref mut analysis,
				} => {
					self.write_variable( analysis.get() );
				}
			}
		}
		
		fn read_variable( &mut self, variable: &mut VariableAnalysis ) {
			if self.frames.last().unwrap().get() == variable.declared_in.get() {
				variable.reads.push( ReadVariable { time: self.tick() } );
			} else {
				self.bind_variable( variable, false );
			}
		}
		
		fn write_variable( &mut self, variable: &mut VariableAnalysis ) {
			if self.frames.last().unwrap().get() == variable.declared_in.get() {
				variable.writes.push( WriteVariable { time: self.tick() } );
			} else {
				self.bind_variable( variable, true );
			}
		}
		
		fn bind_variable( &mut self, variable: &mut VariableAnalysis, mutable: bool ) {
			
			let mut time = 0;
			
			'frame_loop: for &frame in self.frames.iter().rev() {
				
				if frame == variable.declared_in {
					break;
				}
				
				let frame = frame.get();
				let closure = frame.closure.as_mut().unwrap();
				
				time = closure.created_at;
				
				for binding in closure.bindings.mut_iter() {
					
					if binding.variable.get() == variable {
						
						if ! mutable {
							return;
						} else {
							binding.mutable = true;
							continue 'frame_loop;
						}
					}
				}
				
				closure.bindings.push( Binding {
					variable: Raw::new( variable ),
					mutable: mutable,
					storage_index: 0,
				} );
				variable.n_binds += 1;
			}
			
			for binding in variable.root_binds.mut_iter() {
				if binding.time == time {
					binding.mutable = binding.mutable || mutable;
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
