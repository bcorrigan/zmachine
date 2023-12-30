use crate::{memory::Memory, zscii};
//use core::num::traits::Num;
//use core::num::Num;
use num::Integer;
use std::marker::PhantomData;

/*
 * Here we implement all the object-level reading/writing
 * Early z machines used 1 byte objects (ie there oculd be a total of 256 objects),
 * and later used 2 byte objects - so this code is generic across u8 & u16
 */
struct Object<'a, T: 'a>
where
    T: Integer + Into<u8> + Into<u16> + Copy,
{
    mem: &'a mut Memory,
    phantom: PhantomData<&'a T>,
}

trait ReadObject {
    fn read_obj(&self, addr: u16, mem: &Memory) -> Self
    where
        Self: Sized;
}

impl ReadObject for u8 {
    fn read_obj(&self, addr: u16, mem: &Memory) -> u8 {
        mem.read_u8(addr)
    }
}

impl ReadObject for u16 {
    fn read_obj(&self, addr: u16, mem: &Memory) -> u16 {
        mem.read_u16(addr)
    }
}

struct PropAddr {
    addr: u16,
    size_bytes: u8,
    data_length: u8,
}

impl<'a, T> Object<'_, T>
where
    T: Integer + Into<u8> + Into<u16> + Copy + ReadObject + From<u16>,
{
    const WIDE: bool = std::mem::size_of::<T>() == 2;

    const C: () = assert!(
        std::mem::size_of::<T>() < 3,
        "ZObjects can only be width u8 or u16"
    );
    const PARENT: u16 = if Object::<T>::WIDE { 6 } else { 4 };
    const SIBLING: u16 = if Object::<T>::WIDE { 8 } else { 5 };
    const CHILD: u16 = if Object::<T>::WIDE { 10 } else { 6 };
    const PROPS: u16 = if Object::<T>::WIDE { 12 } else { 7 };
    const SIZE: u16 = if Object::<T>::WIDE { 14 } else { 9 };

    const PROPMAX: u16 = if Object::<T>::WIDE { 63 } else { 31 };

    fn object_table_ptr(&self) -> u16 {
        self.mem.object_table() + Object::<T>::PROPMAX * 2 - Object::<T>::SIZE
    }

    fn object_ptr(&self, obj: T) -> u16 {
        self.object_table_ptr() + (Into::<u16>::into(obj) * Object::<T>::SIZE)
    }

    fn get_attr_bytes(&self, obj: u8) -> u32 {
        self.mem
            .read_u32(self.object_table_ptr() + (obj as u16 * Object::<T>::SIZE))
    }

    fn write_attr_bytes(&mut self, obj: u8, attrs: u32) {
        self.mem.write_u32(
            self.object_table_ptr() + (obj as u16 * Object::<T>::SIZE),
            attrs,
        );
    }

    //There are 32 attrs bits across 4 bytes
    //we need to find which byte has the attr, and then test the appropriate bit in that byte
    fn attr_test(&self, obj: u8, attr: u8) -> bool {
        self.get_attr_bytes(obj) & (1 << (31 - attr)) != 0
    }

    fn attr_set(&mut self, obj: T, attr: u8) {
        self.write_attr_bytes(
            obj.into(),
            self.get_attr_bytes(obj.into()) | 1 << (31 - attr),
        );
    }

    fn attr_clear(&mut self, obj: T, attr: u8) {
        self.write_attr_bytes(
            obj.into(),
            self.get_attr_bytes(obj.into()) & !(1 << (31 - attr)),
        );
    }

    fn inside(&self, obj_a: T, obj_b: T) -> bool {
        self.mem
            .read_u8(self.object_ptr(obj_a) + Object::<T>::PARENT)
            == obj_b.into()
    }

    fn sibling(&self, obj: T) -> T {
        obj.read_obj(self.object_ptr(obj) + Object::<T>::SIBLING, self.mem)
    }

    fn parent(&self, obj: T) -> T {
        obj.read_obj(self.object_ptr(obj) + Object::<T>::PARENT, self.mem)
    }

    fn child(&self, obj: T) -> T {
        obj.read_obj(self.object_ptr(obj) + Object::<T>::CHILD, self.mem)
    }

    //address of the props table for given object
    fn props(&self, obj: T) -> u16 {
        self.mem.read_u16(self.object_ptr(obj) + Object::<T>::PROPS)
    }

    fn write_sibling(&mut self, obj: T, sibling: T) {
        self.mem
            .write_u8(self.object_ptr(obj) + Object::<T>::SIBLING, sibling.into())
    }

    fn write_parent(&mut self, obj: T, parent: T) {
        self.mem
            .write_u8(self.object_ptr(obj) + Object::<T>::PARENT, parent.into())
    }

    fn write_child(&mut self, obj: T, child: T) {
        self.mem
            .write_u8(self.object_ptr(obj) + Object::<T>::CHILD, child.into())
    }

    fn remove(&mut self, obj: T) {
        let parent = self.parent(obj.clone());

        if num::Zero::is_zero(&parent) {
            return;
        }

        let obj_sibling = self.sibling(obj.clone());
        let mut child = self.child(obj);

        if child == obj {
            //immediate child
            self.write_child(parent, obj_sibling);
        } else {
            while !num::Zero::is_zero(&child) {
                let sibling = self.child(child);

                if sibling == obj {
                    self.write_sibling(child, obj_sibling);
                    break;
                } else {
                    child = sibling;
                }
            }
        }

        self.write_sibling(obj, num::Zero::zero());
        self.write_parent(obj, num::Zero::zero());
    }

    fn insert(&mut self, obj: T, dest_obj: T) {
        if !num::Zero::is_zero(&self.parent(obj)) {
            self.remove(obj);
        }

        self.write_sibling(obj, self.child(dest_obj));
        self.write_child(dest_obj, obj);
        self.write_parent(obj, dest_obj);
    }

    //print

    fn name(&mut self, obj: T) -> Option<String> {
        if self.mem.read_u8(self.props(obj)) != 0 {
            let mut zscii = zscii::Zscii::new(self.mem);
            Some(zscii.get_string(self.props(obj) + 1))
        } else {
            None
        }
    }

    fn status(&self) -> u16 {
        self.mem
            .read_u16(self.object_ptr(T::from(self.mem.read_global(16))))
    }

    //The property number occupies the bottom 6 bits of the first size byte.
    fn get_prop_len(&self, addr: u16) -> u8 {
        //see sections 12.4.2.1.1 - 12.4.2.2 in standards doc:
        //If the top bit (bit 7) of the first size byte is clear, then there is only one size-and-number byte.
        //Bits 0 to 5 contain the property number; bit 6 is either clear to indicate a property data length of 1,
        //or set to indicate a length of 2; bit 7 is clear.
        if addr == 0 {
            0
        } else if Object::<T>::WIDE {
            let sz = self.mem.read_u8(addr - 1);
            if sz == 0 {
                if sz & 0x40 == 0 {
                    1
                } else {
                    2
                }
            } else {
                if sz & 0x3F == 0 {
                    //if first 6 bits 111111 are all 0
                    //From stds doc: A value of 0 as property data length (in the second byte) should be interpreted as a length of 64
                    64
                } else {
                    sz
                }
            }
        } else {
            (self.mem.read_u8(addr - 1) >> 5) + 1
        }
    }

    //returns address to the property *value* not the size byte
    fn get_prop_addr(&self, obj: T, prop_id: u8) -> PropAddr {
        let top_prop_table_addr = self.props(obj);
        //skip name to first property
        let mut property_addr =
            top_prop_table_addr + self.mem.read_u8(top_prop_table_addr) as u16 * 2 + 1;
        if Object::<T>::WIDE {
            loop {
                let size = self.mem.read_u8(property_addr);
                let id = size & 0x3f; //Bits 0 to 5 contain the property number
                if size == 0 {
                    break;
                }
                if id == prop_id {
                    //bit 7 0x80 indicates if there's one size-and-number byte or two
                    if size & 0x80 == 0 {
                        // bit 6 - 0x40 -  is either clear to indicate a property data length of 1, or set to indicate a length of 2
                        return PropAddr {
                            addr: property_addr + 1,
                            size_bytes: 1,
                            data_length: if size & 0x40 == 0 { 1 } else { 2 },
                        };
                    } else {
                        return PropAddr {
                            addr: property_addr + 2,
                            size_bytes: 2,
                            data_length: if size & 0x40 == 0 { 1 } else { 2 },
                        };
                    }
                } else {
                    if size & 0x80 == 0 {
                        if size & 0x40 == 0 {
                            //one size & number byte, prop data length 1
                            property_addr += 2; //1 hdr , 1 data
                        } else {
                            //one size & number bytes, data length 2
                            property_addr += 3; //1 hdr, 2 data
                        }
                    } else {
                        let next_id = self.mem.read_u8(property_addr + 1) & 0x3f;
                        if next_id == 0 {
                            property_addr += 64 + 2;
                        } else {
                            property_addr += next_id as u16 + 2;
                        }
                    }
                }
            }
            return PropAddr {
                addr: 0,
                size_bytes: 1,
                data_length: 1,
            };
        } else {
            //scan each property for prop_id
            while self.mem.read_u8(property_addr) != 0 {
                let size = self.mem.read_u8(property_addr);
                if size & 0x1f == prop_id {
                    return PropAddr {
                        addr: property_addr + 1,
                        size_bytes: 1,
                        data_length: 1,
                    };
                } else {
                    property_addr += (size >> 5) as u16 + 2;
                }
            }
            return PropAddr {
                addr: 0,
                size_bytes: 1,
                data_length: 1,
            };
        }
    }

    fn get_prop_next(&self, obj: T, prop_id: u8) -> u8 {
        let top_prop_table_addr = self.props(obj);
        //skip name to first property
        let mut property_addr =
            top_prop_table_addr + self.mem.read_u8(top_prop_table_addr) as u16 * 2 + 1;
        if Object::<T>::WIDE {
            0
        } else {
            if prop_id == 0 {
                //return first prop, bits 0-4
                return self.mem.read_u8(property_addr) & 0x1f;
            } else {
                while self.mem.read_u8(property_addr) != 0 {
                    let size = self.mem.read_u8(property_addr);
                    if size & 0x1f == prop_id {
                        return self.mem.read_u8(property_addr + (size >> 5) as u16 + 2) & 0x1f;
                    } else {
                        property_addr += (size >> 5) as u16 + 2;
                    }
                }
                return 0;
            }
        }
    }
}
