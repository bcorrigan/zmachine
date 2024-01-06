#![allow(dead_code)]
mod error;
mod memory;
mod object;
mod zmachine;
mod zscii;
mod zscreen;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
