use serde::{
    de::{self, Visitor},
    ser::SerializeTuple,
    Deserialize, Serialize,
};

/// A collection of `Field` instances.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Fields(Vec<Field>);

impl Fields {
    /// Creates a new `Fields` instance from a vector of `Field`.
    pub fn new(fields: Vec<Field>) -> Self {
        Self(fields)
    }

    /// Returns an iterator over the field names.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|field| field.name.as_str())
    }

    /// Returns an iterator over the `Field` references.
    pub fn iter(&self) -> impl Iterator<Item = &Field> {
        self.0.iter()
    }

    /// Returns an iterator over parameters for each field.
    pub fn params(&self) -> impl Iterator<Item = String> + '_ {
        self.iter().flat_map(Field::params)
    }

    /// Returns an iterator over parameter types for each field.
    pub fn param_types(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.iter().flat_map(Field::param_types)
    }

    /// Returns an iterator over test parameters for each field.
    pub fn test_params(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.iter().flat_map(Field::test_params)
    }
}

impl From<Vec<(&str, FieldType)>> for Fields {
    fn from(value: Vec<(&str, FieldType)>) -> Self {
        Self::new(value.into_iter().map(Field::from).collect())
    }
}

/// Represents a field definition for NFT.
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    field_type: FieldType,
}

impl Field {
    /// Creates a new `Field` instance.
    pub fn new(name: String, field_type: FieldType) -> Self {
        Self { name, field_type }
    }

    /// Returns the name of the field.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the `FieldType` of the field.
    pub fn field_type(&self) -> &FieldType {
        &self.field_type
    }

    /// Returns an iterator over parameters associated with the field.
    pub fn params(&self) -> impl Iterator<Item = String> {
        let field_name = self.name();
        match self.field_type() {
            FieldType::String => vec![field_name.to_string()],
            FieldType::Url => vec![field_name.to_string()],
            FieldType::Attributes => vec![
                format!("{field_name}_keys"),
                format!("{field_name}_values"),
            ],
        }
        .into_iter()
    }

    /// Returns an iterator over the types of parameters associated with the field.
    pub fn param_types(&self) -> impl Iterator<Item = &'static str> {
        match self.field_type() {
            FieldType::String => vec!["std::string::String"],
            FieldType::Url => vec!["vector<u8>"],
            FieldType::Attributes => {
                vec!["vector<std::ascii::String>", "vector<std::ascii::String>"]
            }
        }
        .into_iter()
    }

    /// Returns an iterator over test parameters for the field.
    pub fn test_params(&self) -> impl Iterator<Item = &'static str> {
        match self.field_type() {
            FieldType::String => vec!["std::string::utf8(b\"TEST STRING\")"],
            FieldType::Url => vec!["b\"https://originbyte.io\""],
            FieldType::Attributes => vec![
                "vector[std::ascii::string(b\"key\")]",
                "vector[std::ascii::string(b\"attribute\")]",
            ],
        }
        .into_iter()
    }
}

impl From<(&str, FieldType)> for Field {
    fn from(value: (&str, FieldType)) -> Self {
        Self::new(value.0.to_string(), value.1)
    }
}

/// Represents supported field types
///
/// An explicit `FieldType` enum is defined as there is a limited set of
/// acceptable fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    String,
    Url,
    Attributes,
}

/// Custom deserialization for `Field`.
impl<'de> Deserialize<'de> for Field {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct FieldVisitor {}

        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = Field;

            fn expecting(
                &self,
                formatter: &mut std::fmt::Formatter,
            ) -> std::fmt::Result {
                write!(formatter, r#"sequence of field name and field type"#)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let name: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &"2"))?;
                let field_type: FieldType = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &"2"))?;

                Ok(Field { name, field_type })
            }
        }

        deserializer.deserialize_seq(FieldVisitor {})
    }
}

/// Custom serialization for `Field`.
impl Serialize for Field {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut element = serializer.serialize_tuple(2)?;
        element.serialize_element(&self.name)?;
        element.serialize_element(&self.field_type)?;

        element.end()
    }
}
