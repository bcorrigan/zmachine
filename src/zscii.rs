use std::string::FromUtf16Error;

use crate::memory::Memory;

// See 3.5.3 @ https://www.inform-fiction.org/zmachine/standards/z1point1/sect03.html
const ZSCII_MAP234: [[char; 32]; 3] = [
    [
        ' ', '\0', '\0', '\0', '\0', '\0', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k',
        'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ],
    [
        '\0', '\0', '\0', '\0', '\0', '\0', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K',
        'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ],
    [
        '\0', '\0', '\0', '\0', '\0', '\0', ' ', '\n', '0', '1', '2', '3', '4', '5', '6', '7', '8',
        '9', '.', ',', '!', '?', '_', '#', '\'', '"', '/', '\\', '-', ':', '(', ')',
    ],
];

const ZSCII_MAP1: [[char; 32]; 3] = [
    [
        ' ', '\0', '\0', '\0', '\0', '\0', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k',
        'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ],
    [
        '\0', '\0', '\0', '\0', '\0', '\0', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K',
        'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ],
    [
        '\0', '\0', '\0', '\0', '\0', '\0', ' ', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        '.', ',', '!', '?', '_', '#', '\'', '"', '/', '\\', '<', '-', ':', '(', ')',
    ],
];

pub struct Zscii<'a> {
    ptr: u16,
    mode: u8,
    buf: Vec<char>,
    mem: &'a Memory,
}

impl<'a> Zscii<'a> {
    pub fn new(mem: &Memory) -> Zscii {
        Zscii {
            ptr: 0u16,
            mode: 0u8,
            buf: vec![],
            mem: mem,
        }
    }

    pub fn get_string(&mut self, ptr: u16) -> String {
        self.ptr = ptr;
        self.mode = 0u8;
        self.buf.clear();

        "".to_owned()
    }
}
