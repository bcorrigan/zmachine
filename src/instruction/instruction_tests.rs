use super::*;
use crate::zscreen::ZScreen;

use std::sync::{Arc, Mutex};

// Mock Screen
struct MockScreen {
    output: Arc<Mutex<String>>,
}

impl MockScreen {
    fn new(output: Arc<Mutex<String>>) -> Self {
        MockScreen {
            output,
        }
    }
}

impl ZScreen for MockScreen {
    fn newline(&self) {
        self.output.lock().unwrap().push('\n');
    }
    fn print(&self, str: String) {
        self.output.lock().unwrap().push_str(&str);
    }
    fn read(&self) -> char { ' ' }
    fn readline(&self) -> String { String::new() }
    fn exit(&self) {}
    fn random(&self, _limit: u16) -> u16 { 0 }
    fn set_status(&self, _status: String) {}
    fn get_width(&self) {}
    fn get_height(&self) {}
    fn restart(&self) {}
    fn save(&self, _state: Vec<u8>) {}
    fn restore(&self) -> Vec<u8> { vec![] }
    fn set_window(&self, _num: u16) {}
    fn split_window(&self, _height: u16) {}
    fn erase_window(&self, _num: u16) {}
    fn move_cursor(&self, _x: u8, _y: u8) {}
    fn print_number(&self, _num: u16) {
        self.output.lock().unwrap().push_str(&_num.to_string());
    }
    fn print_char(&self, _char: char) {
        self.output.lock().unwrap().push(_char);
    }
}

fn create_test_state_with_output() -> (State<'static>, Arc<Mutex<String>>) {
    let mut data = vec![0u8; 4096];
    
    // Setup Header
    // 0x06: Initial PC = 0x100 (256)
    data[0x06] = 0x01;
    data[0x07] = 0x00;
    
    // 0x0C: Global Vars = 0x200 (512)
    data[0x0C] = 0x02;
    data[0x0D] = 0x00;

    let output = Arc::new(Mutex::new(String::new()));
    (State::new(&data, Box::new(MockScreen::new(output.clone()))), output)
}

fn create_test_state() -> State<'static> {
    create_test_state_with_output().0
}

#[test]
fn test_print() {
    let (mut state, output) = create_test_state_with_output();
    
    // "hello" in ZChars
    state.mem.write_u8(0x100, 0x35);
    state.mem.write_u8(0x101, 0x51);
    state.mem.write_u8(0x102, 0xC6);
    state.mem.write_u8(0x103, 0x85);
    
    let instr = Print;
    instr.execute(&mut state, vec![]).unwrap();
    
    assert_eq!(state.frame.pc, 0x104);
    assert_eq!(*output.lock().unwrap(), "hello");
}

#[test]
fn test_split_window() {
    let (mut state, output) = create_test_state_with_output();
    let instr = SplitWindow;
    instr.execute(&mut state, vec![10]).unwrap();
    // Verification would require checking side effects on MockScreen.
    // For now, just ensure it runs without error.
    // We can add logging to MockScreen if we really want to verify.
}

#[test]
fn test_verify() {
    let (mut state, _) = create_test_state_with_output();
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0xCA; // Branch true +10

    let instr = Verify;
    instr.execute(&mut state, vec![]).unwrap();
    // Verify always branches true currently
    assert_eq!(state.frame.pc as usize, pc + 1 + 10 - 2);
}

#[test]
fn test_random() {
    let (mut state, _) = create_test_state_with_output();
    // RANDOM 10 -> Result
    
    // PC for store var
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0x10; 

    let instr = Random;
    instr.execute(&mut state, vec![10]).unwrap();
    
    // Result should be 0 (from MockScreen default)
    let result = state.mem.read_u16(0x220);
    assert_eq!(result, 0);
}

#[test]
fn test_print_num() {
    let (mut state, output) = create_test_state_with_output();
    let instr = PrintNum;
    instr.execute(&mut state, vec![1234]).unwrap();
    assert_eq!(*output.lock().unwrap(), "1234");
}

#[test]
fn test_new_line() {
    let (mut state, output) = create_test_state_with_output();
    let instr = NewLine;
    instr.execute(&mut state, vec![]).unwrap();
    assert_eq!(*output.lock().unwrap(), "\n");
}

#[test]
fn test_add() {
    let mut state = create_test_state();
    
    // ADD a, b -> (store)
    // Store result in global variable 0x10 (at 0x200 + 0x10*2 = 0x220)
    // Instruction logic reads store_var from PC
    
    // Setup operands
    let op1 = 10;
    let op2 = 20;
    
    // Setup Memory at PC for 'store_var'
    // Let's say we are executing ADD at PC. 
    // The execute method will read the *next* byte from PC for the store destination.
    // State::next_u8 increments PC.
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0x10; // Store to global 0x10

    let instr = Add;
    instr.execute(&mut state, vec![op1, op2]).unwrap();

    // Check result in global 0x10
    // Address = Global Table (0x200) + 0x10 * 2 = 0x220
    let result = state.mem.read_u16(0x220);
    assert_eq!(result, 30);
    
    // Verify PC advanced by 1 (for the store variable byte)
    assert_eq!(state.frame.pc as usize, pc + 1);
}

#[test]
fn test_add_overflow() {
    let mut state = create_test_state();
    // 32767 + 1 = -32768 (as i16)
    let op1 = 32767;
    let op2 = 1;
    
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0x10; 

    let instr = Add;
    instr.execute(&mut state, vec![op1, op2]).unwrap();

    let result = state.mem.read_u16(0x220);
    // Result is stored as u16 bit pattern of i16 result
    assert_eq!(result as i16, -32768);
}

#[test]
fn test_sub() {
    let mut state = create_test_state();
    let op1 = 20;
    let op2 = 5;
    
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0x10; 

    let instr = Sub;
    instr.execute(&mut state, vec![op1, op2]).unwrap();

    let result = state.mem.read_u16(0x220);
    assert_eq!(result, 15);
}

#[test]
fn test_mul() {
    let mut state = create_test_state();
    let op1 = 10;
    let op2 = -2i16 as u16;
    
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0x10; 

    let instr = Mul;
    instr.execute(&mut state, vec![op1, op2]).unwrap();

    let result = state.mem.read_u16(0x220);
    assert_eq!(result as i16, -20);
}

#[test]
fn test_div() {
    let mut state = create_test_state();
    let op1 = -20i16 as u16;
    let op2 = 5;
    
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0x10; 

    let instr = Div;
    instr.execute(&mut state, vec![op1, op2]).unwrap();

    let result = state.mem.read_u16(0x220);
    assert_eq!(result as i16, -4);
}

#[test]
fn test_mod() {
    let mut state = create_test_state();
    let op1 = 13;
    let op2 = 5;
    
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0x10; 

    let instr = Mod;
    instr.execute(&mut state, vec![op1, op2]).unwrap();

    let result = state.mem.read_u16(0x220);
    assert_eq!(result, 3);
}

#[test]
fn test_or() {
    let mut state = create_test_state();
    let op1 = 0b1010;
    let op2 = 0b0101;
    
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0x10; 

    let instr = Or;
    instr.execute(&mut state, vec![op1, op2]).unwrap();

    let result = state.mem.read_u16(0x220);
    assert_eq!(result, 0b1111);
}

#[test]
fn test_and() {
    let mut state = create_test_state();
    let op1 = 0b1100;
    let op2 = 0b0101;
    
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0x10; 

    let instr = And;
    instr.execute(&mut state, vec![op1, op2]).unwrap();

    let result = state.mem.read_u16(0x220);
    assert_eq!(result, 0b0100);
}

#[test]
fn test_not() {
    let mut state = create_test_state();
    let op1 = 0x00FF;
    
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0x10; 

    let instr = Not;
    instr.execute(&mut state, vec![op1]).unwrap();

    let result = state.mem.read_u16(0x220);
    assert_eq!(result, 0xFF00);
}

#[test]
fn test_div_zero() {
    let mut state = create_test_state();
    let instr = Div;
    let result = instr.execute(&mut state, vec![10, 0]);
    assert!(result.is_err());
}

#[test]
fn test_je_match() {
    let mut state = create_test_state();
    
    // JE 5, 10, 5 -> Branch
    // Should take branch
    
    // Setup branch bytes at PC
    // Branch to offset +10
    // Byte 1: 1 (bit 7 set = branch on true, bit 6 set = short branch) | offset (bit 0-5)
    // Offset 10 = 0x0A. 
    // Byte 1 = 0x80 | 0x40 | 0x0A = 0xCA
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0xCA; 
    
    let instr = Je;
    instr.execute(&mut state, vec![5, 10, 5]).unwrap();
    
    // PC should be: Old PC + 1 (read branch byte) + Offset (10) - 2
    // Wait, logic says: self.frame.pc = (self.frame.pc as i32 + offset as i32 - 2) as u32;
    // self.frame.pc was incremented by next_u8() *before* this calculation.
    // Initial PC (0x100) -> next_u8 (read 0xDA) -> PC is 0x101.
    // New PC = 0x101 + 10 - 2 = 0x101 + 8 = 0x109.
    assert_eq!(state.frame.pc, 0x109);
}

#[test]
fn test_jz() {
    let mut state = create_test_state();
    
    // JZ 0 -> Branch (True)
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0xCA; // Branch on true, offset 10

    let instr = Jz;
    instr.execute(&mut state, vec![0]).unwrap();
    assert_eq!(state.frame.pc as usize, pc + 1 + 10 - 2);

    // JZ 1 -> No Branch
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0xCA; 

    let instr = Jz;
    instr.execute(&mut state, vec![1]).unwrap();
    assert_eq!(state.frame.pc as usize, pc + 1);
}

#[test]
fn test_jl() {
    let mut state = create_test_state();
    
    // JL 5, 10 -> Branch (True)
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0xCA; 

    let instr = Jl;
    instr.execute(&mut state, vec![5, 10]).unwrap();
    assert_eq!(state.frame.pc as usize, pc + 1 + 10 - 2);

    // JL 10, 5 -> No Branch
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0xCA; 

    let instr = Jl;
    instr.execute(&mut state, vec![10, 5]).unwrap();
    assert_eq!(state.frame.pc as usize, pc + 1);
}

#[test]
fn test_jg() {
    let mut state = create_test_state();
    
    // JG 10, 5 -> Branch (True)
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0xCA; 

    let instr = Jg;
    instr.execute(&mut state, vec![10, 5]).unwrap();
    assert_eq!(state.frame.pc as usize, pc + 1 + 10 - 2);
    
    // JG 5, 10 -> No Branch
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0xCA; 

    let instr = Jg;
    instr.execute(&mut state, vec![5, 10]).unwrap();
    assert_eq!(state.frame.pc as usize, pc + 1);
}

#[test]
fn test_jump() {
    let mut state = create_test_state();
    let pc = state.frame.pc as usize;
    
    // JUMP 10 (Instruction operand is 10)
    // JUMP is decoded. Operands contains [10].
    // state.frame.pc is at operand end (conceptually, though execute_instruction handles PC advancement during decode)
    // Wait, execute_instruction advances PC as it reads operands.
    // So for JUMP (1OP), pc is after the 2-byte operand (if 1OP).
    // Or if variable form... 
    // Let's assume decoding works.
    // The instruction impl says: frame.pc = frame.pc - 2 + offset.
    
    let instr = Jump;
    instr.execute(&mut state, vec![10]).unwrap();
    
    // Expected: PC - 2 + 10.
    // But wait, the "PC" in the test context (state.frame.pc) is whatever it was *after* fetch/decode.
    // In this test, we didn't actually run decode. We manually set PC.
    // So if current PC is X. New PC is X - 2 + 10 = X + 8.
    
    assert_eq!(state.frame.pc as usize, pc + 8);
}

#[test]
fn test_test() {
    let mut state = create_test_state();
    
    // TEST 0b1111, 0b0101 -> Branch (True, because 0b1111 & 0b0101 == 0b0101)
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0xCA; 

    let instr = Test;
    instr.execute(&mut state, vec![0b1111, 0b0101]).unwrap();
    assert_eq!(state.frame.pc as usize, pc + 1 + 10 - 2);
    
    // TEST 0b0000, 0b0101 -> No Branch
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0xCA; 

    let instr = Test;
    instr.execute(&mut state, vec![0b0000, 0b0101]).unwrap();
    assert_eq!(state.frame.pc as usize, pc + 1);
}

#[test]
fn test_get_parent() {
    
    
    let mut state = create_test_state();
    
    // Setup Mock Object Table (V3 by default if not set otherwise, State::new defaults to whatever reads from mem)
    // Version = 1 (State::new reads byte 0)
    state.version = 1;
    
    // Object Table @ 0x0A (from create_test_state defaults)
    // Default Header Size = 31 * 2 = 62 bytes.
    // Objects start at 0x0A + 62 = 0x48 (72).
    // Object Size V3 = 9 bytes.
    // Parent Offset = 4.
    
    let _object_table = state.mem.object_table(); // 0x200 from create_test_state? No wait.
    // create_test_state: 
    // data[0x06] = 0x01; // PC
    // data[0x0C] = 0x02; // Global
    // It doesn't set 0x0A (Object Table). 
    // Let's set it.
    state.mem.write_u16(0x0A, 0x300); // Object table at 0x300.
    
    let objects_start = 0x300 + 62;
    
    // Object 1: at objects_start.
    // Set parent (byte 4) to 2.
    state.mem.write_u8(objects_start + 4, 2);
    
    // Object 2: at objects_start + 9.
    // Set parent (byte 4) to 0.
    state.mem.write_u8(objects_start + 9 + 4, 0);

    // JIN 1, 2 -> Branch (True)
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0xCA; 

    let instr = Jin;
    instr.execute(&mut state, vec![1, 2]).unwrap();
    assert_eq!(state.frame.pc as usize, pc + 1 + 10 - 2);
    
    // JIN 2, 1 -> No Branch
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0xCA; 

    let instr = Jin;
    instr.execute(&mut state, vec![2, 1]).unwrap();
    assert_eq!(state.frame.pc as usize, pc + 1);
}

#[test]
fn test_call_ret() {
    let mut state = create_test_state();
    
    // Setup a dummy routine
    // Address 0x400
    // Header: Locals count (1) | Local 1 Default (0x1234)
    state.mem.write_u8(0x400, 1);
    state.mem.write_u16(0x401, 0x1234);
    
    // PC for call instruction result storage: Store to global 0x10
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0x10; 
    
    // Call routine 0x400 / 2 (packed) = 0x200 (if packed)
    // But CALL takes packed address.
    // If V3, addr = packed * 2. So if we want 0x400, packed is 0x200.
    
    let instr_call = Call;
    instr_call.execute(&mut state, vec![0x200]).unwrap();
    
    // Check we are in new frame
    // PC should be after header: 0x400 + 1 (count) + 2 (local) = 0x403
    assert_eq!(state.frame.pc, 0x403);
    
    // Check local 1 initialized
    let local1 = state.frame.read_local(&state.mem.stack, 1);
    assert_eq!(local1, 0x1234);
    
    // RET 0x5678
    let instr_ret = Ret;
    instr_ret.execute(&mut state, vec![0x5678]).unwrap();
    
    // Check we returned to previous frame
    assert_eq!(state.frame.pc as usize, pc + 1);
    
    // Check result stored in global 0x10
    let result = state.mem.read_u16(0x220);
    assert_eq!(result, 0x5678);
}

#[test]
fn test_quit() {
    let mut state = create_test_state();
    assert!(state.running);
    let instr = Quit;
    instr.execute(&mut state, vec![]).unwrap();
    assert!(!state.running);
}

#[test]
fn test_store() {
    let mut state = create_test_state();
    let instr = Store;
    instr.execute(&mut state, vec![0x10, 42]).unwrap();
    let result = state.mem.read_u16(0x220); // Global 0x10
    assert_eq!(result, 42);
}

#[test]
fn test_load() {
    let mut state = create_test_state();
    state.mem.write_u16(0x220, 42); // Global 0x10
    
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0x11; // Result to global 0x11

    let instr = Load;
    instr.execute(&mut state, vec![0x10]).unwrap();
    
    let result = state.mem.read_u16(0x222); // Global 0x11
    assert_eq!(result, 42);
}

#[test]
fn test_storew() {
    let mut state = create_test_state();
    let instr = StoreW;
    instr.execute(&mut state, vec![0x300, 2, 1234]).unwrap();
    let result = state.mem.read_u16(0x300 + 4);
    assert_eq!(result, 1234);
}

#[test]
fn test_storeb() {
    let mut state = create_test_state();
    let instr = StoreB;
    instr.execute(&mut state, vec![0x300, 2, 123]).unwrap();
    let result = state.mem.read_u8(0x302);
    assert_eq!(result, 123);
}

#[test]
fn test_loadw() {
    let mut state = create_test_state();
    state.mem.write_u16(0x304, 1234);
    
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0x10; // Result to global 0x10

    let instr = LoadW;
    instr.execute(&mut state, vec![0x300, 2]).unwrap();
    
    let result = state.mem.read_u16(0x220);
    assert_eq!(result, 1234);
}

#[test]
fn test_loadb() {
    let mut state = create_test_state();
    state.mem.write_u8(0x302, 123);
    
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0x10; // Result to global 0x10

    let instr = LoadB;
    instr.execute(&mut state, vec![0x300, 2]).unwrap();
    
    let result = state.mem.read_u16(0x220);
    assert_eq!(result, 123);
}

#[test]
fn test_push_pull() {
    let mut state = create_test_state();
    
    let instr_push = Push;
    instr_push.execute(&mut state, vec![42]).unwrap();
    
    let instr_pull = Pull;
    instr_pull.execute(&mut state, vec![0x10]).unwrap();
    
    let result = state.mem.read_u16(0x220);
    assert_eq!(result, 42);
}

#[test]
fn test_inc_dec() {
    let mut state = create_test_state();
    state.mem.write_u16(0x220, 42); // Global 0x10
    
    let instr_inc = Inc;
    instr_inc.execute(&mut state, vec![0x10]).unwrap();
    assert_eq!(state.mem.read_u16(0x220), 43);
    
    let instr_dec = Dec;
    instr_dec.execute(&mut state, vec![0x10]).unwrap();
    assert_eq!(state.mem.read_u16(0x220), 42);
}

#[test]
fn test_inc_chk() {
    let mut state = create_test_state();
    state.mem.write_u16(0x220, 42); // Global 0x10
    
    // INC_CHK 0x10, 42 -> 43 > 42 -> Branch (True)
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0xCA; 

    let instr = IncChk;
    instr.execute(&mut state, vec![0x10, 42]).unwrap();
    assert_eq!(state.frame.pc as usize, pc + 1 + 10 - 2);
}

#[test]
fn test_dec_chk() {
    let mut state = create_test_state();
    state.mem.write_u16(0x220, 42); // Global 0x10
    
    // DEC_CHK 0x10, 42 -> 41 < 42 -> Branch (True)
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0xCA; 

    let instr = DecChk;
    instr.execute(&mut state, vec![0x10, 42]).unwrap();
    assert_eq!(state.frame.pc as usize, pc + 1 + 10 - 2);
}

#[test]
fn test_set_attr() {
    let mut state = create_test_state();
    state.version = 1;
    // Set up object 1 at 0x300 + 62
    state.mem.write_u16(0x0A, 0x300);
    
    // Attributes are first 4 bytes (32 bits)
    // 0x300 + 62 + 0 = Attr byte 0 (bits 31-24: attrs 0-7)
    // 0x300 + 62 + 1 = Attr byte 1 (bits 23-16: attrs 8-15)
    // Attr 0 is bit 7 of byte 0.
    
    let instr = SetAttr;
    instr.execute(&mut state, vec![1, 0]).unwrap();
    
    let attr_byte = state.mem.read_u8(0x300 + 62);
    assert_eq!(attr_byte, 0x80); // Bit 7 set
}

#[test]
fn test_clear_attr() {
    let mut state = create_test_state();
    state.version = 1;
    state.mem.write_u16(0x0A, 0x300);
    // Set attr 0 initially
    state.mem.write_u8(0x300 + 62, 0x80);
    
    let instr = ClearAttr;
    instr.execute(&mut state, vec![1, 0]).unwrap();
    
    let attr_byte = state.mem.read_u8(0x300 + 62);
    assert_eq!(attr_byte, 0x00);
}

#[test]
fn test_test_attr() {
    let mut state = create_test_state();
    state.version = 1;
    state.mem.write_u16(0x0A, 0x300);
    state.mem.write_u8(0x300 + 62, 0x80); // Attr 0 set
    
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0xCA; // Branch true +10

    let instr = TestAttr;
    instr.execute(&mut state, vec![1, 0]).unwrap(); // Test attr 0
    assert_eq!(state.frame.pc as usize, pc + 1 + 10 - 2); // Branch taken
    
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0xCA; 
    
    instr.execute(&mut state, vec![1, 1]).unwrap(); // Test attr 1 (not set)
    assert_eq!(state.frame.pc as usize, pc + 1); // No branch
}

#[test]
fn test_insert_obj() {
    let mut state = create_test_state();
    state.version = 1;
    state.mem.write_u16(0x0A, 0x300);
    // Obj 1 (0x300+62), Obj 2 (0x300+62+9)
    // Insert Obj 2 into Obj 1
    
    let instr = InsertObj;
    instr.execute(&mut state, vec![2, 1]).unwrap();
    
    // Obj 2 Parent (offset 4) should be 1
    assert_eq!(state.mem.read_u8(0x300 + 62 + 9 + 4), 1);
    // Obj 1 Child (offset 6) should be 2
    assert_eq!(state.mem.read_u8(0x300 + 62 + 6), 2);
}

#[test]
fn test_remove_obj() {
    let mut state = create_test_state();
    state.version = 1;
    state.mem.write_u16(0x0A, 0x300);
    // Setup Obj 2 as child of Obj 1
    state.mem.write_u8(0x300 + 62 + 9 + 4, 1); // Obj 2 parent = 1
    state.mem.write_u8(0x300 + 62 + 6, 2);     // Obj 1 child = 2
    
    let instr = RemoveObj;
    instr.execute(&mut state, vec![2]).unwrap();
    
    // Obj 2 Parent should be 0
    assert_eq!(state.mem.read_u8(0x300 + 62 + 9 + 4), 0);
    // Obj 1 Child should be 0 (since it was the only child)
    assert_eq!(state.mem.read_u8(0x300 + 62 + 6), 0);
}

#[test]
fn test_get_child() {
    let mut state = create_test_state();
    state.version = 1;
    state.mem.write_u16(0x0A, 0x300);
    state.mem.write_u8(0x300 + 62 + 6, 2); // Obj 1 child = 2
    
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0x10; // Store to global 0x10
    state.mem[pc as u16 + 1] = 0xCA; // Branch true +10

    let instr = GetChild;
    instr.execute(&mut state, vec![1]).unwrap();
    
    assert_eq!(state.mem.read_u16(0x220), 2);
    assert_eq!(state.frame.pc as usize, pc + 2 + 10 - 2);
}

#[test]
fn test_get_sibling() {
    let mut state = create_test_state();
    state.version = 1;
    state.mem.write_u16(0x0A, 0x300);
    state.mem.write_u8(0x300 + 62 + 5, 2); // Obj 1 sibling = 2
    
    let pc = state.frame.pc as usize;
    state.mem[pc as u16] = 0x10; // Store to global 0x10
    state.mem[pc as u16 + 1] = 0xCA; // Branch true +10

    let instr = GetSibling;
    instr.execute(&mut state, vec![1]).unwrap();
    
    assert_eq!(state.mem.read_u16(0x220), 2);
    assert_eq!(state.frame.pc as usize, pc + 2 + 10 - 2);
}
