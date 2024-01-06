use crate::memory::Memory;
use crate::zscreen::ZScreen;

struct ZMachine<'a> {
    mem: Memory,
    zscreen: Box<dyn ZScreen + 'a>,
}
