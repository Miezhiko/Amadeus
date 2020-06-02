macro_rules! pub_struct {
  ($name:ident {$($field:ident: $t:ty,)*}) => {
    #[allow(non_snake_case)]
    #[derive(Deserialize, Debug)]
    pub struct $name {
      $(pub $field: $t),*
    }
  }
}
