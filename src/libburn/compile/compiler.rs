use error::AnalysisError;
use parse::node;
use lang::function::FunctionDefinition;
use lang::string::String;
use mem::raw::Raw;
use mem::rc::Rc;
use vm::code::Code;
use vm::opcode;
use compile::analysis;
use compile::analysis::{VariableAnalysis, ClosureAnalysis};
use compile::analysis::find_variable_declarations::FindVariableDeclarations;
use compile::analysis::resolve_variables_and_set_times::ResolveVariablesAndSetTimes;
use compile::analysis::determine_variable_lifetime_and_storage_type::DetermineVariableLifetimeAndStorageType;
use compile::analysis::determine_allocation::DetermineAllocation;

pub struct Compiler;

	impl Compiler {
		
		pub fn new() -> Compiler {
			Compiler
		}
		
		pub fn compile_script( &mut self, script: &mut node::Script ) -> Result<Code,Vec<AnalysisError>> {
			
			{
				let mut pass = FindVariableDeclarations::new();
				pass.analyze_root( &mut script.root );
				if pass.errors.len() > 0 {
					return Err( pass.errors );
				}
			}
			
			{
				let mut pass = ResolveVariablesAndSetTimes::new();
				pass.analyze_root( &mut script.root );
				if pass.errors.len() > 0 {
					return Err( pass.errors );
				}
			}
			
			{
				let mut pass = DetermineVariableLifetimeAndStorageType::new();
				pass.analyze_root( &mut script.root );
				if pass.errors.len() > 0 {
					return Err( pass.errors );
				}
			}
			
			{
				let mut pass = DetermineAllocation::new();
				pass.analyze_root( &mut script.root );
				if pass.errors.len() > 0 {
					return Err( pass.errors );
				}
			}
			
			let code = {
				let mut compilation = Compilation::new();
				compilation.compile_root( &script.root );
				compilation.code
			};
			
			if unsafe { ::DEBUG_BYTECODE } {
				code.dump();
			}
			
			Ok( code )
		}
		
		pub fn compile_repl( &mut self, repl: &mut node::Repl, repl_state: &mut ::vm::repl::ReplState ) -> Result<Code,Vec<AnalysisError>> {
			
			let mut repl_vars = Vec::<Box<VariableAnalysis>>::new();
			
			for &name in repl_state.variables.keys() {
				let mut var = box VariableAnalysis::new( name );
				var.declared_in = Raw::new( &repl.root.scope );
				repl.root.scope.declared.push( Raw::new( var ) );
				repl_vars.push( var );
			}
			
			{
				let mut pass = FindVariableDeclarations::new();
				pass.analyze_root( &mut repl.root );
				if pass.errors.len() > 0 {
					return Err( pass.errors );
				}
			}
			
			for var in repl.root.scope.declared.iter().skip( repl_vars.len() ) {
				repl_state.declare_variable( var.get().name );
			}
			
			{
				let mut pass = ResolveVariablesAndSetTimes::new();
				pass.analyze_root( &mut repl.root );
				if pass.errors.len() > 0 {
					return Err( pass.errors );
				}
			}
			
			{
				let mut pass = DetermineVariableLifetimeAndStorageType::new();
				pass.analyze_root( &mut repl.root );
				if pass.errors.len() > 0 {
					return Err( pass.errors );
				}
			}
			
			{
				let mut pass = DetermineAllocation::new();
				pass.analyze_root( &mut repl.root );
				if pass.errors.len() > 0 {
					return Err( pass.errors );
				}
			}
			
			let code = {
				let mut compilation = Compilation::new();
				compilation.compile_root( &repl.root );
				compilation.code
			};
			
			if unsafe { ::DEBUG_BYTECODE } {
				code.dump();
			}
			
			Ok( code )
		}
	}

struct Compilation {
	code: Code,
	closure: Raw<ClosureAnalysis>,
}

	type Placeholder = uint;
	
	impl Compilation {
		
		fn new() -> Compilation {
			Compilation {
				code: Code::new(),
				closure: Raw::null(),
			}
		}
		
		fn create_placeholder( &mut self ) -> Placeholder {
			let offset = self.code.opcodes.len();
			self.code.opcodes.push( opcode::Nop );
			offset
		}
		
		fn fill_in_placeholder( &mut self, offset: Placeholder, opcode: opcode::OpCode ) {
			*self.code.opcodes.get_mut( offset ) = opcode;
		}
		
		fn compile_root( &mut self, root: &node::Root ) {
			
			self.code.n_local_variables = root.frame.n_local_variables;
			self.code.n_shared_local_variables = root.frame.n_shared_local_variables;
			
			for statement in root.statements.iter() {
				self.compile_statement( *statement );
			}
			self.code.opcodes.push( opcode::End );
		}
		
		fn compile_function( &mut self, function: &node::Expression ) {
			match *function {
				
				node::Function {
					parameters: _,
					closure: ref closure,
					scope: _,
					block: ref block,
				} => {
					
					self.code.n_local_variables = closure.frame.n_local_variables;
					self.code.n_shared_local_variables = closure.frame.n_shared_local_variables;
					
					for statement in block.iter() {
						self.compile_statement( *statement );
					}
					
					self.code.opcodes.push( opcode::PushNothing );
					self.code.opcodes.push( opcode::Return );
				}
				
				_ => fail!()
			};
		}
		
		fn compile_statement( &mut self, statement: &node::Statement ) {
			match *statement {
				
				node::ExpressionStatement {
					expression: ref expression,
				} => {
					self.compile_expression( *expression );
					self.code.opcodes.push( opcode::Pop );
				}
				
				node::Assignment {
					lvalue: ref lvalue,
					rvalue: ref rvalue,
				} => {
					
					self.compile_expression( *rvalue );
					
					match **lvalue {
						node::VariableLvalue {
							name: _,
							analysis: ref variable,
						} => {
							
							let variable = variable.get();
							
							// TODO free variables!!
							match variable.local_storage_type {
								analysis::LocalStorage => {
									self.code.opcodes.push( opcode::StoreLocal { index: variable.local_storage_index } );
								}
								analysis::SharedLocalStorage => {
									self.code.opcodes.push( opcode::StoreSharedLocal { index: variable.local_storage_index } );
								}
							};
						}
					}
				}
				
				node::If {
					test: ref test,
					scope: _,
					block: ref if_block,
					else_if_clauses: ref else_if_clauses,
					else_clause: ref else_clause,
				} => {
					
					let has_else_if_clauses = else_if_clauses.len() > 0;
					let has_else_clause = else_clause.is_some();
					
					let mut jump_else;
					let mut jump_end = Vec::<Placeholder>::new();
					
					self.compile_expression( *test );
					jump_else = self.create_placeholder();
					
					for statement in if_block.iter() {
						self.compile_statement( *statement );
					}
					
					if has_else_if_clauses || has_else_clause {
						jump_end.push( self.create_placeholder() );
					}
					
					let last_i = else_if_clauses.len() - 1;
					for (i, else_if_clause) in else_if_clauses.iter().enumerate() {
						
						let is_last = ( i == last_i );
						
						let jump = opcode::JumpIfPopFalsy { instruction: self.code.opcodes.len() };
						self.fill_in_placeholder( jump_else, jump );
						
						self.compile_expression( else_if_clause.test );
						jump_else = self.create_placeholder();
						
						for statement in else_if_clause.block.iter() {
							self.compile_statement( *statement );
						}
						
						if ! is_last || has_else_clause {
							jump_end.push( self.create_placeholder() );
						}
					}
					
					let jump = opcode::JumpIfPopFalsy { instruction: self.code.opcodes.len() };
					self.fill_in_placeholder( jump_else, jump );
					
					if has_else_clause {
						for statement in else_clause.as_ref().unwrap().block.iter() {
							self.compile_statement( *statement );
						}
					}
					
					let jump = opcode::Jump { instruction: self.code.opcodes.len() };
					for &placeholder in jump_end.iter() {
						self.fill_in_placeholder( placeholder, jump );
					}
				}
				
				node::Try {
					scope: _,
					block: ref try_block,
					catch_clauses: ref catch_clauses,
					else_clause: ref else_clause,
					finally_clause: ref finally_clause,
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
					
					for statement in try_block.iter() {
						self.compile_statement( *statement );
					}
					
					let mut end_catch = Vec::<Placeholder>::new();
					
					if has_catch_clauses {
						
						self.code.opcodes.push( opcode::PopFlowPoint );
						
						let end_try_jump = self.create_placeholder();
						
						let opcode = opcode::PushStartCatchFlowPoint { instruction: self.code.opcodes.len() };
						self.fill_in_placeholder( push_catch.unwrap(), opcode );
						
						for catch_clause in catch_clauses.iter() {
							
							let has_type = catch_clause.type_.is_some();
							
							if has_type {
								self.compile_expression( *catch_clause.type_.as_ref().unwrap() );
							}
							
							let catch = self.create_placeholder();
							
							for statement in catch_clause.block.iter() {
								self.compile_statement( *statement );
							}
							
							end_catch.push( self.create_placeholder() );
							
							let opcode = if has_type {
								opcode::CatchOrJump { instruction: self.code.opcodes.len() }
							} else {
								opcode::Catch
							};
							self.fill_in_placeholder( catch, opcode );
						}
						
						self.code.opcodes.push( opcode::Rethrow );
						
						let jump = opcode::Jump { instruction: self.code.opcodes.len() };
						self.fill_in_placeholder( end_try_jump, jump );
					}
					
					if has_else_clause {
						
						for statement in else_clause.as_ref().unwrap().block.iter() {
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
						
						for statement in finally_clause.as_ref().unwrap().block.iter() {
							self.compile_statement( *statement );
						}
						
						self.code.opcodes.push( opcode::EndFinally );
					}
				}
				
				node::Let {
					variable: ref variable,
					default: ref default,
					source_offset: _,
				} => {
					
					if default.is_some() {
						
						self.compile_expression( *default.as_ref().unwrap() );
						
						match variable.local_storage_type {
							analysis::LocalStorage => {
								self.code.opcodes.push( opcode::StoreLocal { index: variable.local_storage_index } );
							}
							analysis::SharedLocalStorage => {
								self.code.opcodes.push( opcode::StoreSharedLocal { index: variable.local_storage_index } );
							}
						};
					}
				}
				
				node::Print {
					expression: ref expression,
				} => {
					self.compile_expression( *expression );
					self.code.opcodes.push( opcode::Print );
				}
				
				node::Throw {
					expression: ref expression,
				} => {
					self.compile_expression( *expression );
					self.code.opcodes.push( opcode::Throw );
				}
				
				_ => fail!(), // TODO
			}
		}
		
		fn compile_expression( &mut self, expression: &node::Expression ) {
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
					let string = box String::new( value.clone() );
					self.code.opcodes.push( opcode::PushString { index: self.code.strings.len() } );
					self.code.strings.push( Rc::new( string ) );
				}
				
				node::Variable {
					name: _,
					analysis: ref variable,
					source_offset: _,
				} => {
					
					let variable = variable.get();
					
					if self.closure.is_null() || variable.declared_in.get().frame.get() == &mut self.closure.get().frame {
						
						match variable.local_storage_type {
							analysis::LocalStorage => {
								self.code.opcodes.push( opcode::LoadLocal { index: variable.local_storage_index } );
							}
							analysis::SharedLocalStorage => {
								self.code.opcodes.push( opcode::LoadSharedLocal { index: variable.local_storage_index } );
							}
						}
						
					} else {
						
						for binding in self.closure.get().bound.iter() {
							if binding.variable.get() == variable {
								
								match variable.bound_storage_type {
									analysis::StaticBoundStorage => {
										self.code.opcodes.push( opcode::LoadStaticBound { index: binding.storage_index } );
									}
									analysis::SharedBoundStorage => {
										self.code.opcodes.push( opcode::LoadSharedBound { index: binding.storage_index } );
									}
								}
								
								break;
							}
						}
					}
				}
				
				node::Name {
					identifier: identifier,
				} => {
					// TODO
					self.code.opcodes.push( opcode::LoadIntrinsic { name: identifier } );
				}
				
				node::Call {
					expression: ref expression,
					arguments: ref arguments,
				} => {
					
					self.compile_expression( *expression );
					
					for argument in arguments.iter() {
						self.compile_expression( *argument );
					}
					
					self.code.opcodes.push( opcode::Call { n_arguments: arguments.len() } );
				}
				
				node::Addition {
					left: ref left,
					right: ref right,
				} => {
					self.compile_expression( *left );
					self.compile_expression( *right );
					self.code.opcodes.push( opcode::Add );
				}
				
				node::Subtraction {
					left: ref left,
					right: ref right,
				} => {
					self.compile_expression( *left );
					self.compile_expression( *right );
					self.code.opcodes.push( opcode::Subtract );
				}
				
				node::Union {
					left: ref left,
					right: ref right,
				} => {
					self.compile_expression( *left );
					self.compile_expression( *right );
					self.code.opcodes.push( opcode::Union );
				}
				
				node::Is {
					left: ref left,
					right: ref right,
				} => {
					self.compile_expression( *left );
					self.compile_expression( *right );
					self.code.opcodes.push( opcode::Is );
				}
				
				node::Function {..} => {
					
					let mut compilation = Compilation::new();
					compilation.compile_function( expression );
					let code = compilation.code;
					let definition = box FunctionDefinition::new( Vec::new(), code );
					
					self.code.opcodes.push( opcode::PushFunction { index: self.code.functions.len() } );
					self.code.functions.push( Rc::new( definition ) );
				}
				
				_ => fail!(), // TODO
			}
		}
	}

