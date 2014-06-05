use collections::HashMap;
use mem::raw::Raw;
use lang::identifier::Identifier;
use lang::value;
use vm::bytecode::code::Code;
use vm::bytecode::compiler;
use vm::bytecode::opcode;
use vm::run::rust;
use vm::run::rust::Operation;
use vm::virtual_machine::VirtualMachine;

pub struct Module {
	modules: HashMap<Identifier, Box<Module>>,
	contents: HashMap<Identifier, value::Value>,
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
		
		pub fn add( &mut self, name: &'static str, value: value::Value ) {
			self.add_with_id( Identifier::find_or_create( name ), value );
		}
		
		pub fn add_with_id( &mut self, name: Identifier, value: value::Value ) {
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
		
		pub fn find_id( &self, identifier: Identifier ) -> Result<value::Value, value::Value> {
			match self.contents.find( &identifier ) {
				Some( value ) => Ok( value.clone() ),
				None => Err( value::Nothing ), // todo! add a real error
			}
		}
		
		pub fn get( &self, name: &'static str ) -> value::Value {
			self.get_id( Identifier::find_or_create( name ) )
		}
		
		pub fn get_id( &self, name: Identifier ) -> value::Value {
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

pub struct Use {
	fqn: Vec<Identifier>,
	inlines: Vec<(Raw<Code>, uint)>,
	step: UseOperationStep,
	root_sources: Vec<Path>,
	loaded: value::Value,
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
		fn run( &mut self, vm: &mut VirtualMachine, value: Result<value::Value, value::Value> ) -> rust::Result {
			'step_loop: loop {
				match self.step {
				
					FindRoot => {
						let name = self.fqn.shift().unwrap();
						
						if vm.module_root.has_id( name ) {
							
							self.loaded = vm.module_root.get_id( name );
							self.step = ImportSubs;
							
						} else {
							
							for path in vm.import_paths.iter() {
								
								let mut suspect = path.clone();
								suspect.push( format!( "{}/burnmod.json", name.get_value() ) );
								
								if suspect.exists() {
									
									let module = box Module::new();
									self.loaded = value::Module( Raw::new( module ) );
									vm.module_root.add_module_with_id( name, module );
									
									// todo! read the json
									self.root_sources.push( Path::new( "modules/example/src/example.burn" ) );
									
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
								let source = ::std::io::File::open( &path ).unwrap().read_to_str().unwrap();
								return match compiler::compile_script( source.as_slice() ) {
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
						
						for &(code, offset) in self.inlines.iter() {
							*code.get().opcodes.get_mut( offset ) = opcode;
						}
						
						return rust::Ok( value::Nothing );
					}
				}
			}
		}
	}
