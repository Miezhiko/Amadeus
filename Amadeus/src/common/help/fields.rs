use std::iter::IntoIterator;
use serenity::builder::CreateEmbed;

pub trait FieldsVec {
  fn fields_vec<It>(&mut self, fields: It) -> &mut Self
  where
    It: IntoIterator<Item = (String, String, bool)>;
}

impl FieldsVec for CreateEmbed {
  fn fields_vec<It>(&mut self, fields: It) -> &mut Self
  where
      It: IntoIterator<Item = (String, String, bool)>,
  {
    for (name, value, inline) in fields {
      self.field(name.as_str(), value.as_str(), inline);
    }

    self
  }
}


pub trait FieldsVec2 {
  fn fields_vec2<'a, It>(&mut self, fields: It) -> &mut Self
  where
    It: IntoIterator<Item = (&'a str, String, bool)>;
}

impl FieldsVec2 for CreateEmbed {
  fn fields_vec2<'a, It>(&mut self, fields: It) -> &mut Self
  where
      It: IntoIterator<Item = (&'a str, String, bool)>,
  {
    for (name, value, inline) in fields {
      self.field(name, value.as_str(), inline);
    }

    self
  }
}
