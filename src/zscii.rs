use crate::memory::Memory;

// See 3.5.3 @ https://www.inform-fiction.org/zmachine/standards/z1point1/sect03.html
const ZSCII_MAP234: [[char; 32]; 3] = [
    [
        //A0
        ' ', '\0', '\0', '\0', '\0', '\0', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k',
        'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ],
    [
        //A1
        '\0', '\0', '\0', '\0', '\0', '\0', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K',
        'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ],
    [
        //A2
        '\0', '\0', '\0', '\0', '\0', '\0', ' ', '\n', '0', '1', '2', '3', '4', '5', '6', '7', '8',
        '9', '.', ',', '!', '?', '_', '#', '\'', '"', '/', '\\', '-', ':', '(', ')',
    ],
];

const ZSCII_MAP1: [[char; 32]; 3] = [
    [
        //A0
        ' ', '\0', '\0', '\0', '\0', '\0', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k',
        'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ],
    [
        //A1
        '\0', '\0', '\0', '\0', '\0', '\0', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K',
        'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ],
    [
        //A2
        '\0', '\0', '\0', '\0', '\0', '\0', ' ', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        '.', ',', '!', '?', '_', '#', '\'', '"', '/', '\\', '<', '-', ':', '(', ')',
    ],
];

enum Mode {
    A0,
    A1,
    A2,
    ABBREV,
    ZCODE1,
    ZCODE2,
}

pub struct Zscii<'a> {
    ptr: u16,
    mode: Mode,
    buf: Vec<char>,
    mem: &'a Memory,
}

impl<'a> Zscii<'a> {
    pub fn new(mem: &Memory) -> Zscii {
        Zscii {
            ptr: 0u16,
            mode: Mode::A0,
            buf: vec![],
            mem: mem,
        }
    }

    pub fn get_string(&mut self, ptr: u16) -> String {
        self.ptr = ptr;
        self.mode = Mode::A0;
        self.buf.clear();
        //ZSTRINGs are read in 2 byte pairs. 3 chars are packed into each byte pair, the first bit is set only on the last char
        loop {
            //ZCHARS are 5 bit words. They are packed into consecutive byte pairs as so:
            // XAAAAABB BBBCCCCC
            // the 'X' bit is discarded, we do some shifts/bytewise stuff to extract the rest
            // dummy's guide:
            // 0x1f = 00011111 = extract last 5 bits. Of course, 3 = 0x11
            let byte1 = self.mem.read_u8(ptr);
            self.ptr += 1;
            let byte2 = self.mem.read_u8(ptr);
            self.ptr += 1;

            self.process_zchar((byte1 >> 2) & 0x1f); //AAAAA
            self.process_zchar((byte1 & 3u8) << 3 | (byte2 >> 5)); //BBBBB
            self.process_zchar(byte2 & 0x1f); //CCCCC

            //check the X bit
            if (byte1 & 0x80) == 0 {
                break;
            }
        }
        "".to_owned()
    }

    fn process_zchar(&mut self, ch: u8) {
        match (self.mode) {
            Mode::A0 => {}
            Mode::A1 => {}
            Mode::A2 => {}
            Mode::ABBREV => {}
            Mode::ZCODE1 => {}
            Mode::ZCODE2 => {}
        }
    }
}
