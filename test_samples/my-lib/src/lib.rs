mod module_in_file;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

enum SomeEnum {
    Variant1,
    Variant2(bool),
    Variant3 { x: u64, y: u64 },
}

#[derive(Debug)]
struct MyStruct {
    b: bool,
    x: u8,
}

mod utils {
    fn util_func() {
        println!("util_func");
    }

    #[derive(Debug)]
    struct UtilStruct {
        u: u8,
    }

    impl UtilStruct {
        pub fn new(u: u8) -> Self {
            Self {
                u
            }
        }
    }

    // Putting this here to test multiple impl blocks for the same struct, with unrelated items in
    // between them.
    fn other_func() {
        println!("other_func");
    }

    impl UtilStruct {
        pub fn old(&self) -> u8 {
            self.u
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
