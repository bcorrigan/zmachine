
use std::ops::{Deref, DerefMut, Index, IndexMut};
struct Memory {
    mem: Vec<u8>,
    stack: Stack,
}

struct Stack {
    stack: [u16; 4096],
}

struct StackFrame<'a> {
    prev: &'a Option<StackFrame<'a>>, //ie the call stack
    pc: u32, //program counter
    bp: u32, //base pointer of this stack frame
    locals: Vec<u16>,
}


impl Memory {
    pub fn new(story: &[u8]) -> Self {
        Memory {
            mem: story.into(),
            stack: Stack { stack: [0u16; 4096] }
        }
    }

    //First define various read_* and write_* fns
    fn read_U8(self, addr:u16) -> u8 {
        self[addr]
    }

    fn read_U16(self, addr:u16) -> u16 {
        (self[addr] as u16) << 8 | self[addr + 1] as u16
    }

    fn read_U32(self, addr:u16) -> u32 {
        (self[addr] as u32) << 24 | 
        (self[addr + 1] as u32)  << 16 |
        (self[addr + 2] as u32)  << 8 |
         self[addr + 3] as u32
    }

    //TODO various legality checks as some areas of memory have write restrictions
    fn write_U8(mut self, addr:u16, val:u8) {
        self[addr] = val;
    }

    fn write_U16(mut self, addr:u16, val:u16) {
        let vals = val.to_be_bytes();
        self[addr] = vals[0];
        self[addr] = vals[1];
    }

    fn load(mut self, id: u8, frame: &StackFrame) -> u16 {
        match id {
            0x00 => { //pop from stack
                self.stack.pop(frame)
            },
            0x01..0x0f => { //read from locals
                frame.locals[id]
            },
            _ => { //read from globals
                self.read_U16( self.global_variables() + (id * 2) as u16 )
            }
        }
    }


    fn high_memory(self) -> u16 {
        self.read_U16(0x04)
    }

    fn static_memory(self) -> u16 {
        self.read_U16(0x0E)
    }

    fn object_table(self) -> u16 {
        self.read_U16(0x0A)
    }

    fn dictionary(self) -> u16 {
        self.read_U16(0x08)
    }

    fn global_variables(self) -> u16 {
        self.read_U16(0x0C)
    }

    fn character_table(self) -> u16 {
        self.read_U16(0x2E)
    }

    fn alphabet_table(self) -> u16 {
        self.read_U16(0x34) //"or zero for default"
    }

    fn abbreviations_table(self) -> u16 {
        self.read_U16(0x18)
    }

    fn header_extension_table(self) -> u16 {
        self.read_U16(0x36)
    }

    fn routine_offset(self) -> u16 {
        self.read_U16(0x28)
    }

    fn string_offset(self) -> u16 {
        self.read_U16(0x2A)
    }

    pub fn zmachine_version(self) -> u8 {
        self.read_U8(0x00)
    }

    //in v6 and above this is a packed address but that handling is left to processor
    pub fn initial_pc(self) -> u16 {
        self.read_U16(0x06)
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