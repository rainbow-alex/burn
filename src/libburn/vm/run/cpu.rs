use std::mem;
use mem::rc::Rc;
use lang::value;
use lang::function;
use lang::operations;
use vm::bytecode::opcode;
use vm::result;
use vm::virtual_machine::VirtualMachine;
use vm::run::fiber::Fiber;
use vm::run::{frame, flow, rust};
use vm::run::rust::Operation;
use builtin::burn::{errors, types};

pub fn run( vm: &mut VirtualMachine, mut fiber: Box<Fiber> ) -> result::Result {
	
	'frame_loop: loop {
	if fiber.frame.is_rust() {
		
		match fiber.flow.clone() { // TODO
			
			flow::Running => {
				
				let result = match fiber.replace_flow( flow::Running ) {
					flow::Running => Ok( value::Nothing ),
					flow::Returning( v ) => Ok( v ),
					flow::Throwing( v ) => Err( v ),
					_ => fail!(),
				};
				
				match fiber.frame.get_rust_operation().run( vm, result ) {
					
					rust::Ok( value ) => {
						fiber.set_flow( flow::Returning( value ) );
					}
					
					rust::Throw( value ) => {
						fiber.set_flow( flow::Throwing( value ) );
					}
					
					rust::Burn( frame ) => {
						fiber.flow_points.push( flow::PopFrame { data_stack_len: fiber.data_stack.len() } );
						fiber.push_frame( frame );
					}
					
					_ => { fail!( "TODO" ); }
				}
			}
			
			flow::Returning( value ) => {
				loop {
					match fiber.flow_points.pop().unwrap() {
						
						flow::PopFrame { data_stack_len: n } => {
							fiber.pop_frame();
							fiber.data_stack.truncate( n );
							fiber.push_data( value );
							fiber.set_flow( flow::Running );
							continue 'frame_loop;
						}
						
						flow::PopFrameAndRestoreFlow { data_stack_len: n } => {
							fiber.pop_frame();
							fiber.data_stack.truncate( n );
							fiber.push_data( value );
							fiber.restore_flow();
							continue 'frame_loop;
						}
						
						_ => { fail!(); }
					}
				}
			}
			
			_ => { fail!(); }
		}
		
	} else { // not a rust-type frame
		
		let opcodes = fiber.frame.get_code().opcodes.as_mut_ptr();
		
		'flow_loop: loop {
		match fiber.flow.clone() { // clone because of rust#6393
			
			flow::Running | flow::Catching(..) => {
				
				'instruction_loop: loop {
					
					debug!( {
						println!(
							"VM: running {}/{} ({})",
							fiber.frame.instruction.clone(), // clone because of unidentified rust bug
							fiber.frame.get_code().opcodes.len(),
							fiber.flow_points.len()
						);
					} )
					
					macro_rules! throw (
						( $throwable:expr ) => {{
							fiber.set_flow( flow::Throwing( $throwable ) );
							continue 'flow_loop;
						}}
					)
					
					macro_rules! handle_operation_result (
						( $operation:expr ) => {{
							match $operation {
								rust::Ok( result ) => { fiber.push_data( result ); }
								rust::Throw( t ) => { throw!( t ); }
								rust::Burn( frame ) => {
									fiber.push_frame( frame );
									continue 'frame_loop;
								}
								_ => { fail!(); } // TODO
							};
						}}
					)
					
					match unsafe { *opcodes.offset( fiber.frame.instruction as int ) } {
						
						// Temporary
						
						opcode::Print => {
							match fiber.pop_data().to_string() {
								rust::Ok( value::String( s ) ) => println!( "{}", s.get() ),
								rust::Ok( _ ) => { fail!(); }
								rust::Throw( t ) => { throw!( t ); }
								_ => { fail!( "TODO" ); }
							};
						}
						
						// VM
						
						opcode::Nop => {}
						
						opcode::Fail => {
							fail!();
						}
						
						// Flow
						
						opcode::PopFlowPoint => {
							fiber.flow_points.pop();
						}
						
						opcode::Jump { instruction: i } => {
							fiber.frame.instruction = i;
							continue 'instruction_loop;
						}
						
						opcode::JumpIfPopFalsy { instruction: i } => {
							if ! operations::is_truthy( &fiber.pop_data() ) {
								fiber.frame.instruction = i;
								continue 'instruction_loop;
							}
						}
						
						opcode::FlowJump { n_flow_points: n, instruction: i } => {
							fiber.set_flow( flow::Jumping { n_flow_points: n, instruction: i } );
							continue 'flow_loop;
						}
						
						// Functions
						
						opcode::Call { n_arguments: n_arguments } => {
							
							let function_offset = fiber.data_stack.len() - n_arguments - 1;
							let function = mem::replace( fiber.data_stack.get_mut( function_offset ), value::Nothing );
							
							match function {
								
								value::Function( function ) => {
									
									fiber.frame.instruction += 1;
									
									let mut locals = Vec::from_elem( function.get().definition.get().code.n_local_variables, value::Nothing );
									let mut shared = Vec::from_fn( function.get().definition.get().code.n_shared_local_variables, |_| { Rc::new( value::Nothing ) } );
									
									let parameters = function.get().definition.get().parameters.as_slice();
									assert!( n_arguments == parameters.len() );
									for parameter in parameters.iter().rev() {
										match parameter.storage {
											function::LocalFunctionParameterStorage( i ) => {
												*locals.get_mut( i ) = fiber.pop_data();
											}
											function::SharedLocalFunctionParameterStorage( i ) => {
												*shared.get_mut( i ).get() = fiber.pop_data();
											}
										};
									}
									
									fiber.push_frame( frame::Frame::new_function( function, locals, shared ) );
									
									continue 'frame_loop;
								}
								
								_ => { fail!( "TODO" ); }
							}
						}
						
						opcode::TypeCheckLocal { index: _ } => {
							fail!( "TODO" );
						}
						
						opcode::TypeCheckSharedLocal { index: _ } => {
							fail!( "TODO" );
						}
						
						opcode::Return => {
							if fiber.frame_stack.len() > 0 {
								let flow = flow::Returning( fiber.pop_data() );
								fiber.set_flow( flow );
								continue 'flow_loop;
							} else {
								return result::Done;
							}
						}
						
						opcode::ReturnNothing => {
							if fiber.frame_stack.len() > 0 {
								fiber.set_flow( flow::Returning( value::Nothing ) );
								continue 'flow_loop;
							} else {
								return result::Done;
							}
						}
						
						// Try/Catch
						
						opcode::PushStartCatchFlowPoint { instruction: i } => {
							fiber.flow_points.push( flow::StartCatch { instruction: i } );
						}
						
						opcode::PushStartFinallyFlowPoint { instruction: i } => {
							fiber.flow_points.push( flow::StartFinally { instruction: i } );
						}
						
						opcode::Throw => {
							
							let throwable = fiber.pop_data();
							
							if types::is_throwable( &throwable ) {
								throw!( throwable );
							} else {
								let message = format!( "{} is not Throwable.", throwable.repr() );
								let error = errors::create_type_error( message );
								throw!( error );
							}
						}
						
						opcode::CatchOrJump { instruction: i } => {
							
							let throwable = match fiber.flow {
								flow::Catching( ref t ) => t.clone(), // ref+clone because of rust#6393
								_ => fail!(),
							};
							
							let type_ = fiber.pop_data();
							let result = operations::is( &throwable, &type_ );
							
							match result {
								Ok( true ) => {
									*fiber.frame.get_local_variable( 0 ) = throwable; // TODO this ain't right at all
									fiber.set_flow( flow::Running );
								},
								Ok( false ) => {
									fiber.frame.instruction = i;
									fiber.set_flow( flow::Catching( throwable ) );
									continue 'instruction_loop;
								}
								Err( e ) => { throw!( e ); }
							}
						}
						
						opcode::Catch => {
							
							let throwable = match fiber.flow {
								flow::Catching( ref t ) => t.clone(), // ref+clone because of rust#6393
								_ => fail!(),
							};
							
							*fiber.frame.get_local_variable( 0 ) = throwable;
							fiber.set_flow( flow::Running );
						}
						
						opcode::Rethrow => {
							let throwable = match fiber.flow {
								flow::Catching( ref t ) => t.clone(), // ref+clone because of rust#6393
								_ => fail!(),
							};
							throw!( throwable );
						}
						
						opcode::StartFinally => {
							fiber.flow_points.pop();
							fiber.suppressed_flows.push( flow::Running );
							fiber.flow_points.push( flow::PopSuppressedFlow );
						}
						
						opcode::EndFinally => {
							fiber.flow_points.pop();
							let flow = fiber.suppressed_flows.pop().unwrap();
							fiber.set_flow( flow );
							match fiber.flow {
								flow::Running => {},
								_ => continue 'flow_loop,
							};
						}
						
						// Data stack operations
						
						opcode::Pop => {
							fiber.data_stack.pop();
						}
						
						// Values
						
						opcode::PushFunction { index: i } => {
							
							let mut function = function::Function::new( fiber.frame.get_code().functions.get( i ).clone() );
							
							for binding in function.definition.get().bindings.iter() {
								match *binding {
									function::LocalToStaticBinding( from, to ) => {
										*function.static_bound_variables.get_mut( to ) = fiber.frame.get_local_variable( from ).clone();
									}
									function::LocalSharedToSharedBinding( from, to ) => {
										*function.shared_bound_variables.get_mut( to ) = fiber.frame.get_shared_local_variable( from ).clone();
									}
									function::StaticToStaticBinding( from, to ) => {
										*function.static_bound_variables.get_mut( to ) = fiber.frame.get_closure().static_bound_variables.get( from ).clone();
									}
									function::BoundSharedToSharedBinding( from, to ) => {
										*function.shared_bound_variables.get_mut( to ) = fiber.frame.get_closure().shared_bound_variables.get( from ).clone();
									}
								}
							}
							
							fiber.push_data(
								value::Function(
									vm.functions.register(
										function
									)
								)
							);
						}
						
						opcode::PushString { index: i } => {
							let string = fiber.frame.get_code().strings.get( i ).clone();
							fiber.push_data( value::String( string ) );
						}
						
						opcode::PushFloat { value: f } => {
							fiber.push_data( value::Float( f ) );
						}
						
						opcode::PushInteger { value: i } => {
							fiber.push_data( value::Integer( i ) );
						}
						
						opcode::PushBoolean { value: b } => {
							fiber.push_data( value::Boolean( b ) );
						}
						
						opcode::PushNothing => {
							fiber.push_data( value::Nothing );
						}
						
						opcode::InlinedModule { ptr: ptr } => {
							fiber.push_data( value::Module( ptr ) );
						}
						
						// Variables
						
						opcode::StoreLocal { index: i } => {
							*fiber.frame.get_local_variable( i ) = fiber.pop_data();
						}
						
						opcode::LoadLocal { index: i } => {
							let value = fiber.frame.get_local_variable( i ).clone();
							fiber.push_data( value );
						}
						
						opcode::StoreSharedLocal { index: i } => {
							*fiber.frame.get_shared_local_variable( i ).get() = fiber.pop_data();
						}
						
						opcode::LoadSharedLocal { index: i } => {
							let value = fiber.frame.get_shared_local_variable( i ).get().clone();
							fiber.push_data( value );
						}
						
						opcode::StoreStaticBound { index: i } => {
							*fiber.frame.get_closure().static_bound_variables.get_mut( i ) = fiber.pop_data();
						}
						
						opcode::LoadStaticBound { index: i } => {
							let value = fiber.frame.get_closure().static_bound_variables.get( i ).clone();
							fiber.push_data( value );
						}
						
						opcode::StoreSharedBound { index: i } => {
							*fiber.frame.get_closure().shared_bound_variables.get( i ).get() = fiber.pop_data();
						}
						
						opcode::LoadSharedBound { index: i } => {
							let value = fiber.frame.get_closure().shared_bound_variables.get( i ).get().clone();
							fiber.push_data( value );
						}
						
						// Names
						
						opcode::Use { operation: operation } => {
							let operation = unsafe { operation.get_box() };
							unsafe { *opcodes.offset( fiber.frame.instruction as int ) = opcode::Nop; }
							
							fiber.frame.instruction += 1;
							
							fiber.push_frame( frame::Frame::new_rust( operation as Box<rust::Operation> ) );
							continue 'frame_loop;
						}
						
						opcode::LoadImplicit { name: name } => {
							match vm.implicit.get().find_id( name ) {
								Ok( value ) => {
									fiber.push_data( value.clone() );
								}
								Err( err ) => {
									fiber.set_flow( flow::Throwing( err ) );
									continue 'flow_loop;
								}
							}
						}
						
						// Access
						
						opcode::GetProperty { name: name } => {
							let left = fiber.pop_data();
							handle_operation_result!( operations::get_property( &left, name ) );
						}
						
						opcode::SetProperty { name: name } => {
							let right = fiber.pop_data();
							let left = fiber.pop_data();
							handle_operation_result!( operations::set_property( &left, name, &right ) );
						}
						
						opcode::GetItem => {
							let key = fiber.pop_data();
							let expression = fiber.pop_data();
							(key); (expression);
							fail!( "TODO" );
						}
						
						// Operators
						
						opcode::Add => {
							let right = fiber.pop_data();
							let left = fiber.pop_data();
							handle_operation_result!( operations::add( &left, &right ) );
						}
						
						opcode::Subtract => {
							let right = fiber.pop_data();
							let left = fiber.pop_data();
							handle_operation_result!( operations::subtract( &left, &right ) );
						}
						
						opcode::Multiply => {
							let right = fiber.pop_data();
							let left = fiber.pop_data();
							handle_operation_result!( operations::multiply( &left, &right ) );
						}
						
						opcode::Divide => {
							let right = fiber.pop_data();
							let left = fiber.pop_data();
							handle_operation_result!( operations::divide( &left, &right ) );
						}
						
						opcode::Union => {
							let right = fiber.pop_data();
							let left = fiber.pop_data();
							handle_operation_result!( operations::union( left, right ) );
						}
						
						opcode::Is => {
							let right = fiber.pop_data();
							let left = fiber.pop_data();
							match operations::is( &left, &right ) {
								Ok( result ) => { fiber.push_data( value::Boolean( result ) ); }
								Err( err ) => { throw!( err ); }
							};
						}
						
						opcode::Eq => {
							let right = fiber.pop_data();
							let left = fiber.pop_data();
							handle_operation_result!( operations::eq( &left, &right ) );
						}
						
						opcode::Neq => {
							let right = fiber.pop_data();
							let left = fiber.pop_data();
							handle_operation_result!( operations::neq( &left, &right ) );
						}
						
						opcode::Lt => {
							let right = fiber.pop_data();
							let left = fiber.pop_data();
							handle_operation_result!( operations::lt( &left, &right ) );
						}
						
						opcode::Gt => {
							let right = fiber.pop_data();
							let left = fiber.pop_data();
							handle_operation_result!( operations::gt( &left, &right ) );
						}
						
						opcode::LtEq => {
							let right = fiber.pop_data();
							let left = fiber.pop_data();
							handle_operation_result!( operations::lt_eq( &left, &right ) );
						}
						
						opcode::GtEq => {
							let right = fiber.pop_data();
							let left = fiber.pop_data();
							handle_operation_result!( operations::gt_eq( &left, &right ) );
						}
						
						opcode::Not => {
							fail!( "TODO" );
						}
						
						opcode::ShortCircuitAnd => {
							fail!( "TODO" );
						}
						
						opcode::ShortCircuitOr => {
							fail!( "TODO" );
						}
						
					} // match opcode
					
					fiber.frame.instruction += 1;
					
				} // 'instruction_loop
				
			} // flow::Running | flow::Catching
			
			flow::Jumping { n_flow_points: mut n_flow_points, instruction: instruction } => {
				
				while n_flow_points > 0 {
					
					match fiber.flow_points.pop().unwrap() {
						
						flow::StartCatch {..} => {
							// ignored, there is no throwable that needs to be caught
						}
						
						flow::StartFinally { instruction: i } => {
							fiber.suppressed_flows.push( flow::Jumping { n_flow_points: n_flow_points, instruction: instruction } );
							fiber.flow_points.push( flow::PopSuppressedFlow );
							fiber.set_flow( flow::Running );
							fiber.frame.instruction = i;
							continue 'flow_loop;
						}
						
						flow::PopFrame {..} | flow::PopFrameAndRestoreFlow {..} => fail!(),
						
						flow::PopSuppressedFlow => {
							fiber.suppressed_flows.pop();
						}
					}
					
					n_flow_points -= 1;
				}
				
			} // flow::Jumping( e )
			
			flow::Returning( value ) => {
				
				loop {
					match fiber.flow_points.pop().unwrap() {
						
						flow::StartCatch {..} => {
							// ignored, there is no throwable that needs to be caught
						}
						
						flow::StartFinally { instruction: i } => {
							fiber.suppressed_flows.push( flow::Returning( value ) );
							fiber.flow_points.push( flow::PopSuppressedFlow );
							fiber.set_flow( flow::Running );
							fiber.frame.instruction = i;
							continue 'flow_loop;
						}
						
						flow::PopFrame { data_stack_len: n } => {
							fiber.pop_frame();
							fiber.data_stack.truncate( n );
							fiber.push_data( value );
							fiber.set_flow( flow::Running );
							continue 'frame_loop;
						}
						
						flow::PopFrameAndRestoreFlow { data_stack_len: n } => {
							fiber.pop_frame();
							fiber.data_stack.truncate( n );
							fiber.push_data( value );
							fiber.restore_flow();
							continue 'frame_loop;
						}
						
						flow::PopSuppressedFlow => {
							fiber.suppressed_flows.pop();
						}
					}
				}
				
			} // flow::Returning( value )
			
			flow::Throwing( throwable ) => {
				
				loop {
					
					if fiber.flow_points.len() == 0 {
						return result::UncaughtThrowable( throwable );
					}
					
					match fiber.flow_points.pop().unwrap() {
						
						flow::StartCatch { instruction: i } => {
							fiber.set_flow( flow::Catching( throwable ) );
							fiber.frame.instruction = i;
							continue 'frame_loop;
						}
						
						flow::StartFinally { instruction: i } => {
							fiber.suppressed_flows.push( flow::Throwing( throwable ) );
							fiber.flow_points.push( flow::PopSuppressedFlow );
							fiber.set_flow( flow::Running );
							fiber.frame.instruction = i;
							continue 'frame_loop;
						}
						
						flow::PopFrame { data_stack_len: n }
						| flow::PopFrameAndRestoreFlow { data_stack_len: n } => {
							fiber.pop_frame();
							fiber.data_stack.truncate( n );
							continue 'frame_loop;
						}
						
						flow::PopSuppressedFlow => {
							fiber.suppressed_flows.pop();
						}
					}
				}
				
			} // flow::Throwing( e )
			
		} // match flow
		} // 'flow_start
	} // if is_rust else
	} // 'frame_start
}
