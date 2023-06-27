use crate::memory::Memory;
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

impl<'a, T> Object<'_, T>
where
    T: Integer + Into<u8> + Into<u16> + Copy + ReadObject,
{
    const PARENT: u16 = 4;
    const SIBLING: u16 = 5;
    const CHILD: u16 = 6;
    const PROPS: u16 = 7;
    const SIZE: u16 = 9;

    const PROPMAX: u16 = 31;

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

    fn print(&mut self, obj: T) {
        if self.mem.read_u8(self.props(obj)) != 0 {
            //ZSCII(obj+1)
            //print(zscii_buf,zscii_ptr);
        }
    }
}
