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
    ABBREV(u8),
    ZCODE1,
    ZCODE2(u8),
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
        self.buf.iter().collect()
    }

    fn process_zchar(&mut self, ch: u8) {
        match self.mode {
            Mode::A0 => match ch {
                1 => self.mode = Mode::ABBREV(0),
                2 => self.mode = Mode::ABBREV(1),
                3 => self.mode = Mode::ABBREV(2),
                4 => self.mode = Mode::A1,
                5 => self.mode = Mode::A2,
                _ => self.buf.push(ZSCII_MAP234[0][ch as usize]),
            },
            Mode::A1 => match ch {
                1 => self.mode = Mode::ABBREV(0),
                2 => self.mode = Mode::ABBREV(1),
                3 => self.mode = Mode::ABBREV(2),
                4 => self.mode = Mode::A1,
                5 => self.mode = Mode::A2,
                _ => {
                    self.buf.push(ZSCII_MAP234[1][ch as usize]);
                    self.mode = Mode::A0;
                }
            },
            Mode::A2 => match ch {
                1 => self.mode = Mode::ABBREV(0),
                2 => self.mode = Mode::ABBREV(1),
                3 => self.mode = Mode::ABBREV(2),
                4 => self.mode = Mode::A1,
                5 => self.mode = Mode::A2,
                6 => self.mode = Mode::ZCODE1,
                _ => {
                    self.buf.push(ZSCII_MAP234[2][ch as usize]);
                    self.mode = Mode::A0;
                }
            },
            Mode::ABBREV(table) => {
                let abbrev = table * 32 + ch;
                let mut zscii = Zscii::new(self.mem);
                let str = zscii.get_string(
                    self.mem
                        .read_u16(self.mem.abbreviations_table() + (2 * abbrev as u16)),
                );
                let mut abbrev_vec: Vec<char> = str.chars().collect::<Vec<_>>();
                self.buf.append(&mut abbrev_vec);
                self.mode = Mode::A0;
            }
            Mode::ZCODE1 => self.mode = Mode::ZCODE2(ch),
            Mode::ZCODE2(code1) => {
                let code: [u16; 1] = [ch as u16 | (code1 as u16) << 5];
                let mut char: Vec<char> =
                    char::decode_utf16(code).map(|r| r.unwrap_or(' ')).collect();
                self.buf.append(&mut char);
                self.mode = Mode::A0;
            }
        }
    }
}
