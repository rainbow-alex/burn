use lang::identifier::Identifier;

#[deriving(Show)]
pub enum OpCode {
	
	// Temp
	Print,
	
	// VM commands
	Nop,
	End,
	ReturnPop,
	
	// Scopes, locals, cells
	// PushLocal { pub depth: u32, pub index: u32 },
	// PopLocal { pub depth: u32, pub index: u32 },
	
	// Flow
	PopFlowPoint,
	Jump { pub instruction: uint },
	JumpIfPopFalsy { pub instruction: uint },
	FlowJump { pub n_flow_points: uint, pub instruction: uint },
	
	// Function flow
	Call { pub n_arguments: uint },
	Return,
	
	// Try catch
	PushStartCatchFlowPoint { pub instruction: uint },
	PushStartFinallyFlowPoint { pub instruction: uint },
	Throw,
	CatchOrJump { pub instruction: uint },
	Catch,
	Rethrow,
	StartFinally,
	EndFinally,
	
	// Data stack operations
	PushFunction { pub index: uint },
	//PushBoundBurnFunction { pub index: uint },
	PushString { pub index: uint },
	PushFloat { pub value: f64 },
	PushInteger { pub value: i64 },
	PushBoolean { pub value: bool },
	PushNothing,
	Pop,
	
	// Variables
	StoreLocal { pub index: uint },
	LoadLocal { pub index: uint },
	StoreSharedLocal { pub index: uint },
	LoadSharedLocal { pub index: uint },
	StoreStaticBound { pub index: uint },
	LoadStaticBound { pub index: uint },
	StoreSharedBound { pub index: uint },
	LoadSharedBound { pub index: uint },
	
	// Names
	LoadIntrinsic { pub name: Identifier },
	
	// Operations
	Is,
	Union,
	Add,
	Subtract,
}
