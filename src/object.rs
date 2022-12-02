use crate::memory::Memory;

/*
 * Here we implement all the object-level reading/writing 
 */


struct Object<'a> {
    mem: &'a mut Memory 
}

impl<'a>  Object<'a> {
    pub const PARENT:u16 = 4;
    const SIBLING:u16 = 5;
    const CHILD:u16 = 6;
    const PROPS:u16 = 7;
    pub const SIZE:u16 = 9;

    const PROPMAX:u16 = 31;

    fn object_table_ptr(&self) -> u16 {
        self.mem.object_table() + Object::PROPMAX * 2 - Object::SIZE 
    }

    fn object_ptr(&self, obj: u8) -> u16 {
        self.object_table_ptr() + (obj as u16 * Object::SIZE)
    }

    fn get_attr_bytes(&self, obj: u8) -> u32 {
        self.mem.read_u32(self.object_table_ptr() + (obj as u16 * Object::SIZE))
    }

    fn write_attr_bytes(&mut self, obj: u8, attrs: u32) {
        self.mem.write_u32(self.object_table_ptr() + (obj as u16 * Object::SIZE), attrs);
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
        self.mem.read_u8(self.object_ptr(obj_a) + Object::PARENT) == obj_b
    }

    fn sibling(&self, obj: u8) -> u8 {
        self.mem.read_u8(self.object_ptr(obj) + Object::SIBLING)    
    }

    fn parent(&self, obj: u8) -> u8 {
        self.mem.read_u8(self.object_ptr(obj) + Object::PARENT)    
    }

    fn child(&self, obj: u8) -> u8 {
        self.mem.read_u8(self.object_ptr(obj) + Object::CHILD)    
    }

    fn remove(&mut self, obj: u8) {
        let ptr: u16 = self.object_ptr(obj);
        let parent = self.parent(obj);
        
        if parent==0 { //no parent
            return;
        }

        let obj_sibling = self.sibling(obj);
        let mut child = self.child(obj);

        if child == obj {
            //immediate child
            self.mem.write_u8(self.object_ptr(parent) + Object::CHILD, obj_sibling);
        } else {
            while child!=0 {
                let child_addr = self.object_ptr(child);

                let sibling = self.mem.read_u8(child_addr + Object::SIBLING);

                if sibling == obj {
                    self.mem.write_u8(child_addr + Object::SIBLING, obj_sibling);
                    break;
                } else {
                    child = sibling;
                }
            }
        }

        self.mem.write_u8(ptr + Object::SIBLING, 0);
        self.mem.write_u8(ptr + Object::PARENT, 0);

    }

    

}