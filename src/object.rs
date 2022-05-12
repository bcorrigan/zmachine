use crate::memory::Memory;

/*
 * Here we implement all the object-level reading/writing 
 */


struct Object<'a> {
    mem: &'a Memory 
}

impl<'a>  Object<'a> {
    pub const PARENT:u8 = 4;
    const SIBLING:u8 = 5;
    const CHILD:u8 = 6;
    const PROPS:u8 = 7;
    pub const SIZE:u8 = 9;

    const PROPMAX:u8 = 31;

    fn object_ptr(&self) -> u16 {
        self.mem.object_table() + Object::PROPMAX as u16 * 2 - Object::SIZE as u16
    }

    //There are 32 attrs bits across 4 bytes
    //we need to find which byte has the attr, and then test the appropriate bit in that byte
    fn attr_test(&self, obj: u8, attr: u8) -> bool {
        let attr_bytes = self.mem.read_u32(self.object_ptr() + (obj * Object::SIZE) as u16);
        attr_bytes & (1 << (31 - attr)) != 0
    }
}