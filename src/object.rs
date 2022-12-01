use crate::memory::Memory;

/*
 * Here we implement all the object-level reading/writing 
 */


struct Object<'a> {
    mem: &'a mut Memory 
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

    fn get_attr_bytes(&self, obj: u8) -> u32 {
        self.mem.read_u32(self.object_ptr() + (obj * Object::SIZE) as u16)
    }

    fn write_attr_bytes(&mut self, obj: u8, attrs: u32) {
        self.mem.write_u32(self.object_ptr() + (obj * Object::SIZE) as u16, attrs);
    }

    //There are 32 attrs bits across 4 bytes
    //we need to find which byte has the attr, and then test the appropriate bit in that byte
    fn attr_test(&mut self, obj: u8, attr: u8) -> bool {
        self.get_attr_bytes(obj) & (1 << (31 - attr)) != 0
    }

    fn attr_set(&mut self, obj: u8, attr: u8) {
        self.write_attr_bytes( obj, self.get_attr_bytes(obj) | 1 << (31 - attr));
    }

    fn attr_clear(&mut self, obj: u8, attr: u8) {
        self.write_attr_bytes( obj, self.get_attr_bytes(obj) & !(1 << (31 - attr)));
    }

    fn inside(&self, obj_a: u8, obj_b: u8) -> bool {
        self.mem.read_u8(self.object_ptr() + (obj_a * Object::SIZE) as u16 + Object::PARENT as u16) == obj_b
    }

    fn sibling(&self, obj: u8) -> u8 {
        self.mem.read_u8(self.object_ptr() + (obj * Object::SIZE) as u16 + Object::SIBLING as u16)    
    }

    fn parent(&self, obj: u8) -> u8 {
        self.mem.read_u8(self.object_ptr() + (obj * Object::SIZE) as u16 + Object::PARENT as u16)    
    }

    fn child(&self, obj: u8) -> u8 {
        self.mem.read_u8(self.object_ptr() + (obj * Object::SIZE) as u16 + Object::CHILD as u16)    
    }
    

    

}