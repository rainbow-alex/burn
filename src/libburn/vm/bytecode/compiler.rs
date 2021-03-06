use std::mem;
use mem::raw::Raw;
use mem::rc::Rc;
use parse::{parser, node};
use lang::origin::Origin;
use lang::function;
use lang::value;
use vm::error::Error;
use vm::bytecode::code::Code;
use vm::bytecode::opcode;
use vm::analysis::annotation;
use vm::analysis::resolution::AnalyzeResolution;
use vm::analysis::allocation::AnalyzeAllocation;
use vm::run::frame;
use vm::repl;

pub fn compile(
	origin: Rc<Box<Origin>>,
	mut repl_state: Option<&mut repl::State>,
	source_code: &str
) -> Result<frame::Frame,Vec<Box<Error>>> {
	
	let mut ast = match parser::parse( &origin, source_code ) {
		Ok( ast ) => ast,
		Err( error ) => {
			return Err( vec!( box error as Box<Error> ) );
		}
	};
	
	{
		let mut pass = AnalyzeResolution::new( &origin );
		pass.analyze_root( &mut ast, &mut repl_state );
		if pass.errors.len() > 0 {
			return Err( pass.errors );
		}
	}
	
	let mut pass = AnalyzeAllocation::new();
	pass.analyze_root( &mut ast, &mut repl_state );
	
	let code = {
		let mut compilation = Compilation::new();
		compilation.compile_root( &mut ast );
		compilation.code
	};
	
	debug!( { code.dump(); } )
	
	let locals = Vec::from_elem( code.n_local_variables, value::Nothing );
	let mut shared = Vec::from_elem( code.n_shared_local_variables, None );
	
	repl_state.map( |repl_state| {
		for variable in ast.frame.declared_variables.iter().take( repl_state.variables.len() ) {
			let repl_var = repl_state.variables.find( &variable.name ).unwrap().clone();
			*shared.get_mut( variable.local_storage_index ) = Some( repl_var );
		}
	} );
	
	Ok( frame::BurnRootFrame {
		origin: origin,
		code: code,
		context: frame::BurnContext::new( locals, shared ),
	} )
}

struct Compilation {
	code: Box<Code>,
	frames: Vec<Raw<annotation::Frame>>,
}

	type Placeholder = uint;
	
	impl Compilation {
		
		fn new() -> Compilation {
			Compilation {
				code: box Code::new(),
				frames: Vec::new(),
			}
		}
		
		fn get_current_frame<'l>( &'l self ) -> Raw<annotation::Frame> {
			*self.frames.last().unwrap()
		}
		
		fn find_bound_storage_index( &self, variable: Raw<annotation::Variable> ) -> uint {
			self.get_current_frame().get_closure()
				.bindings.iter().find( |b| { b.variable == variable } ).unwrap()
				.storage_index
		}
		
		fn create_placeholder( &mut self ) -> Placeholder {
			let offset = self.code.opcodes.len();
			self.code.opcodes.push( opcode::Nop );
			offset
		}
		
		fn fill_in_placeholder( &mut self, offset: Placeholder, opcode: opcode::OpCode ) {
			*self.code.opcodes.get_mut( offset ) = opcode;
		}
		
		fn compile_root( &mut self, root: &mut node::Root ) {
			
			self.frames.push( Raw::new( &root.frame ) );
			
			self.code.n_local_variables = root.frame.n_local_variables;
			self.code.n_shared_local_variables = root.frame.n_shared_local_variables;
			
			for statement in root.statements.mut_iter() {
				self.compile_statement( *statement );
			}
			self.code.opcodes.push( opcode::ReturnNothing );
			
			self.frames.pop();
		}
		
		fn compile_function( &mut self, frame: &annotation::Frame, block: &mut [Box<node::Statement>] ) {
			
			self.frames.push( Raw::new( frame ) );
			
			self.code.n_local_variables = frame.n_local_variables;
			self.code.n_shared_local_variables = frame.n_shared_local_variables;
			
			for statement in block.mut_iter() {
				self.compile_statement( *statement );
			}
			self.code.opcodes.push( opcode::ReturnNothing );
			
			self.frames.pop();
		}
		
		fn compile_statement( &mut self, statement: &mut node::Statement ) {
			match *statement {
				
				node::Use {
					path: ref path,
					annotation: ref mut annotation,
				} => {
					let operation = box ::lang::module::Use::new( path.clone() );
					annotation.operation = Raw::new( operation );
					self.code.opcodes.push( opcode::Use { operation: Raw::new( operation ) } );
					unsafe { mem::forget( operation ); }
				}
				
				node::ExpressionStatement {
					expression: ref mut expression,
				} => {
					self.compile_expression( *expression );
					self.code.opcodes.push( opcode::Pop );
				}
				
				node::Assignment {
					lvalue: ref mut lvalue,
					rvalue: ref mut rvalue,
				} => {
					
					match **lvalue {
						
						node::VariableLvalue {
							name: _,
							annotation: variable,
							source_offset: _,
						} => {
							
							self.compile_expression( *rvalue );
							
							if variable.declared_in == self.get_current_frame() {
								
								match variable.local_storage_type {
									annotation::storage::Local => {
										self.code.opcodes.push(
											opcode::StoreLocal( variable.local_storage_index )
										);
									}
									annotation::storage::SharedLocal => {
										self.code.opcodes.push(
											opcode::StoreSharedLocal( variable.local_storage_index )
										);
									}
								};
								
							} else {
								
								let bound_storage_index = self.find_bound_storage_index( variable );
								
								match variable.bound_storage_type {
									annotation::storage::StaticBound => {
										self.code.opcodes.push(
											opcode::StoreStaticBound( bound_storage_index )
										);
									}
									annotation::storage::SharedBound => {
										self.code.opcodes.push(
											opcode::StoreSharedBound( bound_storage_index )
										);
									}
								};
							}
						}
						
						node::DotAccessLvalue {
							expression: ref mut expression,
							name: name,
						} => {
							self.compile_expression( *expression );
							self.compile_expression( *rvalue );
							self.code.opcodes.push( opcode::SetProperty { name: name } );
						}
					}
				}
				
				node::Let {
					variable_offset: _,
					variable_name: _,
					annotation: ref annotation,
					default: ref mut default,
				} => {
					
					if default.is_some() {
						
						self.compile_expression( *default.as_mut().unwrap() );
						
						match annotation.local_storage_type {
							annotation::storage::Local => {
								self.code.opcodes.push(
									opcode::StoreLocal( annotation.local_storage_index )
								);
							}
							annotation::storage::SharedLocal => {
								self.code.opcodes.push(
									opcode::InitializeSharedLocal( annotation.local_storage_index )
								);
								self.code.opcodes.push(
									opcode::StoreSharedLocal( annotation.local_storage_index )
								);
							}
						};
						
					} else {
						
						match annotation.local_storage_type {
							annotation::storage::SharedLocal => {
								self.code.opcodes.push(
									opcode::InitializeSharedLocal( annotation.local_storage_index )
								);
							}
							_ => {}
						};
					}
				}
				
				node::Print {
					expression: ref mut expression,
				} => {
					self.compile_expression( *expression );
					self.code.opcodes.push( opcode::ToString );
					self.code.opcodes.push( opcode::Print );
				}
				
				node::Return {
					expression: ref mut expression,
				} => {
					match *expression {
						Some( ref mut expression ) => {
							self.compile_expression( *expression );
						}
						None => {
							self.code.opcodes.push( opcode::PushNothing );
						}
					};
					self.code.opcodes.push( opcode::Return );
				}
				
				node::Throw {
					expression: ref mut expression,
				} => {
					self.compile_expression( *expression );
					self.code.opcodes.push( opcode::Throw );
				}
				
				node::If {
					test: ref mut test,
					block: ref mut if_block,
					else_if_clauses: ref mut else_if_clauses,
					else_clause: ref mut else_clause,
				} => {
					
					let has_else_if_clauses = else_if_clauses.len() > 0;
					let has_else_clause = else_clause.is_some();
					
					let mut jump_else;
					let mut jump_end = Vec::<Placeholder>::new();
					
					self.compile_expression( *test );
					jump_else = self.create_placeholder();
					
					for statement in if_block.mut_iter() {
						self.compile_statement( *statement );
					}
					
					if has_else_if_clauses || has_else_clause {
						jump_end.push( self.create_placeholder() );
					}
					
					let last_i = else_if_clauses.len() - 1;
					for (i, else_if_clause) in else_if_clauses.mut_iter().enumerate() {
						
						let is_last = ( i == last_i );
						
						let jump = opcode::JumpIfPopFalsy { instruction: self.code.opcodes.len() };
						self.fill_in_placeholder( jump_else, jump );
						
						self.compile_expression( else_if_clause.test );
						jump_else = self.create_placeholder();
						
						for statement in else_if_clause.block.mut_iter() {
							self.compile_statement( *statement );
						}
						
						if ! is_last || has_else_clause {
							jump_end.push( self.create_placeholder() );
						}
					}
					
					let jump = opcode::JumpIfPopFalsy { instruction: self.code.opcodes.len() };
					self.fill_in_placeholder( jump_else, jump );
					
					if has_else_clause {
						for statement in else_clause.as_mut().unwrap().block.mut_iter() {
							self.compile_statement( *statement );
						}
					}
					
					let jump = opcode::Jump { instruction: self.code.opcodes.len() };
					for &placeholder in jump_end.iter() {
						self.fill_in_placeholder( placeholder, jump );
					}
				}
				
				node::While {
					test: ref mut while_test,
					block: ref mut while_block,
					else_clause: ref mut else_clause,
				} => {
					
					let start = self.code.opcodes.len();
					
					self.compile_expression( *while_test );
					let test_opcode = self.create_placeholder();
					
					for statement in while_block.mut_iter() {
						self.compile_statement( *statement );
					}
					
					self.code.opcodes.push( opcode::Jump { instruction: start } );
					
					let jump = opcode::JumpIfPopFalsy { instruction: self.code.opcodes.len() };
					self.fill_in_placeholder( test_opcode, jump );
					
					if else_clause.is_some() {
						unimplemented!();
					}
				}
				
				node::Try {
					block: ref mut try_block,
					catch_clauses: ref mut catch_clauses,
					else_clause: ref mut else_clause,
					finally_clause: ref mut finally_clause,
				} => {
					
					let has_catch_clauses = catch_clauses.len() > 0;
					let has_else_clause = else_clause.is_some();
					let has_finally_clause = finally_clause.is_some();
					
					// TRY
					
					let push_finally = if has_finally_clause {
						Some( self.create_placeholder() )
					} else {
						None
					};
					
					let push_catch = if has_catch_clauses {
						Some( self.create_placeholder() )
					} else {
						None
					};
					
					for statement in try_block.mut_iter() {
						self.compile_statement( *statement );
					}
					
					let mut end_catch = Vec::<Placeholder>::new();
					
					if has_catch_clauses {
						
						self.code.opcodes.push( opcode::PopFlowPoint );
						
						let end_try_jump = self.create_placeholder();
						
						let opcode = opcode::PushStartCatchFlowPoint { instruction: self.code.opcodes.len() };
						self.fill_in_placeholder( push_catch.unwrap(), opcode );
						
						for catch_clause in catch_clauses.mut_iter() {
							
							let has_type = catch_clause.type_.is_some();
							let variable = catch_clause.variable;
							
							if has_type {
								self.compile_expression( *catch_clause.type_.as_mut().unwrap() );
								self.code.opcodes.push( opcode::ThrownIs );
							}
							
							let catch = self.create_placeholder();
							
							for statement in catch_clause.block.mut_iter() {
								self.compile_statement( *statement );
							}
							
							end_catch.push( self.create_placeholder() );
							
							let opcode = if has_type {
								
								match variable.local_storage_type {
									annotation::storage::Local => {
										opcode::CatchLocalOrJump {
											storage_index: variable.local_storage_index,
											instruction: self.code.opcodes.len(),
										}
									}
									annotation::storage::SharedLocal => {
										opcode::CatchSharedLocalOrJump {
											storage_index: variable.local_storage_index,
											instruction: self.code.opcodes.len(),
										}
									}
								}
								
							} else {
								
								match variable.local_storage_type {
									annotation::storage::Local => {
										opcode::CatchLocal {
											storage_index: variable.local_storage_index,
										}
									}
									annotation::storage::SharedLocal => {
										opcode::CatchSharedLocal {
											storage_index: variable.local_storage_index,
										}
									}
								}
								
							};
							
							self.fill_in_placeholder( catch, opcode );
						}
						
						self.code.opcodes.push( opcode::Rethrow );
						
						let jump = opcode::Jump { instruction: self.code.opcodes.len() };
						self.fill_in_placeholder( end_try_jump, jump );
					}
					
					if has_else_clause {
						
						for statement in else_clause.as_mut().unwrap().block.mut_iter() {
							self.compile_statement( *statement );
						}
					}
					
					let jump = opcode::Jump { instruction: self.code.opcodes.len() };
					for placeholder in end_catch.move_iter() {
						self.fill_in_placeholder( placeholder, jump );
					}
					
					if has_finally_clause {
						
						self.code.opcodes.push( opcode::StartFinally );
						
						let opcode = opcode::PushStartFinallyFlowPoint { instruction: self.code.opcodes.len() };
						self.fill_in_placeholder( push_finally.unwrap(), opcode );
						
						for statement in finally_clause.as_mut().unwrap().block.mut_iter() {
							self.compile_statement( *statement );
						}
						
						self.code.opcodes.push( opcode::EndFinally );
					}
				}
			}
		}
		
		fn compile_expression( &mut self, expression: &mut node::Expression ) {
			match *expression {
				
				node::Nothing => {
					self.code.opcodes.push( opcode::PushNothing );
				}
				
				node::Boolean {
					value: b,
				} => {
					self.code.opcodes.push( opcode::PushBoolean { value: b } );
				}
				
				node::Integer {
					value: i,
				} => {
					self.code.opcodes.push( opcode::PushInteger { value: i } );
				}
				
				node::Float {
					value: f,
				} => {
					self.code.opcodes.push( opcode::PushFloat { value: f } );
				}
				
				node::String {
					value: ref value,
				} => {
					self.code.opcodes.push( opcode::PushString { index: self.code.strings.len() } );
					self.code.strings.push( Rc::new( value.clone() ) );
				}
				
				node::Variable {
					name: _,
					annotation: variable,
					source_offset: _,
				} => {
					
					if variable.declared_in == self.get_current_frame() {
						
						match variable.local_storage_type {
							annotation::storage::Local => {
								self.code.opcodes.push(
									opcode::LoadLocal( variable.local_storage_index )
								);
							}
							annotation::storage::SharedLocal => {
								self.code.opcodes.push(
									opcode::LoadSharedLocal( variable.local_storage_index )
								);
							}
						}
						
					} else {
						
						let bound_storage_index = self.find_bound_storage_index( variable );
						
						match variable.bound_storage_type {
							annotation::storage::StaticBound => {
								self.code.opcodes.push(
									opcode::LoadStaticBound( bound_storage_index )
								);
							}
							annotation::storage::SharedBound => {
								self.code.opcodes.push(
									opcode::LoadSharedBound( bound_storage_index )
								);
							}
						};
					}
				}
				
				node::Name {
					identifier: identifier,
					annotation: annotation,
				} => {
					match annotation.resolution {
						annotation::Implicit => {
							self.code.opcodes.push( opcode::LoadImplicit { name: identifier } );
						}
						annotation::Use( mut use_annotation ) => {
							use_annotation.operation.add_inline( Raw::new( self.code ), self.code.opcodes.len() );
							self.code.opcodes.push( opcode::Fail );
						}
					};
				}
				
				node::DotAccess {
					expression: ref mut expression,
					name: name,
				} => {
					self.compile_expression( *expression );
					self.code.opcodes.push( opcode::GetProperty { name: name } );
				}
				
				node::ItemAccess {
					expression: ref mut expression,
					key_expression: ref mut key_expression,
				} => {
					self.compile_expression( *expression );
					self.compile_expression( *key_expression );
					self.code.opcodes.push( opcode::GetItem );
				}
				
				node::Call {
					expression: ref mut expression,
					arguments: ref mut arguments,
				} => {
					
					self.compile_expression( *expression );
					
					for argument in arguments.mut_iter() {
						self.compile_expression( *argument );
					}
					
					self.code.opcodes.push( opcode::Call { n_arguments: arguments.len() } );
				}
				
				node::Addition {
					left: ref mut left,
					right: ref mut right,
				} => {
					self.compile_expression( *left );
					self.compile_expression( *right );
					self.code.opcodes.push( opcode::Add );
				}
				
				node::Subtraction {
					left: ref mut left,
					right: ref mut right,
				} => {
					self.compile_expression( *left );
					self.compile_expression( *right );
					self.code.opcodes.push( opcode::Subtract );
				}
				
				node::Multiplication {
					left: ref mut left,
					right: ref mut right,
				} => {
					self.compile_expression( *left );
					self.compile_expression( *right );
					self.code.opcodes.push( opcode::Multiply );
				}
				
				node::Division {
					left: ref mut left,
					right: ref mut right,
				} => {
					self.compile_expression( *left );
					self.compile_expression( *right );
					self.code.opcodes.push( opcode::Divide );
				}
				
				node::Union {
					left: ref mut left,
					right: ref mut right,
				} => {
					self.compile_expression( *left );
					self.compile_expression( *right );
					self.code.opcodes.push( opcode::Union );
				}
				
				node::Is {
					left: ref mut left,
					right: ref mut right,
				} => {
					self.compile_expression( *left );
					self.compile_expression( *right );
					self.code.opcodes.push( opcode::Is );
				}
				
				node::Eq {
					left: ref mut left,
					right: ref mut right,
				} => {
					self.compile_expression( *left );
					self.compile_expression( *right );
					self.code.opcodes.push( opcode::Eq );
				}
				
				node::Neq {
					left: ref mut left,
					right: ref mut right,
				} => {
					self.compile_expression( *left );
					self.compile_expression( *right );
					self.code.opcodes.push( opcode::Neq );
				}
				
				node::Lt {
					left: ref mut left,
					right: ref mut right,
				} => {
					self.compile_expression( *left );
					self.compile_expression( *right );
					self.code.opcodes.push( opcode::Lt );
				}
				
				node::Gt {
					left: ref mut left,
					right: ref mut right,
				} => {
					self.compile_expression( *left );
					self.compile_expression( *right );
					self.code.opcodes.push( opcode::Gt );
				}
				
				node::LtEq {
					left: ref mut left,
					right: ref mut right,
				} => {
					self.compile_expression( *left );
					self.compile_expression( *right );
					self.code.opcodes.push( opcode::LtEq );
				}
				
				node::GtEq {
					left: ref mut left,
					right: ref mut right,
				} => {
					self.compile_expression( *left );
					self.compile_expression( *right );
					self.code.opcodes.push( opcode::GtEq );
				}
				
				node::Not {
					expression: ref mut expression,
				} => {
					self.compile_expression( *expression );
					self.code.opcodes.push( opcode::Not );
				}
				
				node::And {
					left: ref mut left,
					right: ref mut right,
				} => {
					self.compile_expression( *left );
					let placeholder = self.create_placeholder();
					self.compile_expression( *right );
					self.fill_in_placeholder( placeholder, opcode::ShortCircuitAnd );
				}
				
				node::Or {
					left: ref mut left,
					right: ref mut right,
				} => {
					self.compile_expression( *left );
					let placeholder = self.create_placeholder();
					self.compile_expression( *right );
					self.fill_in_placeholder( placeholder, opcode::ShortCircuitOr );
				}
				
				node::Function {
					parameters: ref parameters,
					frame: ref frame,
					block: ref mut block,
				} => {
					
					let mut compilation = Compilation::new();
					compilation.compile_function( frame, block.as_mut_slice() );
					let code = compilation.code;
					
					let mut parameter_definitions = Vec::<function::FunctionParameterDefinition>::new();
					for parameter in parameters.iter() {
						let variable = parameter.variable;
						match variable.local_storage_type {
							annotation::storage::Local => {
								parameter_definitions.push( function::FunctionParameterDefinition {
									name: variable.name,
									storage: function::LocalFunctionParameterStorage( variable.local_storage_index ),
								} );
							}
							annotation::storage::SharedLocal => {
								parameter_definitions.push( function::FunctionParameterDefinition {
									name: variable.name,
									storage: function::SharedLocalFunctionParameterStorage( variable.local_storage_index ),
								} );
							}
						};
					}
					
					let mut binding_definitions = Vec::<function::FunctionBindingDefinition>::new();
					for binding in frame.closure.as_ref().unwrap().bindings.iter() {
						let variable = binding.variable;
						
						// local to bound
						if variable.declared_in == self.get_current_frame() {
							
							match variable.bound_storage_type {
								annotation::storage::StaticBound => {
									binding_definitions.push( function::LocalToStaticBoundBinding(
										variable.local_storage_index,
										binding.storage_index
									) );
								}
								annotation::storage::SharedBound => {
									binding_definitions.push( function::SharedLocalToSharedBoundBinding(
										variable.local_storage_index,
										binding.storage_index
									) );
								}
							};
							
						// bound to bound
						} else {
							
							let current_bound_storage_index = self.find_bound_storage_index( variable );
							
							match variable.bound_storage_type {
								annotation::storage::StaticBound => {
									binding_definitions.push( function::StaticBoundToStaticBoundBinding(
										current_bound_storage_index,
										binding.storage_index
									) );
								}
								annotation::storage::SharedBound => {
									binding_definitions.push( function::SharedBoundToSharedBoundBinding(
										current_bound_storage_index,
										binding.storage_index
									) );
								}
							};
						}
					}
					
					let definition = Rc::new( function::FunctionDefinition::new(
						code,
						parameter_definitions,
						binding_definitions
					) );
					
					self.code.opcodes.push( opcode::PushFunction { index: self.code.functions.len() } );
					self.code.functions.push( definition );
				}
			}
		}
	}
