use mem::raw::Raw;
use lang::identifier::Identifier;
use parse::node;
use vm::error::AnalysisError;
use vm::analysis::annotation;
use vm::repl;

struct Scope {
	frame: Raw<annotation::Frame>,
	declared_variables: Vec<Raw<annotation::Variable>>,
	used: Vec<Raw<annotation::Use>>,
}

pub struct AnalyzeResolution {
	frames: Vec<Raw<annotation::Frame>>,
	scopes: Vec<Scope>,
	time: annotation::Time,
	pub errors: Vec<AnalysisError>,
}

	impl AnalyzeResolution {
		
		pub fn new() -> AnalyzeResolution {
			AnalyzeResolution {
				frames: Vec::new(),
				scopes: Vec::new(),
				time: 0,
				errors: Vec::new(),
			}
		}
		
		fn tick( &mut self ) -> annotation::Time {
			self.time += 1;
			self.time
		}
		
		fn push_frame( &mut self, frame: &mut annotation::Frame ) {
			self.frames.push( Raw::new( frame ) );
		}
		
		fn pop_frame( &mut self ) {
			self.frames.pop();
		}
		
		fn push_scope( &mut self ) {
			let frame_ptr = self.get_current_frame();
			self.scopes.push( Scope {
				frame: frame_ptr,
				declared_variables: Vec::new(),
				used: Vec::new(),
			} );
		}
		
		fn pop_scope( &mut self ) {
			self.scopes.pop();
		}
		
		fn get_current_scope<'l>( &'l mut self ) -> &'l mut Scope {
			self.scopes.mut_last().unwrap()
		}
		
		fn get_current_frame<'l>( &'l self ) -> Raw<annotation::Frame> {
			*self.frames.last().unwrap()
		}
		
		fn declare_variable( &mut self, name: Identifier ) -> Raw<annotation::Variable> {
			
			let mut variable = box annotation::Variable::new( name );
			let ptr = Raw::new( variable );
			
			variable.declared_in = self.get_current_frame();
			self.get_current_scope().declared_variables.push( ptr );
			self.get_current_frame().declared_variables.push( variable );
			
			ptr
		}
		
		fn find_variable( &mut self, name: Identifier ) -> Result<Raw<annotation::Variable>,()> {
			
			for scope in self.scopes.iter().rev() {
				for &variable in scope.declared_variables.iter() {
					if variable.name == name {
						return Ok( variable );
					}
				}
			}
			
			Err( () )
		}
		
		fn find_name( &mut self, name: Identifier ) -> annotation::NameResolution {
			
			for scope in self.scopes.iter().rev() {
				for &use_ in scope.used.iter() {
					if use_.name == name {
						return annotation::Use( use_ );
					}
				}
			}
			
			annotation::Implicit
		}
		
		pub fn analyze_root( &mut self, root: &mut node::Root ) {
			
			self.push_frame( &mut root.frame );
			self.push_scope();
			
			self.analyze_block( &mut root.statements );
			
			self.pop_scope();
			self.pop_frame();
		}
		
		pub fn analyze_repl_root( &mut self, root: &mut node::Root, repl_state: &mut repl::State ) {
			
			self.push_frame( &mut root.frame );
			self.push_scope();
			
			// put repl_state vars into the root scope
			for &name in repl_state.variables.keys() {
				self.declare_variable( name );
			}
			
			self.analyze_block( &mut root.statements );
			
			// put any new vars into repl_state
			{
				let declared_variables = self.get_current_scope().declared_variables.iter();
				let mut new_variables = declared_variables.skip( repl_state.variables.len() );
				for variable in new_variables {
					repl_state.declare_variable( variable.name );
				}
			}
			
			self.pop_scope();
			self.pop_frame();
		}
		
		fn analyze_block( &mut self, block: &mut Vec<Box<node::Statement>> ) {
			for statement in block.mut_iter() {
				self.analyze_statement( *statement );
			}
		}
		
		fn analyze_statement( &mut self, statement: &mut node::Statement ) {
			match *statement {
				
				node::Use {
					path: _,
					annotation: ref mut annotation,
				} => {
					// todo! check for duplicate use
					self.get_current_scope().used.push( Raw::new( annotation ) );
				}
				
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
					variable_name: name,
					annotation: ref mut annotation,
					default: ref mut default,
					source_offset: source_offset,
				} => {
					
					let is_duplicate = self.get_current_scope().declared_variables.iter()
						.find( |v| { v.name == name } ).is_some();
					
					if is_duplicate {
						self.errors.push( AnalysisError {
							message: format!( "Duplicate declaration of ${}", name ),
							source_offset: source_offset,
						} );
					}
					
					*annotation = self.declare_variable( name );
					
					match *default {
						Some( ref mut expression ) => {
							self.analyze_expression( *expression );
							self.write_variable( *annotation );
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
					self.analyze_block( if_block );
					self.pop_scope();
					
					for else_if_clause in else_if_clauses.mut_iter() {
						self.analyze_expression( else_if_clause.test );
						self.push_scope();
						self.analyze_block( &mut else_if_clause.block );
						self.pop_scope();
					}
					
					match *else_clause {
						Some( ref mut else_clause ) => {
							self.push_scope();
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
					let start = self.tick();
					self.analyze_block( while_block );
					let end = self.tick();
					self.repeat_variable_usages( start, end );
					self.pop_scope();
					
					match *else_clause {
						Some( ref mut else_clause ) => {
							self.push_scope();
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
						
						catch_clause.variable = self.declare_variable( catch_clause.variable_name );
						self.analyze_block( &mut catch_clause.block );
						
						self.pop_scope();
					}
					
					match *else_clause {
						Some( ref mut else_clause ) => {
							self.push_scope();
							self.analyze_block( &mut else_clause.block );
							self.pop_scope();
						}
						None => {}
					}
					
					match *finally_clause {
						Some( ref mut finally_clause ) => {
							self.push_scope();
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
			
			let expression_ptr = Raw::new( expression );
			
			match *expression {
				
				node::Nothing
				| node::Boolean {..}
				| node::Integer {..}
				| node::Float {..}
				| node::String {..}
				=> {}
				
				node::Variable {
					name: name,
					annotation: ref mut annotation,
					source_offset: source_offset,
				} => {
					match self.find_variable( name ) {
						Ok( variable ) => {
							*annotation = variable;
							self.read_variable( variable );
						}
						Err(..) => {
							self.errors.push( AnalysisError {
								message: format!( "Variable not found: ${}", name ),
								source_offset: source_offset,
							} );
						}
					};
				}
				
				node::Name {
					identifier: identifier,
					annotation: ref mut annotation,
				} => {
					annotation.resolution = self.find_name( identifier );
				}
				
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
				| node::Eq { left: ref mut left, right: ref mut right }
				| node::Neq { left: ref mut left, right: ref mut right }
				| node::Lt { left: ref mut left, right: ref mut right }
				| node::Gt { left: ref mut left, right: ref mut right }
				| node::LtEq { left: ref mut left, right: ref mut right }
				| node::GtEq { left: ref mut left, right: ref mut right }
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
					
					frame.get_closure().created_at = self.tick();
					
					self.get_current_frame().functions.push( expression_ptr );
					
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
					for parameter in parameters.mut_iter() {
						parameter.variable = self.declare_variable( parameter.variable_name );
					}
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
					annotation: ref mut annotation,
					source_offset: source_offset,
				} => {
					match self.find_variable( name ) {
						Ok( variable ) => {
							*annotation = variable;
						}
						Err(..) => {
							self.errors.push( AnalysisError {
								message: format!( "Variable not found: ${}.", name ),
								source_offset: source_offset,
							} );
						}
					}
				}
				
				node::DotAccessLvalue {
					expression: ref mut expression,
					name: _,
				} => {
					self.analyze_expression( *expression );
				}
			}
		}
		
		fn analyze_lvalue_write( &mut self, lvalue: &mut node::Lvalue ) {
			match *lvalue {
				
				node::VariableLvalue {
					name: _,
					annotation: annotation,
					source_offset: _,
				} => {
					// the variable might not have been found
					if ! annotation.is_null() {
						self.write_variable( annotation );
					}
				}
				
				node::DotAccessLvalue {..} => {}
			}
		}
		
		fn read_variable( &mut self, mut variable: Raw<annotation::Variable> ) {
			if self.get_current_frame() == variable.declared_in {
				variable.reads.push( annotation::ReadVariable { time: self.tick() } );
			} else {
				self.bind_variable( variable, false );
			}
		}
		
		fn write_variable( &mut self, mut variable: Raw<annotation::Variable> ) {
			if self.get_current_frame() == variable.declared_in {
				variable.writes.push( annotation::WriteVariable { time: self.tick() } );
			} else {
				self.bind_variable( variable, true );
			}
		}
		
		fn bind_variable( &mut self, mut variable: Raw<annotation::Variable>, mutable: bool ) {
			
			let mut time = 0;
			
			'frame_loop: for &mut frame in self.frames.iter().rev() {
				
				if frame == variable.declared_in {
					break;
				}
				
				time = frame.get_closure().created_at;
				
				for binding in frame.get_closure().bindings.mut_iter() {
					
					if binding.variable == variable {
						
						if ! mutable {
							return;
						} else {
							binding.mutable = true;
							continue 'frame_loop;
						}
					}
				}
				
				frame.get_closure().bindings.push( annotation::Binding {
					variable: variable,
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
			
			variable.root_binds.push( annotation::BindVariable {
				time: time,
				mutable: mutable,
			} );
			variable.n_binds += 1;
		}
		
		fn repeat_variable_usages( &mut self, from: annotation::Time, to: annotation::Time ) {
			
			let current_frame = self.get_current_frame();
			
			for scope in self.scopes.iter().rev() {
				
				if scope.frame != current_frame {
					break;
				}
				
				for &mut variable in scope.declared_variables.iter() {
					
					let is_read = variable.reads.iter()
						.find( |r| { from < r.time && r.time < to } ).is_some();
					
					if is_read {
						variable.reads.push( annotation::ReadVariable {
							time: to,
						} );
					}
					
					let is_written = variable.writes.iter()
						.find( |w| { from < w.time && w.time < to } ).is_some();
					
					if is_written {
						variable.writes.push( annotation::WriteVariable {
							time: to,
						} );
					}
					
					let mut is_bound = false;
					let mut is_bound_mutable = false;
					for bind in variable.root_binds.iter() {
						if from < bind.time && bind.time < to {
							is_bound = true;
							if bind.mutable {
								is_bound_mutable = true;
								break;
							}
						}
					}
					
					if is_bound {
						variable.root_binds.push( annotation::BindVariable {
							time: to,
							mutable: is_bound_mutable,
						} );
						variable.n_binds += 1;
					}
				}
			}
		}
	}
