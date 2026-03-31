use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
}

impl ParameterType {
    pub fn to_json_type(&self) -> &str {
        match self {
            ParameterType::String => "string",
            ParameterType::Number => "number",
            ParameterType::Integer => "integer",
            ParameterType::Boolean => "boolean",
            ParameterType::Array => "array",
            ParameterType::Object => "object",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: ParameterType,
    pub description: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
}

impl ToolParameter {
    pub fn required(name: &str, param_type: ParameterType, description: &str) -> Self {
        Self {
            name: name.to_string(),
            param_type,
            description: description.to_string(),
            required: true,
            enum_values: None,
        }
    }

    pub fn optional(name: &str, param_type: ParameterType, description: &str) -> Self {
        Self {
            name: name.to_string(),
            param_type,
            description: description.to_string(),
            required: false,
            enum_values: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolDescriptor {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
}

impl ToolDescriptor {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            parameters: Vec::new(),
        }
    }

    pub fn with_parameters(mut self, params: Vec<ToolParameter>) -> Self {
        self.parameters = params;
        self
    }

    pub fn to_openai_function(&self) -> serde_json::Value {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for param in &self.parameters {
            let mut prop = serde_json::Map::new();
            prop.insert(
                "type".to_string(),
                serde_json::Value::String(param.param_type.to_json_type().to_string()),
            );
            prop.insert(
                "description".to_string(),
                serde_json::Value::String(param.description.clone()),
            );
            if let Some(ref enum_vals) = param.enum_values {
                prop.insert(
                    "enum".to_string(),
                    serde_json::Value::Array(
                        enum_vals
                            .iter()
                            .map(|v| serde_json::Value::String(v.clone()))
                            .collect(),
                    ),
                );
            }
            properties.insert(param.name.clone(), serde_json::Value::Object(prop));

            if param.required {
                required.push(serde_json::Value::String(param.name.clone()));
            }
        }

        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": {
                    "type": "object",
                    "properties": properties,
                    "required": required,
                }
            }
        })
    }
}
