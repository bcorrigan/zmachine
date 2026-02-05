#![allow(dead_code)]

pub mod error;
pub mod zmachine;
pub mod zscreen;

mod instruction;
mod memory;
mod object;
mod zscii;

pub use zmachine::ZMachine;
pub use zscreen::ZScreen;
pub use error::Error;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
