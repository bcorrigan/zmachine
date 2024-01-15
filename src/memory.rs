use crate::error::Error;
use std::ops::{Deref, DerefMut, Index, IndexMut};

pub struct Memory {
    mem: Vec<u8>,
    stack: Stack,
}

struct Stack {
    stack: [u16; 4096],
    sp: u16, //stack pointer
}

struct StackFrame {
    prev: Box<Option<StackFrame>>, //ie the call stack
    pc: u32,                       //program counter
    bp: u16, //base pointer of this stack frame - illegal for sp to drop below this
             //locals: Vec<u16>,
}

impl StackFrame {
    fn main(mem: &Memory) -> StackFrame {
        StackFrame {
            prev: Box::new(None),
            pc: mem.initial_pc() as u32,
            bp: 17,
        }
    }

    //advance stack pointer for new locals & create new stackframe
    fn push(self, stack: &mut Stack, pc: u32) -> StackFrame {
        stack.sp += 16; //16 locals
        StackFrame {
            prev: Box::new(Some(self)),
            pc,
            bp: stack.sp,
        }
    }

    //return to previous stack frame, zeroing out space used by this stack
    fn pop(self, stack: &mut Stack) -> Result<StackFrame, Error> {
        for p in stack.sp..=(self.bp - 16) {
            //erase current stack + 16 locals
            stack[p] = 0;
        }
        stack.sp = self.bp - 16;
        match *self.prev {
            Some(prev) => Ok(prev),
            None => Err(Error::ZMachineError(
                "Attempted to return from main routine".to_string(),
            )),
        }
    }

    fn read_local(&self, stack: &Stack, i: u16) -> u16 {
        stack[self.bp - i]
    }

    fn write_local(&self, stack: &mut Stack, i: u16, val: u16) {
        stack[self.bp - i] = val;
    }
}

impl Stack {
    fn pop(&mut self, frame: &StackFrame) -> Result<u16, Error> {
        if self.sp <= frame.bp {
            Err(Error::ZMachineError("Stack underflow".to_string()))
        } else {
            self.sp -= 1;
            Ok(self[self.sp])
        }
    }

    fn push(&mut self, _frame: &mut StackFrame, val: u16) -> Result<(), Error> {
        let new_sp = self.sp + 1;
        self.sp = new_sp;
        if new_sp > self.stack.len() as u16 {
            Err(Error::ZMachineError("Stack Overflow".to_string()))
        } else {
            self[new_sp] = val;
            Ok(())
        }
    }
}

impl Memory {
    pub fn new(story: &[u8]) -> Self {
        Memory {
            mem: story.into(),
            stack: Stack {
                stack: [0u16; 4096],
                sp: 0,
            },
        }
    }

    //First define various read_* and write_* fns
    pub fn read_u8(&self, addr: u16) -> u8 {
        self[addr]
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        (self[addr] as u16) << 8 | self[addr + 1] as u16
    }

    pub fn read_u32(&self, addr: u16) -> u32 {
        (self[addr] as u32) << 24
            | (self[addr + 1] as u32) << 16
            | (self[addr + 2] as u32) << 8
            | self[addr + 3] as u32
    }

    pub fn write_u32(&mut self, addr: u16, val: u32) {
        let bytes = val.to_be_bytes();
        self[addr] = bytes[0];
        self[addr + 1] = bytes[1];
        self[addr + 2] = bytes[2];
        self[addr + 3] = bytes[3];
    }

    //TODO various legality checks as some areas of memory have write restrictions
    pub fn write_u8(&mut self, addr: u16, val: u8) {
        self[addr] = val;
    }

    pub fn write_u16(&mut self, addr: u16, val: u16) {
        let vals = val.to_be_bytes();
        self[addr] = vals[0];
        self[addr] = vals[1];
    }

    pub fn load(&mut self, id: u8, frame: &mut StackFrame) -> Result<u16, Error> {
        match id {
            0x00 => {
                //pop from stack
                self.stack.pop(frame)
            }
            0x01..=0x0f => {
                //read from locals
                Ok(frame.read_local(&self.stack, id as u16))
            }
            _ => {
                //read from globals
                Ok(self.read_global(id))
            }
        }
    }

    pub fn read_global(&self, id: u8) -> u16 {
        self.read_u16(self.global_variables() + (id * 2) as u16)
    }

    pub fn high_memory(&self) -> u16 {
        self.read_u16(0x04)
    }

    pub fn static_memory(&self) -> u16 {
        self.read_u16(0x0E)
    }

    pub fn object_table(&self) -> u16 {
        self.read_u16(0x0A)
    }

    pub fn dictionary(&self) -> u16 {
        self.read_u16(0x08)
    }

    pub fn global_variables(&self) -> u16 {
        self.read_u16(0x0C)
    }

    pub fn character_table(&self) -> u16 {
        self.read_u16(0x2E)
    }

    pub fn alphabet_table(&self) -> u16 {
        self.read_u16(0x34) //"or zero for default"
    }

    pub fn abbreviations_table(&self) -> u16 {
        self.read_u16(0x18)
    }

    pub fn header_extension_table(&self) -> u16 {
        self.read_u16(0x36)
    }

    pub fn routine_offset(&self) -> u16 {
        self.read_u16(0x28)
    }

    pub fn string_offset(&self) -> u16 {
        self.read_u16(0x2A)
    }

    pub fn zmachine_version(&self) -> u8 {
        self.read_u8(0x00)
    }

    //in v6 and above this is a packed address but that handling is left to processor
    pub fn initial_pc(&self) -> u16 {
        self.read_u16(0x06)
    }
}

//VARIOUS BOILERPLATE

impl Deref for Memory {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.mem
    }
}

impl DerefMut for Memory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mem
    }
}

impl Index<u16> for Memory {
    type Output = u8;

    fn index(&self, i: u16) -> &Self::Output {
        &self.mem[i as usize]
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, i: u16) -> &mut u8 {
        &mut self.mem[i as usize]
    }
}

impl Index<u32> for Memory {
    type Output = u8;

    fn index(&self, i: u32) -> &Self::Output {
        &self.mem[i as usize]
    }
}

impl IndexMut<u32> for Memory {
    fn index_mut(&mut self, i: u32) -> &mut u8 {
        &mut self.mem[i as usize]
    }
}

impl Deref for Stack {
    type Target = [u16; 4096];

    fn deref(&self) -> &Self::Target {
        &self.stack
    }
}

impl DerefMut for Stack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stack
    }
}

impl Index<u16> for Stack {
    type Output = u16;

    fn index(&self, i: u16) -> &Self::Output {
        &self.stack[i as usize]
    }
}

impl IndexMut<u16> for Stack {
    fn index_mut(&mut self, i: u16) -> &mut u16 {
        &mut self.stack[i as usize]
    }
}

impl Index<u32> for Stack {
    type Output = u16;

    fn index(&self, i: u32) -> &Self::Output {
        &self.stack[i as usize]
    }
}

impl IndexMut<u32> for Stack {
    fn index_mut(&mut self, i: u32) -> &mut u16 {
        &mut self.stack[i as usize]
    }
}
