use native_types::NativeType;
use serde::de::DeserializeOwned;

/// represents an instance of a native type declared in the Prisma schema
#[derive(Debug, Clone, PartialEq)]
pub struct NativeTypeInstance {
    /// the name of the native type used in the Prisma schema
    pub name: String,
    /// the arguments that were provided
    pub args: Vec<String>,
    /// the serialized representation of this native type. The serialized format is generated by the `native-types` library
    pub serialized_native_type: serde_json::Value,
}

impl NativeTypeInstance {
    pub fn new(name: &str, args: Vec<String>, native_type: &dyn NativeType) -> Self {
        NativeTypeInstance {
            name: name.to_string(),
            args,
            serialized_native_type: native_type.to_json(),
        }
    }

    pub fn deserialize_native_type<T>(&self) -> T
    where
        T: DeserializeOwned,
    {
        let error_msg = format!(
            "Deserializing the native type from json failed: {:?}",
            self.serialized_native_type.as_str()
        );
        serde_json::from_value(self.serialized_native_type.clone()).expect(&error_msg)
    }

    pub fn render(&self) -> String {
        if self.args.len() == 0 {
            self.name.to_string()
        } else {
            let args_as_strings: Vec<String> = self.args.iter().map(|a| a.to_string()).collect();
            format!("{}({})", self.name, args_as_strings.join(","))
        }
    }
}
