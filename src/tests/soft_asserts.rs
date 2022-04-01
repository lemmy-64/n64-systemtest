use alloc::format;
use alloc::string::String;
use core::fmt::{Debug, Display, LowerHex};

pub fn soft_assert_eq<T: Debug + PartialEq + Eq>(v1: T, v2: T, help: &str) -> Result<(), String> {
    if v1 == v2 {
        Ok(())
    } else {
        Err(format!("a == b expected, but a={:?} b={:?} (hex: a=0x{:x?} b=0x{:x?}). {}", v1, v2, v1, v2, help))
    }
}

pub fn soft_assert_neq<T: Display + LowerHex + PartialEq + Eq>(v1: T, v2: T, help: &str) -> Result<(), String> {
    if v1 != v2 {
        Ok(())
    } else {
        Err(format!("a != b expected, but a={} b={} (hex: a=0x{:x} b=0x{:x}). {}", v1, v2, v1, v2, help))
    }
}

pub fn soft_assert_greater_or_equal(v1: u32, v2: u32, help: &str) -> Result<(), String> {
    if v1 >= v2 {
        Ok(())
    } else {
        Err(format!("a >= b expected, but a={} b={} (hex: a=0x{:x} b=0x{:x}). {}", v1, v2, v1, v2, help))
    }
}

pub fn soft_assert_less(v1: u32, v2: u32, help: &str) -> Result<(), String> {
    if v1 < v2 {
        Ok(())
    } else {
        Err(format!("a < b expected, but a={} b={} (hex: a=0x{:x} b=0x{:x}). {}", v1, v2, v1, v2, help))
    }
}
