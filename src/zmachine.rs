use crate::memory::{Memory, StackFrame};
use crate::zscreen::ZScreen;
use crate::instruction::Instruction;
use crate::error::Error;

pub struct State<'a> {
    pub mem: Memory,
    pub zscreen: Box<dyn ZScreen + 'a>,
    pub frame: StackFrame,
    pub running: bool,
    pub version: u8,
}

impl<'a> State<'a> {
    pub fn new(story: &[u8], zscreen: Box<dyn ZScreen + 'a>) -> Self {
        let mem = Memory::new(story);
        let frame = StackFrame::main(&mem);
        let version = mem.zmachine_version();
        State {
            mem,
            zscreen,
            frame,
            running: true,
            version,
        }
    }

    pub fn next_u8(&mut self) -> u8 {
        let val = self.mem.read_u8(self.frame.pc as u16); // TODO: PC is u32, handle high mem?
        self.frame.pc = self.frame.pc.wrapping_add(1);
        val
    }

    pub fn next_u16(&mut self) -> u16 {
        let val = self.mem.read_u16(self.frame.pc as u16);
        self.frame.pc = self.frame.pc.wrapping_add(2);
        val
    }

    pub fn read_variable(&mut self, id: u8) -> Result<u16, Error> {
        self.mem.load(id, &mut self.frame)
    }
    
    pub fn store_variable(&mut self, id: u8, val: u16) -> Result<(), Error> {
        self.mem.store(id, val, &mut self.frame)
    }

    pub fn branch(&mut self, condition: bool) {
        let offset_byte_1 = self.next_u8();
        
        // Logic from ZMachine.java Branch(cond)
        // int x = U8(frame.pc++);
        // if((x & 0x80) == 0) cond = !cond;
        let branch_on_true = (offset_byte_1 & 0x80) != 0;
        
        if branch_on_true == condition {
            // Condition matched, take branch
            let offset: i16;
            if (offset_byte_1 & 0x40) != 0 {
                // Short branch (6 bits)
                offset = (offset_byte_1 & 0x3f) as i16;
            } else {
                 // Long branch (14 bits, signed)
                 // x = x | 0xffffffc0; ?? Java logic is weird for sign extension
                 // x = (x << 8) | U8(frame.pc++);
                 
                 let offset_byte_2 = self.next_u8();
                 let val = ((offset_byte_1 & 0x3f) as u16) << 8 | (offset_byte_2 as u16);
                 
                 // Sign extend 14-bit value
                 if (val & 0x2000) != 0 {
                     offset = (val | 0xC000) as i16;
                 } else {
                     offset = val as i16;
                 }
            }

            if offset == 0 {
                // Return 0 (RFALSE)
                // TODO: Implement Return logic or fake it
                // self.return_val(0);
                println!("BRANCH RETURN 0 (Not Implemented)");
            } else if offset == 1 {
                // Return 1 (RTRUE)
                // self.return_val(1);
                 println!("BRANCH RETURN 1 (Not Implemented)");
            } else {
                // Branch to offset
                // frame.pc = frame.pc + x - 2;
                // Note: The offset is calculated from the address of the branch instruction 
                // but the PC has already advanced past the offset bytes.
                // ZMachine.java says: frame.pc = frame.pc + x - 2;
                // because it incremented PC while reading offset bytes.
                // My next_u8 also increments PC.
                // So this logic should hold.
                 self.frame.pc = (self.frame.pc as i32 + offset as i32 - 2) as u32;
            }
        } else {
            // Condition failed, skip branch info
             if (offset_byte_1 & 0x40) == 0 {
                 // It was a long branch, skip the second byte
                 self.next_u8();
             }
        }
    }
}

pub struct ZMachine<'a> {
    pub state: State<'a>,
    pub instructions: Vec<Box<dyn Instruction>>,
}

impl<'a> ZMachine<'a> {
    pub fn new(story: &[u8], zscreen: Box<dyn ZScreen + 'a>) -> Self {
        let mut instructions: Vec<Box<dyn Instruction>> = vec![];
        for i in 0..=255 {
            instructions.push(Box::new(crate::instruction::IllegalInstruction(i as u8)));
        }

        // Register JE (Opcode 1)
        // 2OP forms: 0x01, 0x21, 0x41, 0x61
        // Variable form: 0xC1
        instructions[0x01] = Box::new(crate::instruction::Je);
        instructions[0x21] = Box::new(crate::instruction::Je);
        instructions[0x41] = Box::new(crate::instruction::Je);
        instructions[0x61] = Box::new(crate::instruction::Je);
        instructions[0xC1] = Box::new(crate::instruction::Je);

        // Register JZ (Opcode 128 -> 0x80)
        // 1OP forms: 0x80, 0x90, 0xA0
        instructions[0x80] = Box::new(crate::instruction::Jz);
        instructions[0x90] = Box::new(crate::instruction::Jz);
        instructions[0xA0] = Box::new(crate::instruction::Jz);

        // Register JL (Opcode 2 -> 0x02)
        // 2OP forms: 0x02, 0x22, 0x42, 0x62
        // Variable form: 0xC2
        instructions[0x02] = Box::new(crate::instruction::Jl);
        instructions[0x22] = Box::new(crate::instruction::Jl);
        instructions[0x42] = Box::new(crate::instruction::Jl);
        instructions[0x62] = Box::new(crate::instruction::Jl);
        instructions[0xC2] = Box::new(crate::instruction::Jl);

        // Register JG (Opcode 3 -> 0x03)
        // 2OP forms: 0x03, 0x23, 0x43, 0x63
        // Variable form: 0xC3
        instructions[0x03] = Box::new(crate::instruction::Jg);
        instructions[0x23] = Box::new(crate::instruction::Jg);
        instructions[0x43] = Box::new(crate::instruction::Jg);
        instructions[0x63] = Box::new(crate::instruction::Jg);
        instructions[0xC3] = Box::new(crate::instruction::Jg);

        // Register JUMP (Opcode 140 -> 0x8C)
        // 1OP forms: 0x8C, 0x9C, 0xAC
        instructions[0x8C] = Box::new(crate::instruction::Jump);
        instructions[0x9C] = Box::new(crate::instruction::Jump);
        instructions[0xAC] = Box::new(crate::instruction::Jump);

        // Register JIN (Opcode 6 -> 0x06)
        // 2OP forms: 0x06, 0x26, 0x46, 0x66
        // Variable form: 0xC6
        instructions[0x06] = Box::new(crate::instruction::Jin);
        instructions[0x26] = Box::new(crate::instruction::Jin);
        instructions[0x46] = Box::new(crate::instruction::Jin);
        instructions[0x66] = Box::new(crate::instruction::Jin);
        instructions[0xC6] = Box::new(crate::instruction::Jin);

        // Register TEST (Opcode 7 -> 0x07)
        // 2OP forms: 0x07, 0x27, 0x47, 0x67
        // Variable form: 0xC7
        instructions[0x07] = Box::new(crate::instruction::Test);
        instructions[0x27] = Box::new(crate::instruction::Test);
        instructions[0x47] = Box::new(crate::instruction::Test);
        instructions[0x67] = Box::new(crate::instruction::Test);
        instructions[0xC7] = Box::new(crate::instruction::Test);

        // Register STORE (0x0D)
        instructions[0x0D] = Box::new(crate::instruction::Store);
        instructions[0x2D] = Box::new(crate::instruction::Store);
        instructions[0x4D] = Box::new(crate::instruction::Store);
        instructions[0x6D] = Box::new(crate::instruction::Store);
        instructions[0xCD] = Box::new(crate::instruction::Store);

        // Register LOAD (0x8E)
        instructions[0x8E] = Box::new(crate::instruction::Load);
        instructions[0x9E] = Box::new(crate::instruction::Load);
        instructions[0xAE] = Box::new(crate::instruction::Load);

        // Register STOREW (0xE1)
        instructions[0xE1] = Box::new(crate::instruction::StoreW);

        // Register STOREB (0xE2)
        instructions[0xE2] = Box::new(crate::instruction::StoreB);

        // Register LOADW (0x0F)
        instructions[0x0F] = Box::new(crate::instruction::LoadW);
        instructions[0x2F] = Box::new(crate::instruction::LoadW);
        instructions[0x4F] = Box::new(crate::instruction::LoadW);
        instructions[0x6F] = Box::new(crate::instruction::LoadW);
        instructions[0xCF] = Box::new(crate::instruction::LoadW);

        // Register LOADB (0x10)
        instructions[0x10] = Box::new(crate::instruction::LoadB);
        instructions[0x30] = Box::new(crate::instruction::LoadB);
        instructions[0x50] = Box::new(crate::instruction::LoadB);
        instructions[0x70] = Box::new(crate::instruction::LoadB);
        instructions[0xD0] = Box::new(crate::instruction::LoadB);

        // Register SET_ATTR (0x0B)
        instructions[0x0B] = Box::new(crate::instruction::SetAttr);
        instructions[0x2B] = Box::new(crate::instruction::SetAttr);
        instructions[0x4B] = Box::new(crate::instruction::SetAttr);
        instructions[0x6B] = Box::new(crate::instruction::SetAttr);
        instructions[0xCB] = Box::new(crate::instruction::SetAttr);

        // Register CLEAR_ATTR (0x0C)
        instructions[0x0C] = Box::new(crate::instruction::ClearAttr);
        instructions[0x2C] = Box::new(crate::instruction::ClearAttr);
        instructions[0x4C] = Box::new(crate::instruction::ClearAttr);
        instructions[0x6C] = Box::new(crate::instruction::ClearAttr);
        instructions[0xCC] = Box::new(crate::instruction::ClearAttr);

        // Register TEST_ATTR (0x0A)
        instructions[0x0A] = Box::new(crate::instruction::TestAttr);
        instructions[0x2A] = Box::new(crate::instruction::TestAttr);
        instructions[0x4A] = Box::new(crate::instruction::TestAttr);
        instructions[0x6A] = Box::new(crate::instruction::TestAttr);
        instructions[0xCA] = Box::new(crate::instruction::TestAttr);

        // Register INSERT_OBJ (0x0E)
        instructions[0x0E] = Box::new(crate::instruction::InsertObj);
        instructions[0x2E] = Box::new(crate::instruction::InsertObj);
        instructions[0x4E] = Box::new(crate::instruction::InsertObj);
        instructions[0x6E] = Box::new(crate::instruction::InsertObj);
        instructions[0xCE] = Box::new(crate::instruction::InsertObj);

        // Register REMOVE_OBJ (0x89)
        instructions[0x89] = Box::new(crate::instruction::RemoveObj);
        instructions[0x99] = Box::new(crate::instruction::RemoveObj);
        instructions[0xA9] = Box::new(crate::instruction::RemoveObj);

        // Register GET_PARENT (0x83)
        instructions[0x83] = Box::new(crate::instruction::GetParent);
        instructions[0x93] = Box::new(crate::instruction::GetParent);
        instructions[0xA3] = Box::new(crate::instruction::GetParent);

        // Register GET_CHILD (0x82)
        instructions[0x82] = Box::new(crate::instruction::GetChild);
        instructions[0x92] = Box::new(crate::instruction::GetChild);
        instructions[0xA2] = Box::new(crate::instruction::GetChild);

        // Register GET_SIBLING (0x81)
        instructions[0x81] = Box::new(crate::instruction::GetSibling);
        instructions[0x91] = Box::new(crate::instruction::GetSibling);
        instructions[0xA1] = Box::new(crate::instruction::GetSibling);

        // Register GET_PROP (0x11)
        instructions[0x11] = Box::new(crate::instruction::GetProp);
        instructions[0x31] = Box::new(crate::instruction::GetProp);
        instructions[0x51] = Box::new(crate::instruction::GetProp);
        instructions[0x71] = Box::new(crate::instruction::GetProp);
        instructions[0xD1] = Box::new(crate::instruction::GetProp);

        // Register GET_PROP_ADDR (0x12)
        instructions[0x12] = Box::new(crate::instruction::GetPropAddr);
        instructions[0x32] = Box::new(crate::instruction::GetPropAddr);
        instructions[0x52] = Box::new(crate::instruction::GetPropAddr);
        instructions[0x72] = Box::new(crate::instruction::GetPropAddr);
        instructions[0xD2] = Box::new(crate::instruction::GetPropAddr);

        // Register GET_PROP_LEN (0x84)
        instructions[0x84] = Box::new(crate::instruction::GetPropLen);
        instructions[0x94] = Box::new(crate::instruction::GetPropLen);
        instructions[0xA4] = Box::new(crate::instruction::GetPropLen);

        // Register GET_NEXT_PROP (0x13)
        instructions[0x13] = Box::new(crate::instruction::GetNextProp);
        instructions[0x33] = Box::new(crate::instruction::GetNextProp);
        instructions[0x53] = Box::new(crate::instruction::GetNextProp);
        instructions[0x73] = Box::new(crate::instruction::GetNextProp);
        instructions[0xD3] = Box::new(crate::instruction::GetNextProp);

        // Register PUT_PROP (0xE3)
        instructions[0xE3] = Box::new(crate::instruction::PutProp);

        // Register CALL (VAR:E0 -> 0xE0, 1OP:88 -> 0x88, etc)
        // CALL_VS (0xE0)
        instructions[0xE0] = Box::new(crate::instruction::Call);
        // CALL_1S (0x88)
        instructions[0x88] = Box::new(crate::instruction::Call);
        instructions[0x98] = Box::new(crate::instruction::Call);
        instructions[0xA8] = Box::new(crate::instruction::Call);
        // CALL_2S (0x19)
        instructions[0x19] = Box::new(crate::instruction::Call);
        instructions[0x39] = Box::new(crate::instruction::Call);
        instructions[0x59] = Box::new(crate::instruction::Call);
        instructions[0x79] = Box::new(crate::instruction::Call);
        instructions[0xD9] = Box::new(crate::instruction::Call);
        // CALL_VN (0xF9) - V5+ ? Logic might be different (no store?)
        // Instructions like 2N, VN don't store.
        // For now, mapping all CALLs to the same struct.
        // We might need `CallN` for calls that don't store.
        
        // Register RET (0x8B)
        instructions[0x8B] = Box::new(crate::instruction::Ret);
        instructions[0x9B] = Box::new(crate::instruction::Ret);
        instructions[0xAB] = Box::new(crate::instruction::Ret);

        // Register RTRUE (0xB0)
        instructions[0xB0] = Box::new(crate::instruction::RTrue);

        // Register RFALSE (0xB1)
        instructions[0xB1] = Box::new(crate::instruction::RFalse);

        // Register RET_POPPED (0xB8)
        instructions[0xB8] = Box::new(crate::instruction::RetPopped);

        // Register RESTART (0xB7)
        instructions[0xB7] = Box::new(crate::instruction::Restart);

        // Register QUIT (0xBA)
        instructions[0xBA] = Box::new(crate::instruction::Quit);

        // Register PRINT (0xB2)
        instructions[0xB2] = Box::new(crate::instruction::Print);

        // Register PRINT_RET (0xB3)
        instructions[0xB3] = Box::new(crate::instruction::PrintRet);

        // Register PRINT_ADDR (0x87)
        instructions[0x87] = Box::new(crate::instruction::PrintAddr);
        instructions[0x97] = Box::new(crate::instruction::PrintAddr);
        instructions[0xA7] = Box::new(crate::instruction::PrintAddr);

        // Register PRINT_PADDR (0x8D)
        instructions[0x8D] = Box::new(crate::instruction::PrintPAddr);
        instructions[0x9D] = Box::new(crate::instruction::PrintPAddr);
        instructions[0xAD] = Box::new(crate::instruction::PrintPAddr);

        // Register PRINT_OBJ (0x8A)
        instructions[0x8A] = Box::new(crate::instruction::PrintObj);
        instructions[0x9A] = Box::new(crate::instruction::PrintObj);
        instructions[0xAA] = Box::new(crate::instruction::PrintObj);

        // Register PRINT_CHAR (0xE5)
        instructions[0xE5] = Box::new(crate::instruction::PrintChar);

        // Register PRINT_NUM (0xE6)
        instructions[0xE6] = Box::new(crate::instruction::PrintNum);

        // Register NEW_LINE (0xBB)
        instructions[0xBB] = Box::new(crate::instruction::NewLine);

        // Register SREAD (0xE4)
        instructions[0xE4] = Box::new(crate::instruction::Sread);

        // Register READ_CHAR (0xF6)
        instructions[0xF6] = Box::new(crate::instruction::ReadChar);

        // Register PUSH (0xE8)
        instructions[0xE8] = Box::new(crate::instruction::Push);

        // Register PULL (0xE9)
        instructions[0xE9] = Box::new(crate::instruction::Pull);

        // Register INC (0x85)
        instructions[0x85] = Box::new(crate::instruction::Inc);
        instructions[0x95] = Box::new(crate::instruction::Inc);
        instructions[0xA5] = Box::new(crate::instruction::Inc);

        // Register DEC (0x86)
        instructions[0x86] = Box::new(crate::instruction::Dec);
        instructions[0x96] = Box::new(crate::instruction::Dec);
        instructions[0xA6] = Box::new(crate::instruction::Dec);

        // Register INC_CHK (0x05)
        instructions[0x05] = Box::new(crate::instruction::IncChk);
        instructions[0x25] = Box::new(crate::instruction::IncChk);
        instructions[0x45] = Box::new(crate::instruction::IncChk);
        instructions[0x65] = Box::new(crate::instruction::IncChk);
        instructions[0xC5] = Box::new(crate::instruction::IncChk);

        // Register DEC_CHK (0x04)
        instructions[0x04] = Box::new(crate::instruction::DecChk);
        instructions[0x24] = Box::new(crate::instruction::DecChk);
        instructions[0x44] = Box::new(crate::instruction::DecChk);
        instructions[0x64] = Box::new(crate::instruction::DecChk);
        instructions[0xC4] = Box::new(crate::instruction::DecChk);

        // Register SPLIT_WINDOW (0xEA)
        instructions[0xEA] = Box::new(crate::instruction::SplitWindow);

        // Register SET_WINDOW (0xEB)
        instructions[0xEB] = Box::new(crate::instruction::SetWindow);

        // Register ERASE_WINDOW (0xED)
        instructions[0xED] = Box::new(crate::instruction::EraseWindow);

        // Register MOVE_CURSOR (0xEF)
        instructions[0xEF] = Box::new(crate::instruction::MoveCursor);

        // Register SET_COLOR (0x1B)
        instructions[0x1B] = Box::new(crate::instruction::SetColor);
        instructions[0x3B] = Box::new(crate::instruction::SetColor);
        instructions[0x5B] = Box::new(crate::instruction::SetColor);
        instructions[0x7B] = Box::new(crate::instruction::SetColor);
        instructions[0xDB] = Box::new(crate::instruction::SetColor);

        // Register RANDOM (0xE7)
        instructions[0xE7] = Box::new(crate::instruction::Random);

        // Register SAVE (0xB5)
        instructions[0xB5] = Box::new(crate::instruction::Save);

        // Register RESTORE (0xB6)
        instructions[0xB6] = Box::new(crate::instruction::Restore);

        // Register VERIFY (0xBD)
        instructions[0xBD] = Box::new(crate::instruction::Verify);

        // Register SHOW_STATUS (0xBC)
        instructions[0xBC] = Box::new(crate::instruction::ShowStatus);

        // Register ADD (0x14)
        // 2OP forms: 0x14, 0x34, 0x54, 0x74
        // Variable form: 0xD4
        instructions[0x14] = Box::new(crate::instruction::Add);
        instructions[0x34] = Box::new(crate::instruction::Add);
        instructions[0x54] = Box::new(crate::instruction::Add);
        instructions[0x74] = Box::new(crate::instruction::Add);
        instructions[0xD4] = Box::new(crate::instruction::Add);

        // Register SUB (Opcode 21 -> 0x15)
        instructions[0x15] = Box::new(crate::instruction::Sub);
        instructions[0x35] = Box::new(crate::instruction::Sub);
        instructions[0x55] = Box::new(crate::instruction::Sub);
        instructions[0x75] = Box::new(crate::instruction::Sub);
        instructions[0xD5] = Box::new(crate::instruction::Sub);

        // Register MUL (Opcode 22 -> 0x16)
        instructions[0x16] = Box::new(crate::instruction::Mul);
        instructions[0x36] = Box::new(crate::instruction::Mul);
        instructions[0x56] = Box::new(crate::instruction::Mul);
        instructions[0x76] = Box::new(crate::instruction::Mul);
        instructions[0xD6] = Box::new(crate::instruction::Mul);

        // Register DIV (Opcode 23 -> 0x17)
        instructions[0x17] = Box::new(crate::instruction::Div);
        instructions[0x37] = Box::new(crate::instruction::Div);
        instructions[0x57] = Box::new(crate::instruction::Div);
        instructions[0x77] = Box::new(crate::instruction::Div);
        instructions[0xD7] = Box::new(crate::instruction::Div);

        // Register MOD (Opcode 24 -> 0x18)
        instructions[0x18] = Box::new(crate::instruction::Mod);
        instructions[0x38] = Box::new(crate::instruction::Mod);
        instructions[0x58] = Box::new(crate::instruction::Mod);
        instructions[0x78] = Box::new(crate::instruction::Mod);
        instructions[0xD8] = Box::new(crate::instruction::Mod);

        // Register OR (Opcode 8 -> 0x08)
        instructions[0x08] = Box::new(crate::instruction::Or);
        instructions[0x28] = Box::new(crate::instruction::Or);
        instructions[0x48] = Box::new(crate::instruction::Or);
        instructions[0x68] = Box::new(crate::instruction::Or);
        instructions[0xC8] = Box::new(crate::instruction::Or);

        // Register AND (Opcode 9 -> 0x09)
        instructions[0x09] = Box::new(crate::instruction::And);
        instructions[0x29] = Box::new(crate::instruction::And);
        instructions[0x49] = Box::new(crate::instruction::And);
        instructions[0x69] = Box::new(crate::instruction::And);
        instructions[0xC9] = Box::new(crate::instruction::And);

        // Register NOT (Opcode 143/248 -> 0x8F, 0xF8)
        // 1OP forms: 0x8F, 0x9F, 0xAF
        // VAR form: 0xF8 (V5/6)
        instructions[0x8F] = Box::new(crate::instruction::Not);
        instructions[0x9F] = Box::new(crate::instruction::Not);
        instructions[0xAF] = Box::new(crate::instruction::Not);
        instructions[0xF8] = Box::new(crate::instruction::Not);
        
        ZMachine {
            state: State::new(story, zscreen),
            instructions,
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        while self.state.running {
            self.execute_instruction()?;
        }
        Ok(())
    }

    fn execute_instruction(&mut self) -> Result<(), Error> {
        let op = self.state.next_u8();
        let mut operands: Vec<u16> = Vec::new();
        
        let op_high = op >> 4;
        
        // DECODE OPERANDS
        // Based on ZMachine.java logic
        match op_high {
            0x00 | 0x01 => { // 2OP, sconst, sconst
                 operands.push(self.state.next_u8() as u16);
                 operands.push(self.state.next_u8() as u16);
            }
            0x02 | 0x03 => { // 2OP, sconst, var
                 operands.push(self.state.next_u8() as u16);
                 let var_id = self.state.next_u8();
                 operands.push(self.state.read_variable(var_id)?);
            }
            0x04 | 0x05 => { // 2OP, var, sconst
                 let var_id = self.state.next_u8();
                 operands.push(self.state.read_variable(var_id)?);
                 operands.push(self.state.next_u8() as u16);
            }
             0x06 | 0x07 => { // 2OP, var, var
                 let var_id1 = self.state.next_u8();
                 operands.push(self.state.read_variable(var_id1)?);
                 let var_id2 = self.state.next_u8();
                 operands.push(self.state.read_variable(var_id2)?);
            }
            0x08 => { // 1OP, lconst (word)
                 operands.push(self.state.next_u16());
            }
            0x09 => { // 1OP, sconst
                 operands.push(self.state.next_u8() as u16);
            }
            0x0A => { // 1OP, var
                 let var_id = self.state.next_u8();
                 operands.push(self.state.read_variable(var_id)?);
            }
            0x0B => { // 0OP or Extended
                 if op == 0xBE {
                      return Err(Error::ZMachineError("Extended opcode 0xBE not supported yet".to_string()));
                 }
                 // 0OP has no operands.
            }
            0x0C | 0x0D | 0x0E | 0x0F => { // VAR (Variable operands)
                 let types_byte = self.state.next_u8();
                 for i in 0..4 {
                     let shift = 6 - (i * 2);
                     let type_bits = (types_byte >> shift) & 0x03;
                     
                     match type_bits {
                         0 => { // lconst
                              operands.push(self.state.next_u16());
                         }
                         1 => { // sconst
                              operands.push(self.state.next_u8() as u16);
                         }
                         2 => { // var
                              let var_id = self.state.next_u8();
                              operands.push(self.state.read_variable(var_id)?);
                         }
                         3 => { // none
                              break;
                         }
                         _ => unreachable!(),
                     }
                 }
            }
            _ => unreachable!(),
        }

        let instruction = &self.instructions[op as usize];
        instruction.execute(&mut self.state, operands)
    }
}