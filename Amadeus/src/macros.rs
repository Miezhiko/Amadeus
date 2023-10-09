macro_rules! pub_struct {
  ($name:ident {$($field:ident: $t:ty,)*}) => {
    #[allow(non_snake_case)]
    #[derive(Deserialize, Debug)]
    pub struct $name {
      $(pub $field: $t),*
    }
  }
}

#[macro_export]
macro_rules! set {
  ($init:ident = $val:expr, $($lhs:ident = $rhs:expr),*) => {
      let $init = $val;
    $(
      let $lhs = $rhs;
    )*
  };
}

#[macro_export]
macro_rules! setm {
  ($init:ident = $val:expr, $($lhs:ident = $rhs:expr),*) => {
      let mut $init = $val;
    $(
      let mut $lhs = $rhs;
    )*
  };
}

macro_rules! bjr {
  ($from:expr => $to:expr) => {
    #[allow(clippy::trivial_regex)]
    #[allow(clippy::single_component_path_imports)]
    Rule::new_re(Regex::new($from).unwrap(), $to)
  };
}

macro_rules! bjf {
  ($fun:expr) => { Rule::new_fn($fun) };
}

#[cfg(test)]
mod macros_tests {
  pub_struct!(TestStruct {
    f1: u32,
    f2: u32,
  });
  #[test]
  fn all_macros() {
    set!{ v1 = 5
        , v2 = 6 };
    setm!{ v3 = 5
         , v4 = 4 };
    let ts = TestStruct {
      f1: v1,
      f2: v2
    };
    v3 += ts.f1;
    v4 += ts.f2;
    assert!(v3 + v4 == 20);
  }
}
