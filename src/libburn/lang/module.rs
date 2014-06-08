use std::collections::HashMap;
use std::io::File;
use serialize::{json, Decodable};
use mem::raw::Raw;
use mem::rc::Rc;
use lang::origin;
use lang::origin::Origin;
use lang::identifier::Identifier;
use lang::value;
use lang::value::Value;
use vm::bytecode::code::Code;
use vm::bytecode::compiler;
use vm::bytecode::opcode;
use vm::run::rust;
use vm::run::rust::Operation;
use vm::virtual_machine::VirtualMachine;

pub struct Module {
	modules: HashMap<Identifier, Box<Module>>,
	contents: HashMap<Identifier, Value>,
	locked: bool,
}

	impl Module {
		pub fn new() -> Module {
			Module {
				modules: HashMap::new(),
				contents: HashMap::new(),
				locked: false,
			}
		}
		
		pub fn add_module( &mut self,  name: &'static str, module: Box<Module> ) {
			self.add_module_with_id( Identifier::find_or_create( name ), module );
		}
		
		pub fn add_module_with_id( &mut self, name: Identifier, module: Box<Module> ) {
			assert!( ! self.locked );
			self.contents.insert( name, value::Module( Raw::new( module ) ) );
			self.modules.insert( name, module );
		}
		
		pub fn add( &mut self, name: &'static str, value: Value ) {
			self.add_with_id( Identifier::find_or_create( name ), value );
		}
		
		pub fn add_with_id( &mut self, name: Identifier, value: Value ) {
			assert!( ! self.locked );
			self.contents.insert( name, value );
		}
		
		pub fn has( &self, name: &'static str ) -> bool {
			self.has_id( Identifier::find_or_create( name ) )
		}
		
		pub fn has_id( &self, name: Identifier ) -> bool {
			self.contents.contains_key( &name )
		}
		
		pub fn lock( &mut self ) {
			assert!( ! self.locked );
			self.locked = true
		}
		
		pub fn find_id( &self, identifier: Identifier ) -> Result<Value, Value> {
			match self.contents.find( &identifier ) {
				Some( value ) => Ok( value.clone() ),
				None => Err( value::Nothing ), // todo! add a real error
			}
		}
		
		pub fn get( &self, name: &'static str ) -> Value {
			self.get_id( Identifier::find_or_create( name ) )
		}
		
		pub fn get_id( &self, name: Identifier ) -> Value {
			match self.contents.find( &name ) {
				Some( value ) => value.clone(),
				None => { fail!(); },
			}
		}
		
		pub fn get_module<'l>( &'l mut self, name: &'static str ) -> &'l mut Module {
			match self.modules.find_mut( &Identifier::find_or_create( name ) ) {
				Some( module ) => &mut **module,
				None => { fail!(); },
			}
		}
	}

#[deriving(Decodable)]
pub struct MetaData {
	sources: Vec<String>,
}

pub struct Use {
	fqn: Vec<Identifier>,
	inlines: Vec<(Raw<Code>, uint)>,
	step: UseOperationStep,
	root_sources: Vec<Path>,
	loaded: Value,
}

	impl Use {
		
		pub fn new( fqn: Vec<Identifier> ) -> Use {
			Use {
				fqn: fqn,
				inlines: Vec::new(),
				step: FindRoot,
				root_sources: Vec::new(),
				loaded: value::Nothing,
			}
		}
		
		pub fn add_inline( &mut self, code: Raw<Code>, offset: uint ) {
			self.inlines.push( (code, offset) );
		}
	}
	
	enum UseOperationStep {
		FindRoot,
		ImportRoot,
		ImportSubs,
		Inline,
	}
	
	impl Operation for Use {
		fn run( &mut self, vm: &mut VirtualMachine, value: Result<Value, Value> ) -> rust::Result {
			'step_loop: loop {
				match self.step {
				
					FindRoot => {
						let mut module_name = self.fqn.shift().unwrap();
						
						if vm.module_root.has_id( module_name ) {
							
							self.loaded = vm.module_root.get_id( module_name );
							self.step = ImportSubs;
							
						} else {
							
							for import_path in vm.import_paths.iter() {
								
								let mut suspect = import_path.clone();
								suspect.push( format!( "{}/burn_module.json", module_name.get_value() ) );
								
								if suspect.exists() {
									
									let module = box Module::new();
									self.loaded = value::Module( Raw::new( module ) );
									vm.module_root.add_module_with_id( module_name, module );
									
									// todo! handle errors
									let mut meta_file = File::open( &suspect ).unwrap();
									let meta_data = json::from_reader( &mut meta_file ).unwrap();
									let mut decoder = json::Decoder::new( meta_data );
									let meta_struct: MetaData = Decodable::decode( &mut decoder ).unwrap();	
									
									for source in meta_struct.sources.move_iter() {
										let mut source_path = import_path.clone();
										source_path.push( format!( "{}/{}", module_name.get_value(), source ) );
										self.root_sources.push( source_path );
									}
									
									self.step = ImportRoot;
									continue 'step_loop;
								}
							}
							
							return rust::Throw( value::Integer( 3 ) );
						}
					}
					
					ImportRoot => {
						match self.root_sources.shift() {
							None => {
								self.step = ImportSubs;
							}
							Some( path ) => {
								
								let script = box origin::Script { path: path };
								let source = ::std::io::File::open( &script.path ).unwrap().read_to_str().unwrap(); // todo! handle errors
								let origin = script as Box<Origin>;
								
								return match compiler::compile( Rc::new( origin ), None, source.as_slice() ) {
									Ok( frame ) => rust::Burn( frame ),
									Err( errors ) => {
										(errors);
										return rust::Throw( value::Nothing ); // todo! add a real error
									}
								}
							}
						}
					}
					
					ImportSubs => {
						
						match value {
							Ok( value::Nothing ) => {},
							Ok( _ ) => { unreachable!(); },
							Err( t ) => {
								return rust::Throw( t );
							}
						};
						
						match self.fqn.shift() {
							None => {
								self.step = Inline;
							}
							Some( name ) => {
								(name);
								unimplemented!();
							}
						}
					}
					
					Inline => {
						
						let opcode = match self.loaded {
							value::Module( m ) => opcode::InlinedModule { ptr: m },
							_ => { unimplemented!(); }
						};
						
						for &(mut code, offset) in self.inlines.mut_iter() {
							*code.opcodes.get_mut( offset ) = opcode;
						}
						
						return rust::Ok( value::Nothing );
					}
				}
			}
		}
	}
