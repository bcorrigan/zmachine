use crate::object::Object;
use crate::zmachine::State;
use crate::error::Error;
use crate::memory::{Memory, StackFrame};
use std::ops::Deref;

/// Trait representing a single Z-Machine instruction.
pub trait Instruction {
    /// Executes the instruction with the given operands.
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error>;

    /// Returns the mnemonic name of the instruction (e.g., "ADD", "JE").
    fn name(&self) -> &'static str;

    /// Returns a helpful description of the instruction.
    fn description(&self) -> &'static str {
        "No description available."
    }
}

// ============================================================================
// Placeholder / Default Instructions
// ============================================================================

#[derive(Clone)]
pub struct IllegalInstruction(pub u8);

impl Instruction for IllegalInstruction {
    fn execute(&self, _state: &mut State, _operands: Vec<u16>) -> Result<(), Error> {
        Err(Error::ZMachineError(format!("Illegal opcode: 0x{:02x}", self.0)))
    }

    fn name(&self) -> &'static str {
        "ILLEGAL"
    }

    fn description(&self) -> &'static str {
        "An illegal or unsupported opcode."
    }
}

#[derive(Clone)]
pub struct NopInstruction;

impl Instruction for NopInstruction {
    fn execute(&self, _state: &mut State, _operands: Vec<u16>) -> Result<(), Error> {
        Ok(())
    }

    fn name(&self) -> &'static str {
        "NOP"
    }

    fn description(&self) -> &'static str {
        "No operation."
    }
}

#[derive(Clone)]
pub struct Add;

impl Instruction for Add {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let a = operands[0] as i16;
        let b = operands[1] as i16;
        let result = a.wrapping_add(b);
        
        let store_var = state.next_u8();
        state.store_variable(store_var, result as u16)
    }

    fn name(&self) -> &'static str {
        "ADD"
    }

    fn description(&self) -> &'static str {
        "Adds two signed 16-bit integers."
    }
}

#[derive(Clone)]
pub struct Sub;

impl Instruction for Sub {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let a = operands[0] as i16;
        let b = operands[1] as i16;
        let result = a.wrapping_sub(b);
        
        let store_var = state.next_u8();
        state.store_variable(store_var, result as u16)
    }

    fn name(&self) -> &'static str {
        "SUB"
    }

    fn description(&self) -> &'static str {
        "Subtracts two signed 16-bit integers."
    }
}

#[derive(Clone)]
pub struct Mul;

impl Instruction for Mul {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let a = operands[0] as i16;
        let b = operands[1] as i16;
        let result = a.wrapping_mul(b);
        
        let store_var = state.next_u8();
        state.store_variable(store_var, result as u16)
    }

    fn name(&self) -> &'static str {
        "MUL"
    }

    fn description(&self) -> &'static str {
        "Multiplies two signed 16-bit integers."
    }
}

#[derive(Clone)]
pub struct Div;

impl Instruction for Div {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let a = operands[0] as i16;
        let b = operands[1] as i16;
        if b == 0 {
            return Err(Error::ZMachineError("Division by zero".to_string()));
        }
        let result = a.wrapping_div(b);
        
        let store_var = state.next_u8();
        state.store_variable(store_var, result as u16)
    }

    fn name(&self) -> &'static str {
        "DIV"
    }

    fn description(&self) -> &'static str {
        "Divides two signed 16-bit integers."
    }
}

#[derive(Clone)]
pub struct Mod;

impl Instruction for Mod {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let a = operands[0] as i16;
        let b = operands[1] as i16;
        if b == 0 {
            return Err(Error::ZMachineError("Division by zero".to_string()));
        }
        let result = a.wrapping_rem(b);
        
        let store_var = state.next_u8();
        state.store_variable(store_var, result as u16)
    }

    fn name(&self) -> &'static str {
        "MOD"
    }

    fn description(&self) -> &'static str {
        "Remainder of division of two signed 16-bit integers."
    }
}

#[derive(Clone)]
pub struct Or;

impl Instruction for Or {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let a = operands[0];
        let b = operands[1];
        let result = a | b;
        
        let store_var = state.next_u8();
        state.store_variable(store_var, result)
    }

    fn name(&self) -> &'static str {
        "OR"
    }

    fn description(&self) -> &'static str {
        "Bitwise OR."
    }
}

#[derive(Clone)]
pub struct And;

impl Instruction for And {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let a = operands[0];
        let b = operands[1];
        let result = a & b;
        
        let store_var = state.next_u8();
        state.store_variable(store_var, result)
    }

    fn name(&self) -> &'static str {
        "AND"
    }

    fn description(&self) -> &'static str {
        "Bitwise AND."
    }
}

#[derive(Clone)]
pub struct Not;

impl Instruction for Not {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let a = operands[0];
        let result = !a;
        
        let store_var = state.next_u8();
        state.store_variable(store_var, result)
    }

    fn name(&self) -> &'static str {
        "NOT"
    }

    fn description(&self) -> &'static str {
        "Bitwise NOT."
    }
}

#[derive(Clone)]
pub struct Store;

impl Instruction for Store {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let var_id = operands[0] as u8;
        let value = operands[1];
        state.store_variable(var_id, value)
    }

    fn name(&self) -> &'static str {
        "STORE"
    }

    fn description(&self) -> &'static str {
        "Stores the value in the given variable."
    }
}

#[derive(Clone)]
pub struct Load;

impl Instruction for Load {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let var_id = operands[0] as u8;
        let value = state.read_variable(var_id)?;
        let result_var = state.next_u8();
        state.store_variable(result_var, value)
    }

    fn name(&self) -> &'static str {
        "LOAD"
    }

    fn description(&self) -> &'static str {
        "Loads the value from the given variable and stores it in the result variable."
    }
}

#[derive(Clone)]
pub struct StoreW;

impl Instruction for StoreW {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let array = operands[0];
        let index = operands[1];
        let value = operands[2];
        state.mem.write_u16(array.wrapping_add(index.wrapping_mul(2)), value);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "STOREW"
    }

    fn description(&self) -> &'static str {
        "Stores a 16-bit word at the given array address and index."
    }
}

#[derive(Clone)]
pub struct StoreB;

impl Instruction for StoreB {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let array = operands[0];
        let index = operands[1];
        let value = operands[2] as u8;
        state.mem.write_u8(array.wrapping_add(index), value);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "STOREB"
    }

    fn description(&self) -> &'static str {
        "Stores an 8-bit byte at the given array address and index."
    }
}

#[derive(Clone)]
pub struct LoadW;

impl Instruction for LoadW {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let array = operands[0];
        let index = operands[1];
        let value = state.mem.read_u16(array.wrapping_add(index.wrapping_mul(2)));
        let result_var = state.next_u8();
        state.store_variable(result_var, value)
    }

    fn name(&self) -> &'static str {
        "LOADW"
    }

    fn description(&self) -> &'static str {
        "Loads a 16-bit word from the given array address and index."
    }
}

#[derive(Clone)]
pub struct LoadB;

impl Instruction for LoadB {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let array = operands[0];
        let index = operands[1];
        let value = state.mem.read_u8(array.wrapping_add(index)) as u16;
        let result_var = state.next_u8();
        state.store_variable(result_var, value)
    }

    fn name(&self) -> &'static str {
        "LOADB"
    }

    fn description(&self) -> &'static str {
        "Loads an 8-bit byte from the given array address and index."
    }
}

#[derive(Clone)]
pub struct Push;

impl Instruction for Push {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let value = operands[0];
        state.store_variable(0, value)
    }

    fn name(&self) -> &'static str {
        "PUSH"
    }

    fn description(&self) -> &'static str {
        "Pushes a value onto the stack."
    }
}

#[derive(Clone)]
pub struct Pull;

impl Instruction for Pull {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let var_id = operands[0] as u8;
        let value = state.read_variable(0)?;
        state.store_variable(var_id, value)
    }

    fn name(&self) -> &'static str {
        "PULL"
    }

    fn description(&self) -> &'static str {
        "Pulls a value from the stack and stores it in the given variable."
    }
}

#[derive(Clone)]
pub struct Inc;

impl Instruction for Inc {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let var_id = operands[0] as u8;
        let value = state.read_variable(var_id)? as i16;
        state.store_variable(var_id, value.wrapping_add(1) as u16)
    }

    fn name(&self) -> &'static str {
        "INC"
    }

    fn description(&self) -> &'static str {
        "Increments the value in the given variable."
    }
}

#[derive(Clone)]
pub struct Dec;

impl Instruction for Dec {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let var_id = operands[0] as u8;
        let value = state.read_variable(var_id)? as i16;
        state.store_variable(var_id, value.wrapping_sub(1) as u16)
    }

    fn name(&self) -> &'static str {
        "DEC"
    }

    fn description(&self) -> &'static str {
        "Decrements the value in the given variable."
    }
}

#[derive(Clone)]
pub struct IncChk;

impl Instruction for IncChk {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let var_id = operands[0] as u8;
        let threshold = operands[1] as i16;
        let value = (state.read_variable(var_id)? as i16).wrapping_add(1);
        state.store_variable(var_id, value as u16)?;
        state.branch(value > threshold);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "INC_CHK"
    }

    fn description(&self) -> &'static str {
        "Increments the variable and jumps if it is greater than the threshold."
    }
}

#[derive(Clone)]
pub struct DecChk;

impl Instruction for DecChk {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let var_id = operands[0] as u8;
        let threshold = operands[1] as i16;
        let value = (state.read_variable(var_id)? as i16).wrapping_sub(1);
        state.store_variable(var_id, value as u16)?;
        state.branch(value < threshold);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "DEC_CHK"
    }

    fn description(&self) -> &'static str {
        "Decrements the variable and jumps if it is less than the threshold."
    }
}

#[derive(Clone)]
pub struct SetAttr;

impl Instruction for SetAttr {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let obj_id = operands[0];
        let attr = operands[1] as u8;
        
        if state.version <= 3 {
             let mut obj = Object::<u8>::new(&mut state.mem);
             obj.attr_set(obj_id as u8, attr);
        } else {
             let mut obj = Object::<u16>::new(&mut state.mem);
             obj.attr_set(obj_id, attr);
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "SET_ATTR"
    }

    fn description(&self) -> &'static str {
        "Sets the attribute of the object."
    }
}

#[derive(Clone)]
pub struct ClearAttr;

impl Instruction for ClearAttr {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let obj_id = operands[0];
        let attr = operands[1] as u8;
        
        if state.version <= 3 {
             let mut obj = Object::<u8>::new(&mut state.mem);
             obj.attr_clear(obj_id as u8, attr);
        } else {
             let mut obj = Object::<u16>::new(&mut state.mem);
             obj.attr_clear(obj_id, attr);
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "CLEAR_ATTR"
    }

    fn description(&self) -> &'static str {
        "Clears the attribute of the object."
    }
}

#[derive(Clone)]
pub struct TestAttr;

impl Instruction for TestAttr {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let obj_id = operands[0];
        let attr = operands[1] as u8;
        
        let result = if state.version <= 3 {
             let obj = Object::<u8>::new(&mut state.mem);
             obj.attr_test(obj_id as u8, attr)
        } else {
             let obj = Object::<u16>::new(&mut state.mem);
             obj.attr_test(obj_id as u8, attr)
        };
        
        state.branch(result);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "TEST_ATTR"
    }

    fn description(&self) -> &'static str {
        "Jumps if the object has the attribute set."
    }
}

#[derive(Clone)]
pub struct InsertObj;

impl Instruction for InsertObj {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let obj_id = operands[0];
        let dest_id = operands[1];
        
        if state.version <= 3 {
             let mut obj = Object::<u8>::new(&mut state.mem);
             obj.insert(obj_id as u8, dest_id as u8);
        } else {
             let mut obj = Object::<u16>::new(&mut state.mem);
             obj.insert(obj_id, dest_id);
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "INSERT_OBJ"
    }

    fn description(&self) -> &'static str {
        "Moves object to be the first child of the destination object."
    }
}

#[derive(Clone)]
pub struct RemoveObj;

impl Instruction for RemoveObj {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let obj_id = operands[0];
        
        if state.version <= 3 {
             let mut obj = Object::<u8>::new(&mut state.mem);
             obj.remove(obj_id as u8);
        } else {
             let mut obj = Object::<u16>::new(&mut state.mem);
             obj.remove(obj_id);
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "REMOVE_OBJ"
    }

    fn description(&self) -> &'static str {
        "Detaches the object from its parent."
    }
}

#[derive(Clone)]
pub struct GetParent;

impl Instruction for GetParent {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let obj_id = operands[0];
        
        let parent = if state.version <= 3 {
             let obj = Object::<u8>::new(&mut state.mem);
             obj.parent(obj_id as u8) as u16
        } else {
             let obj = Object::<u16>::new(&mut state.mem);
             obj.parent(obj_id)
        };
        
        let store_var = state.next_u8();
        state.store_variable(store_var, parent)
    }

    fn name(&self) -> &'static str {
        "GET_PARENT"
    }

    fn description(&self) -> &'static str {
        "Stores the parent of the object."
    }
}

#[derive(Clone)]
pub struct GetChild;

impl Instruction for GetChild {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let obj_id = operands[0];
        
        let child = if state.version <= 3 {
             let obj = Object::<u8>::new(&mut state.mem);
             obj.child(obj_id as u8) as u16
        } else {
             let obj = Object::<u16>::new(&mut state.mem);
             obj.child(obj_id)
        };
        
        let store_var = state.next_u8();
        state.store_variable(store_var, child)?;
        state.branch(child != 0);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "GET_CHILD"
    }

    fn description(&self) -> &'static str {
        "Stores the first child of the object and jumps if it exists."
    }
}

#[derive(Clone)]
pub struct GetSibling;

impl Instruction for GetSibling {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let obj_id = operands[0];
        
        let sibling = if state.version <= 3 {
             let obj = Object::<u8>::new(&mut state.mem);
             obj.sibling(obj_id as u8) as u16
        } else {
             let obj = Object::<u16>::new(&mut state.mem);
             obj.sibling(obj_id)
        };
        
        let store_var = state.next_u8();
        state.store_variable(store_var, sibling)?;
        state.branch(sibling != 0);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "GET_SIBLING"
    }

    fn description(&self) -> &'static str {
        "Stores the next sibling of the object and jumps if it exists."
    }
}

#[derive(Clone)]
pub struct GetProp;

impl Instruction for GetProp {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let obj_id = operands[0];
        let prop_id = operands[1] as u8;
        
        let value = if state.version <= 3 {
             let obj = Object::<u8>::new(&mut state.mem);
             obj.get_prop(obj_id as u8, prop_id)
        } else {
             let obj = Object::<u16>::new(&mut state.mem);
             obj.get_prop(obj_id, prop_id)
        };
        
        let store_var = state.next_u8();
        state.store_variable(store_var, value)
    }

    fn name(&self) -> &'static str {
        "GET_PROP"
    }

    fn description(&self) -> &'static str {
        "Stores the value of the property for the object."
    }
}

#[derive(Clone)]
pub struct GetPropAddr;

impl Instruction for GetPropAddr {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let obj_id = operands[0];
        let prop_id = operands[1] as u8;
        
        let addr = if state.version <= 3 {
             let obj = Object::<u8>::new(&mut state.mem);
             obj.get_prop_addr(obj_id as u8, prop_id).addr
        } else {
             let obj = Object::<u16>::new(&mut state.mem);
             obj.get_prop_addr(obj_id, prop_id).addr
        };
        
        let store_var = state.next_u8();
        state.store_variable(store_var, addr)
    }

    fn name(&self) -> &'static str {
        "GET_PROP_ADDR"
    }

    fn description(&self) -> &'static str {
        "Stores the address of the property data for the object."
    }
}

#[derive(Clone)]
pub struct GetPropLen;

impl Instruction for GetPropLen {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let prop_addr = operands[0];
        
        let len = if state.version <= 3 {
             let obj = Object::<u8>::new(&mut state.mem);
             obj.get_prop_len(prop_addr)
        } else {
             let obj = Object::<u16>::new(&mut state.mem);
             obj.get_prop_len(prop_addr)
        };
        
        let store_var = state.next_u8();
        state.store_variable(store_var, len as u16)
    }

    fn name(&self) -> &'static str {
        "GET_PROP_LEN"
    }

    fn description(&self) -> &'static str {
        "Stores the length of the property data at the given address."
    }
}

#[derive(Clone)]
pub struct GetNextProp;

impl Instruction for GetNextProp {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let obj_id = operands[0];
        let prop_id = operands[1] as u8;
        
        let next_prop = if state.version <= 3 {
             let obj = Object::<u8>::new(&mut state.mem);
             obj.get_prop_next(obj_id as u8, prop_id)
        } else {
             let obj = Object::<u16>::new(&mut state.mem);
             obj.get_prop_next(obj_id, prop_id)
        };
        
        let store_var = state.next_u8();
        state.store_variable(store_var, next_prop as u16)
    }

    fn name(&self) -> &'static str {
        "GET_NEXT_PROP"
    }

    fn description(&self) -> &'static str {
        "Stores the next property ID of the object."
    }
}

#[derive(Clone)]
pub struct PutProp;

impl Instruction for PutProp {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let obj_id = operands[0];
        let prop_id = operands[1] as u8;
        let value = operands[2];
        
        if state.version <= 3 {
             let obj = Object::<u8>::new(&mut state.mem);
             obj.put_prop(obj_id as u8, prop_id, value);
        } else {
             let obj = Object::<u16>::new(&mut state.mem);
             obj.put_prop(obj_id, prop_id, value);
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "PUT_PROP"
    }

    fn description(&self) -> &'static str {
        "Sets the value of the property for the object."
    }
}

#[derive(Clone)]
pub struct Call;

impl Instruction for Call {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let routine_addr = operands[0];
        let args = &operands[1..];
        
        if routine_addr == 0 {
            // Calling 0 returns false immediately
            let result_store = state.next_u8();
            state.store_variable(result_store, 0)?;
            return Ok(());
        }
        
        let packed_addr = routine_addr as u32 * 2; // TODO: V4/V5 packed address logic
        // But for now, assuming unpacked or V3 logic (addr * 2) matches ZMachine.java for V3?
        // ZMachine.java: pc *= PACK; (PACK=2 for V3)
        // Let's assume V3 for now or generic packed handling later.
        // Actually, operands are u16. Packed address * 2 might overflow u16, so use u32.
        
        // Read routine metadata
        let num_locals = state.mem.read_u8(packed_addr as u16);
        
        // Create new stack frame
        // PC starts after local count byte
        let mut new_pc = packed_addr + 1;
        
        // We read `store_var` now.
        let store_var = state.next_u8();
        
        // Push new frame with store var
        let new_frame = state.frame.clone().push(&mut state.mem.stack, new_pc, Some(store_var));
        
        // Initialize locals
        // V3: Locals are initialized with default values from the routine header
        // V4: Locals are initialized to 0
        if state.version <= 4 { // Actually V1-V4 use defaults? ZMachine.java says V5 zeroes, V1-V4 load defaults.
             for i in 0..num_locals {
                 let default_val = state.mem.read_u16(new_pc as u16);
                 new_pc += 2;
                 new_frame.write_local(&mut state.mem.stack, i as u16 + 1, default_val);
             }
        } else {
             // V5+: Locals zeroed
             for i in 0..num_locals {
                 new_frame.write_local(&mut state.mem.stack, i as u16 + 1, 0);
             }
        }
        
        // Overwrite locals with arguments
        for (i, arg) in args.iter().enumerate() {
            if i < num_locals as usize {
                new_frame.write_local(&mut state.mem.stack, i as u16 + 1, *arg);
            }
        }
        
        // Update state
        state.frame = new_frame;
        state.frame.pc = new_pc; // Set PC to start of instructions (after locals)
        
        // Read store variable for result (this is part of the CALL instruction in the OLD frame context)
        // BUT, the Z-Machine spec says the store variable is part of the call instruction.
        // And the result is stored AFTER returning.
        // So we need to save where to store the result in the stack frame?
        // StackFrame has `prev`... but doesn't explicitly store the "return result storage variable".
        // ZMachine.java `op_call`: "f.res = res;" where res is the store variable.
        // Rust StackFrame struct logic seems to be missing this 'res' field to store the return variable index.
        // We need to update StackFrame to hold this.
        
        // Let's hold off on modifying StackFrame in this atomic step.
        // Instead, let's assume `next_u8()` was called before this execute?
        // No, `state.next_u8()` reads from `state.frame.pc`.
        // We effectively just switched frames.
        // The `store_var` byte is at the PC of the *calling* frame (old frame).
        
        // We read `store_var` now.
        // Now we need to save `store_var` in the new frame so we know where to put the result when we return.
        // TODO: Update StackFrame to support `return_store_var`.
        
        Ok(())
    }

    fn name(&self) -> &'static str {
        "CALL"
    }

    fn description(&self) -> &'static str {
        "Calls a routine."
    }
}

#[derive(Clone)]
pub struct Ret;

impl Instruction for Ret {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let value = operands[0];
        
        // Pop frame
        let old_frame = std::mem::replace(&mut state.frame, StackFrame::main(&state.mem)); // Temp dummy
        let store_var = old_frame.return_store_var;
        let prev_frame = old_frame.pop(&mut state.mem.stack)?;
        state.frame = prev_frame;
        
        // Store result
        if let Some(var_id) = store_var {
            state.store_variable(var_id, value)?;
        }
        
        Ok(())
    }

    fn name(&self) -> &'static str {
        "RET"
    }

    fn description(&self) -> &'static str {
        "Returns from a routine with the given value."
    }
}

#[derive(Clone)]
pub struct RTrue;

impl Instruction for RTrue {
    fn execute(&self, state: &mut State, _operands: Vec<u16>) -> Result<(), Error> {
        let instr = Ret;
        instr.execute(state, vec![1])
    }

    fn name(&self) -> &'static str {
        "RTRUE"
    }

    fn description(&self) -> &'static str {
        "Returns true (1) from a routine."
    }
}

#[derive(Clone)]
pub struct RFalse;

impl Instruction for RFalse {
    fn execute(&self, state: &mut State, _operands: Vec<u16>) -> Result<(), Error> {
        let instr = Ret;
        instr.execute(state, vec![0])
    }

    fn name(&self) -> &'static str {
        "RFALSE"
    }

    fn description(&self) -> &'static str {
        "Returns false (0) from a routine."
    }
}

#[derive(Clone)]
pub struct RetPopped;

impl Instruction for RetPopped {
    fn execute(&self, state: &mut State, _operands: Vec<u16>) -> Result<(), Error> {
        let value = state.mem.load(0, &mut state.frame)?; // Pop from stack
        let instr = Ret;
        instr.execute(state, vec![value])
    }

    fn name(&self) -> &'static str {
        "RET_POPPED"
    }

    fn description(&self) -> &'static str {
        "Pops a value from the stack and returns it."
    }
}

#[derive(Clone)]
pub struct Restart;

impl Instruction for Restart {
    fn execute(&self, state: &mut State, _operands: Vec<u16>) -> Result<(), Error> {
        // TODO: Implement actual restart (reload memory, reset stack)
        // For now, just reset PC?
        // ZMachine.java reloads memory from backup.
        // We don't have backup memory implemented in State yet.
        println!("RESTART (Not fully implemented)");
        let _mem = Memory::new(&state.mem.deref().clone()); // Re-init from current mem? No, need original.
        state.frame = StackFrame::main(&state.mem);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "RESTART"
    }

    fn description(&self) -> &'static str {
        "Restarts the game."
    }
}

#[derive(Clone)]
pub struct Quit;

impl Instruction for Quit {
    fn execute(&self, state: &mut State, _operands: Vec<u16>) -> Result<(), Error> {
        state.running = false;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "QUIT"
    }

    fn description(&self) -> &'static str {
        "Terminates the game."
    }
}

use crate::zscii::Zscii;

#[derive(Clone)]
pub struct Print;

impl Instruction for Print {
    fn execute(&self, state: &mut State, _operands: Vec<u16>) -> Result<(), Error> {
        let mut zscii = Zscii::new(&state.mem);
        let s = zscii.get_string(state.frame.pc as u16);
        state.frame.pc = zscii.get_ptr() as u32;
        state.zscreen.print(s);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "PRINT"
    }

    fn description(&self) -> &'static str {
        "Prints a literal string from the instruction stream."
    }
}

#[derive(Clone)]
pub struct PrintRet;

impl Instruction for PrintRet {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let instr_print = Print;
        instr_print.execute(state, vec![])?;
        state.zscreen.newline();
        let instr_ret = Ret;
        instr_ret.execute(state, vec![1])
    }

    fn name(&self) -> &'static str {
        "PRINT_RET"
    }

    fn description(&self) -> &'static str {
        "Prints a literal string, a newline, and returns true."
    }
}

#[derive(Clone)]
pub struct PrintAddr;

impl Instruction for PrintAddr {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let addr = operands[0];
        let mut zscii = Zscii::new(&state.mem);
        let s = zscii.get_string(addr);
        state.zscreen.print(s);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "PRINT_ADDR"
    }

    fn description(&self) -> &'static str {
        "Prints a string from the given byte address."
    }
}

#[derive(Clone)]
pub struct PrintPAddr;

impl Instruction for PrintPAddr {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let paddr = operands[0];
        // TODO: Proper packed address logic for V4/V5
        let addr = paddr * 2; 
        let mut zscii = Zscii::new(&state.mem);
        let s = zscii.get_string(addr);
        state.zscreen.print(s);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "PRINT_PADDR"
    }

    fn description(&self) -> &'static str {
        "Prints a string from the given packed address."
    }
}

#[derive(Clone)]
pub struct PrintObj;

impl Instruction for PrintObj {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let obj_id = operands[0];
        let name = if state.version <= 3 {
            let mut obj = Object::<u8>::new(&mut state.mem);
            obj.name(obj_id as u8)
        } else {
            let mut obj = Object::<u16>::new(&mut state.mem);
            obj.name(obj_id)
        };
        
        if let Some(s) = name {
            state.zscreen.print(s);
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "PRINT_OBJ"
    }

    fn description(&self) -> &'static str {
        "Prints the name of the given object."
    }
}

#[derive(Clone)]
pub struct PrintChar;

impl Instruction for PrintChar {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let zscii_code = operands[0];
        // TODO: ZSCII to char conversion
        state.zscreen.print_char(zscii_code as u8 as char);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "PRINT_CHAR"
    }

    fn description(&self) -> &'static str {
        "Prints a single ZSCII character."
    }
}

#[derive(Clone)]
pub struct PrintNum;

impl Instruction for PrintNum {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let num = operands[0];
        state.zscreen.print_number(num);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "PRINT_NUM"
    }

    fn description(&self) -> &'static str {
        "Prints a signed 16-bit integer."
    }
}

#[derive(Clone)]
pub struct NewLine;

impl Instruction for NewLine {
    fn execute(&self, state: &mut State, _operands: Vec<u16>) -> Result<(), Error> {
        state.zscreen.newline();
        Ok(())
    }

    fn name(&self) -> &'static str {
        "NEW_LINE"
    }

    fn description(&self) -> &'static str {
        "Prints a newline."
    }
}

#[derive(Clone)]
pub struct Sread;

impl Instruction for Sread {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let text_buffer = operands[0];
        let parse_buffer = operands[1];
        
        let input = state.zscreen.readline();
        // TODO: Full Sread implementation (writing to buffer, tokenizing)
        // For now, just a placeholder
        println!("SREAD: {} into 0x{:X}", input, text_buffer);
        
        if state.version >= 4 {
            let result_var = state.next_u8();
            state.store_variable(result_var, 10)?; // 10 is typically the terminating character (newline)
        }
        
        Ok(())
    }

    fn name(&self) -> &'static str {
        "SREAD"
    }

    fn description(&self) -> &'static str {
        "Reads a line of input."
    }
}

#[derive(Clone)]
pub struct ReadChar;

impl Instruction for ReadChar {
    fn execute(&self, state: &mut State, _operands: Vec<u16>) -> Result<(), Error> {
        let ch = state.zscreen.read();
        let result_var = state.next_u8();
        state.store_variable(result_var, ch as u16)
    }

    fn name(&self) -> &'static str {
        "READ_CHAR"
    }

    fn description(&self) -> &'static str {
        "Reads a single character from the input."
    }
}

#[derive(Clone)]
pub struct SplitWindow;

impl Instruction for SplitWindow {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let lines = operands[0];
        state.zscreen.split_window(lines);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "SPLIT_WINDOW"
    }

    fn description(&self) -> &'static str {
        "Splits the screen into two windows."
    }
}

#[derive(Clone)]
pub struct SetWindow;

impl Instruction for SetWindow {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let window = operands[0];
        state.zscreen.set_window(window);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "SET_WINDOW"
    }

    fn description(&self) -> &'static str {
        "Sets the current window for output."
    }
}

#[derive(Clone)]
pub struct EraseWindow;

impl Instruction for EraseWindow {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let window = operands[0];
        state.zscreen.erase_window(window);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "ERASE_WINDOW"
    }

    fn description(&self) -> &'static str {
        "Erases the specified window."
    }
}

#[derive(Clone)]
pub struct MoveCursor;

impl Instruction for MoveCursor {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let line = operands[0];
        let column = operands[1];
        state.zscreen.move_cursor(column as u8, line as u8);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "MOVE_CURSOR"
    }

    fn description(&self) -> &'static str {
        "Moves the cursor to the specified position."
    }
}

#[derive(Clone)]
pub struct SetColor;

impl Instruction for SetColor {
    fn execute(&self, _state: &mut State, _operands: Vec<u16>) -> Result<(), Error> {
        // TODO: Implement color support in ZScreen
        Ok(())
    }

    fn name(&self) -> &'static str {
        "SET_COLOR"
    }

    fn description(&self) -> &'static str {
        "Sets the foreground and background colors."
    }
}

#[derive(Clone)]
pub struct Random;

impl Instruction for Random {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let range = operands[0] as i16;
        let result = if range <= 0 {
            state.zscreen.random(0); // Seed
            0
        } else {
            state.zscreen.random(range as u16)
        };
        
        let result_var = state.next_u8();
        state.store_variable(result_var, result)
    }

    fn name(&self) -> &'static str {
        "RANDOM"
    }

    fn description(&self) -> &'static str {
        "Generates a random number."
    }
}

#[derive(Clone)]
pub struct Save;

impl Instruction for Save {
    fn execute(&self, state: &mut State, _operands: Vec<u16>) -> Result<(), Error> {
        // TODO: Implement full serialization
        // For now, just a placeholder return false (failed)
        // V3 branch logic?
        // ZMachine.java: if(V4) BADOP... Branch(scr.Save(save()));
        // If V3, SAVE is a branching instruction on success.
        // If V4, it stores a result.
        
        // Let's assume V3 for now as we did for other branching
        // TODO: Proper version check and branch vs store logic
        
        if state.version <= 3 {
             state.branch(false); // Fail for now
        } else {
             let result_var = state.next_u8();
             state.store_variable(result_var, 0)?; // 0 = failure
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "SAVE"
    }

    fn description(&self) -> &'static str {
        "Saves the game state."
    }
}

#[derive(Clone)]
pub struct Restore;

impl Instruction for Restore {
    fn execute(&self, state: &mut State, _operands: Vec<u16>) -> Result<(), Error> {
        if state.version <= 3 {
             state.branch(false); // Fail for now
        } else {
             let result_var = state.next_u8();
             state.store_variable(result_var, 0)?; // 0 = failure
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "RESTORE"
    }

    fn description(&self) -> &'static str {
        "Restores the game state."
    }
}

#[derive(Clone)]
pub struct Verify;

impl Instruction for Verify {
    fn execute(&self, state: &mut State, _operands: Vec<u16>) -> Result<(), Error> {
        // Always verify true for now
        state.branch(true);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "VERIFY"
    }

    fn description(&self) -> &'static str {
        "Verifies the story file integrity."
    }
}

#[derive(Clone)]
pub struct ShowStatus;

impl Instruction for ShowStatus {
    fn execute(&self, state: &mut State, _operands: Vec<u16>) -> Result<(), Error> {
        // Only valid in V3
        if state.version <= 3 {
            // Logic to render status line
            // Need to read globals 17 (object) etc?
            // Actually, ZMachine.java calls UpdateStatus() which calls obj.status()
            // obj.status() reads global 16 (score/time object?)
            // For now, no-op or basic call to screen
            state.zscreen.set_status("Status Line Placeholder".to_string());
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "SHOW_STATUS"
    }

    fn description(&self) -> &'static str {
        "Updates the status line (V3 only)."
    }
}

#[derive(Clone)]
pub struct Je;

impl Instruction for Je {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let first = operands[0];
        let mut match_found = false;
        
        for &val in &operands[1..] {
            if val == first {
                match_found = true;
                break;
            }
        }
        
        state.branch(match_found);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "JE"
    }

    fn description(&self) -> &'static str {
        "Jumps if the first operand is equal to any of the subsequent operands."
    }
}

#[derive(Clone)]
pub struct Jz;

impl Instruction for Jz {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let a = operands[0];
        state.branch(a == 0);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "JZ"
    }

    fn description(&self) -> &'static str {
        "Jumps if the operand is zero."
    }
}

#[derive(Clone)]
pub struct Jl;

impl Instruction for Jl {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let a = operands[0] as i16;
        let b = operands[1] as i16;
        state.branch(a < b);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "JL"
    }

    fn description(&self) -> &'static str {
        "Jumps if the first operand is less than the second (signed)."
    }
}

#[derive(Clone)]
pub struct Jg;

impl Instruction for Jg {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let a = operands[0] as i16;
        let b = operands[1] as i16;
        state.branch(a > b);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "JG"
    }

    fn description(&self) -> &'static str {
        "Jumps if the first operand is greater than the second (signed)."
    }
}

#[derive(Clone)]
pub struct Jump;

impl Instruction for Jump {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let offset = operands[0] as i16;
        // PC is currently pointing to the next instruction (after fetch/decode).
        // JUMP offset is relative to the *operand* start?
        // ZMachine.java: f.pc = f.pc - 2 + ((short)a);
        // My next_u16 increments PC by 2.
        // So state.frame.pc is (Instr + 1 + 2).
        // If we subtract 2, we get address of operand.
        // So state.frame.pc = (state.frame.pc - 2) + offset.
        let pc = state.frame.pc as i32;
        state.frame.pc = (pc - 2 + offset as i32) as u32;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "JUMP"
    }

    fn description(&self) -> &'static str {
        "Unconditional jump to the given label (offset)."
    }
}

#[derive(Clone)]
pub struct Test;

impl Instruction for Test {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let a = operands[0];
        let b = operands[1];
        state.branch((a & b) == b);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "TEST"
    }

    fn description(&self) -> &'static str {
        "Jumps if all flags in the second operand are set in the first operand."
    }
}

#[derive(Clone)]
pub struct Jin;

impl Instruction for Jin {
    fn execute(&self, state: &mut State, operands: Vec<u16>) -> Result<(), Error> {
        let obj_a = operands[0];
        let obj_b = operands[1];
        
        // TODO: Move this logic to a proper Object handling module
        // Constants depend on version
        let v3 = state.version <= 3;
        let obj_size = if v3 { 9 } else { 14 };
        let parent_offset = if v3 { 4 } else { 6 };
        
        // Calculate Object Table locations
        // ZMachine.java: object_ptr = default_prop_ptr + obj.PropMax * 2 - obj.Size;
        // But simpler: Mem has object_table() address.
        // The object table header is 31 words (defaults).
        // Objects start after that.
        // But object numbering starts at 1.
        
        let object_table_base = state.mem.object_table();
        // Skip 31*2 bytes of default properties (62 bytes) for V3
        // Or 63*2 bytes (126 bytes) for V4+
        let prop_max = if v3 { 31 } else { 63 };
        let objects_start = object_table_base + (prop_max * 2);
        
        // Address of Obj A
        // Objects are 1-indexed
        if obj_a == 0 {
             state.branch(false);
             return Ok(());
        }
        
        let addr = objects_start + ((obj_a - 1) * obj_size);
        
        let parent = if v3 {
            state.mem.read_u8(addr + parent_offset) as u16
        } else {
            state.mem.read_u16(addr + parent_offset)
        };
        
        state.branch(parent == obj_b);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "JIN"
    }

    fn description(&self) -> &'static str {
        "Jumps if object a is inside object b (i.e. parent(a) == b)."
    }
}

#[cfg(test)]
mod instruction_tests;
