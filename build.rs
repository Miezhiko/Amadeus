fn main() {
  cxx_build::bridge("src/stains/cxx/moth.rs")  // returns a cc::Build
      .file("cxx/moth.cxx")
      .flag_if_supported("-std=c++17")
      .compile("moth");

  println!("cargo:rerun-if-changed=src/stains/cxx/moth.rs");
  println!("cargo:rerun-if-changed=cxx/moth.hpp");
  println!("cargo:rerun-if-changed=cxx/moth.cxx");
}
