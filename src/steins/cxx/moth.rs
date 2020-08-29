#[cxx::bridge]
pub mod ffi {
  extern "C" {
    // One or more headers with the matching C++ declarations. Our code
    // generators don't read it but it gets #include'd and used in static
    // assertions to ensure our picture of the FFI boundary is accurate.
    include!("cxx/moth.hpp");

    pub unsafe fn do_moth(a: u32) -> u32;
  }
}

#[cfg(test)]
mod moth_tests {
  use super::*;
  #[test]
  fn moth_test() {
    assert_eq!(15, ffi::do_moth(5));
  }
}
