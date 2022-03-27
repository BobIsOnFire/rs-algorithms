#![feature(slice_take)]
#![no_std]

extern crate alloc;

pub mod sort;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
