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
