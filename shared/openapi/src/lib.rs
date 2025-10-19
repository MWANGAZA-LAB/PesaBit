/// OpenAPI documentation generation for PesaBit
/// 
/// This library provides automatic OpenAPI 3.0 specification generation
/// for all PesaBit services with proper documentation and examples.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenAPI 3.0 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: Info,
    pub servers: Vec<Server>,
    pub paths: HashMap<String, PathItem>,
    pub components: Option<Components>,
    pub security: Option<Vec<SecurityRequirement>>,
    pub tags: Option<Vec<Tag>>,
}

/// API information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub title: String,
    pub description: String,
    pub version: String,
    pub contact: Option<Contact>,
    pub license: Option<License>,
}

/// Contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    pub email: String,
    pub url: Option<String>,
}

/// License information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub name: String,
    pub url: Option<String>,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub url: String,
    pub description: Option<String>,
}

/// Path item containing HTTP methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathItem {
    pub get: Option<Operation>,
    pub post: Option<Operation>,
    pub put: Option<Operation>,
    pub delete: Option<Operation>,
    pub patch: Option<Operation>,
    pub options: Option<Operation>,
}

/// API operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub summary: String,
    pub description: Option<String>,
    pub operation_id: String,
    pub tags: Option<Vec<String>>,
    pub parameters: Option<Vec<Parameter>>,
    pub request_body: Option<RequestBody>,
    pub responses: HashMap<String, Response>,
    pub security: Option<Vec<SecurityRequirement>>,
}

/// Parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub r#in: String, // "query", "header", "path", "cookie"
    pub description: Option<String>,
    pub required: bool,
    pub schema: Schema,
}

/// Request body definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    pub description: Option<String>,
    pub content: HashMap<String, MediaType>,
    pub required: bool,
}

/// Response definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub description: String,
    pub content: Option<HashMap<String, MediaType>>,
    pub headers: Option<HashMap<String, Header>>,
}

/// Media type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    pub schema: Schema,
    pub example: Option<serde_json::Value>,
}

/// Header definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub description: Option<String>,
    pub schema: Schema,
}

/// Schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    #[serde(rename = "type")]
    pub schema_type: Option<String>,
    pub format: Option<String>,
    pub description: Option<String>,
    pub properties: Option<HashMap<String, Schema>>,
    pub items: Option<Box<Schema>>,
    pub required: Option<Vec<String>>,
    pub example: Option<serde_json::Value>,
    pub enum_values: Option<Vec<serde_json::Value>>,
}

/// Security requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirement {
    #[serde(flatten)]
    pub requirements: HashMap<String, Vec<String>>,
}

/// Components section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    pub schemas: Option<HashMap<String, Schema>>,
    pub security_schemes: Option<HashMap<String, SecurityScheme>>,
}

/// Security scheme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScheme {
    #[serde(rename = "type")]
    pub scheme_type: String,
    pub description: Option<String>,
    pub name: Option<String>,
    pub r#in: Option<String>,
    pub scheme: Option<String>,
    pub bearer_format: Option<String>,
}

/// Tag definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub description: Option<String>,
}

impl OpenApiSpec {
    /// Create a new OpenAPI specification for PesaBit
    pub fn new() -> Self {
        Self {
            openapi: "3.0.3".to_string(),
            info: Info {
                title: "PesaBit API".to_string(),
                description: "Lightning-fast money transfer API connecting M-Pesa with Bitcoin Lightning Network".to_string(),
                version: "1.0.0".to_string(),
                contact: Some(Contact {
                    name: "PesaBit Support".to_string(),
                    email: "support@pesa.co.ke".to_string(),
                    url: Some("https://pesa.co.ke".to_string()),
                }),
                license: Some(License {
                    name: "MIT".to_string(),
                    url: Some("https://opensource.org/licenses/MIT".to_string()),
                }),
            },
            servers: vec![
                Server {
                    url: "https://api.pesa.co.ke/v1".to_string(),
                    description: Some("Production server".to_string()),
                },
                Server {
                    url: "http://localhost:3000/v1".to_string(),
                    description: Some("Development server".to_string()),
                },
            ],
            paths: HashMap::new(),
            components: Some(Components {
                schemas: Some(HashMap::new()),
                security_schemes: Some(HashMap::new()),
            }),
            security: Some(vec![
                SecurityRequirement {
                    requirements: {
                        let mut req = HashMap::new();
                        req.insert("BearerAuth".to_string(), vec![]);
                        req
                    },
                },
            ]),
            tags: Some(vec![
                Tag {
                    name: "Authentication".to_string(),
                    description: Some("User authentication and authorization".to_string()),
                },
                Tag {
                    name: "Users".to_string(),
                    description: Some("User management and profiles".to_string()),
                },
                Tag {
                    name: "Payments".to_string(),
                    description: Some("Payment processing and wallet management".to_string()),
                },
                Tag {
                    name: "Lightning".to_string(),
                    description: Some("Bitcoin Lightning Network operations".to_string()),
                },
                Tag {
                    name: "M-Pesa".to_string(),
                    description: Some("M-Pesa integration operations".to_string()),
                },
            ]),
        }
    }

    /// Add authentication endpoints
    pub fn add_auth_endpoints(&mut self) {
        // POST /auth/register
        self.add_endpoint(
            "/auth/register",
            "post",
            Operation {
                summary: "Register new user".to_string(),
                description: Some("Register a new user with phone number and send OTP for verification".to_string()),
                operation_id: "register_user".to_string(),
                tags: Some(vec!["Authentication".to_string()]),
                parameters: None,
                request_body: Some(RequestBody {
                    description: Some("User registration data".to_string()),
                    content: {
                        let mut content = HashMap::new();
                        content.insert("application/json".to_string(), MediaType {
                            schema: Schema {
                                schema_type: Some("object".to_string()),
                                properties: Some({
                                    let mut props = HashMap::new();
                                    props.insert("phone_number".to_string(), Schema {
                                        schema_type: Some("string".to_string()),
                                        format: Some("phone".to_string()),
                                        description: Some("Phone number in E.164 format".to_string()),
                                        example: Some(serde_json::Value::String("+254712345678".to_string())),
                                        ..Default::default()
                                    });
                                    props.insert("full_name".to_string(), Schema {
                                        schema_type: Some("string".to_string()),
                                        description: Some("User's full name".to_string()),
                                        example: Some(serde_json::Value::String("John Doe".to_string())),
                                        ..Default::default()
                                    });
                                    props
                                }),
                                required: Some(vec!["phone_number".to_string()]),
                                ..Default::default()
                            },
                            example: Some(serde_json::json!({
                                "phone_number": "+254712345678",
                                "full_name": "John Doe"
                            })),
                        });
                        content
                    },
                    required: true,
                }),
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert("200".to_string(), Response {
                        description: "Registration successful".to_string(),
                        content: Some({
                            let mut content = HashMap::new();
                            content.insert("application/json".to_string(), MediaType {
                                schema: Schema {
                                    schema_type: Some("object".to_string()),
                                    properties: Some({
                                        let mut props = HashMap::new();
                                        props.insert("verification_token".to_string(), Schema {
                                            schema_type: Some("string".to_string()),
                                            description: Some("Token for OTP verification".to_string()),
                                            ..Default::default()
                                        });
                                        props.insert("message".to_string(), Schema {
                                            schema_type: Some("string".to_string()),
                                            description: Some("Success message".to_string()),
                                            ..Default::default()
                                        });
                                        props
                                    }),
                                    ..Default::default()
                                },
                                example: Some(serde_json::json!({
                                    "verification_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
                                    "message": "OTP sent to your phone"
                                })),
                            });
                            content
                        }),
                        headers: None,
                    });
                    responses.insert("400".to_string(), Response {
                        description: "Bad request".to_string(),
                        content: Some({
                            let mut content = HashMap::new();
                            content.insert("application/json".to_string(), MediaType {
                                schema: Schema {
                                    schema_type: Some("object".to_string()),
                                    properties: Some({
                                        let mut props = HashMap::new();
                                        props.insert("error".to_string(), Schema {
                                            schema_type: Some("string".to_string()),
                                            description: Some("Error code".to_string()),
                                            ..Default::default()
                                        });
                                        props.insert("message".to_string(), Schema {
                                            schema_type: Some("string".to_string()),
                                            description: Some("Error message".to_string()),
                                            ..Default::default()
                                        });
                                        props
                                    }),
                                    ..Default::default()
                                },
                                ..Default::default()
                            });
                            content
                        }),
                        headers: None,
                    });
                    responses
                },
                security: None,
            },
        );

        // POST /auth/verify-otp
        self.add_endpoint(
            "/auth/verify-otp",
            "post",
            Operation {
                summary: "Verify OTP and complete registration".to_string(),
                description: Some("Verify the OTP code sent to user's phone and complete registration".to_string()),
                operation_id: "verify_otp".to_string(),
                tags: Some(vec!["Authentication".to_string()]),
                parameters: None,
                request_body: Some(RequestBody {
                    description: Some("OTP verification data".to_string()),
                    content: {
                        let mut content = HashMap::new();
                        content.insert("application/json".to_string(), MediaType {
                            schema: Schema {
                                schema_type: Some("object".to_string()),
                                properties: Some({
                                    let mut props = HashMap::new();
                                    props.insert("verification_token".to_string(), Schema {
                                        schema_type: Some("string".to_string()),
                                        description: Some("Token from registration response".to_string()),
                                        ..Default::default()
                                    });
                                    props.insert("otp_code".to_string(), Schema {
                                        schema_type: Some("string".to_string()),
                                        description: Some("6-digit OTP code".to_string()),
                                        example: Some(serde_json::Value::String("123456".to_string())),
                                        ..Default::default()
                                    });
                                    props.insert("pin".to_string(), Schema {
                                        schema_type: Some("string".to_string()),
                                        description: Some("4-digit PIN for future authentication".to_string()),
                                        example: Some(serde_json::Value::String("1234".to_string())),
                                        ..Default::default()
                                    });
                                    props
                                }),
                                required: Some(vec!["verification_token".to_string(), "otp_code".to_string(), "pin".to_string()]),
                                ..Default::default()
                            },
                            example: Some(serde_json::json!({
                                "verification_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
                                "otp_code": "123456",
                                "pin": "1234"
                            })),
                        });
                        content
                    },
                    required: true,
                }),
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert("200".to_string(), Response {
                        description: "Verification successful".to_string(),
                        content: Some({
                            let mut content = HashMap::new();
                            content.insert("application/json".to_string(), MediaType {
                                schema: Schema {
                                    schema_type: Some("object".to_string()),
                                    properties: Some({
                                        let mut props = HashMap::new();
                                        props.insert("access_token".to_string(), Schema {
                                            schema_type: Some("string".to_string()),
                                            description: Some("JWT access token".to_string()),
                                            ..Default::default()
                                        });
                                        props.insert("refresh_token".to_string(), Schema {
                                            schema_type: Some("string".to_string()),
                                            description: Some("JWT refresh token".to_string()),
                                            ..Default::default()
                                        });
                                        props.insert("expires_in".to_string(), Schema {
                                            schema_type: Some("integer".to_string()),
                                            description: Some("Token expiry time in seconds".to_string()),
                                            ..Default::default()
                                        });
                                        props.insert("token_type".to_string(), Schema {
                                            schema_type: Some("string".to_string()),
                                            description: Some("Token type".to_string()),
                                            ..Default::default()
                                        });
                                        props
                                    }),
                                    ..Default::default()
                                },
                                example: Some(serde_json::json!({
                                    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
                                    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
                                    "expires_in": 900,
                                    "token_type": "Bearer"
                                })),
                            });
                            content
                        }),
                        headers: None,
                    });
                    responses
                },
                security: None,
            },
        );

        // POST /auth/login
        self.add_endpoint(
            "/auth/login",
            "post",
            Operation {
                summary: "Login with phone number and PIN".to_string(),
                description: Some("Authenticate user with phone number and PIN".to_string()),
                operation_id: "login_user".to_string(),
                tags: Some(vec!["Authentication".to_string()]),
                parameters: None,
                request_body: Some(RequestBody {
                    description: Some("Login credentials".to_string()),
                    content: {
                        let mut content = HashMap::new();
                        content.insert("application/json".to_string(), MediaType {
                            schema: Schema {
                                schema_type: Some("object".to_string()),
                                properties: Some({
                                    let mut props = HashMap::new();
                                    props.insert("phone_number".to_string(), Schema {
                                        schema_type: Some("string".to_string()),
                                        format: Some("phone".to_string()),
                                        description: Some("Phone number in E.164 format".to_string()),
                                        example: Some(serde_json::Value::String("+254712345678".to_string())),
                                        ..Default::default()
                                    });
                                    props.insert("pin".to_string(), Schema {
                                        schema_type: Some("string".to_string()),
                                        description: Some("4-digit PIN".to_string()),
                                        example: Some(serde_json::Value::String("1234".to_string())),
                                        ..Default::default()
                                    });
                                    props
                                }),
                                required: Some(vec!["phone_number".to_string(), "pin".to_string()]),
                                ..Default::default()
                            },
                            example: Some(serde_json::json!({
                                "phone_number": "+254712345678",
                                "pin": "1234"
                            })),
                        });
                        content
                    },
                    required: true,
                }),
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert("200".to_string(), Response {
                        description: "Login successful".to_string(),
                        content: Some({
                            let mut content = HashMap::new();
                            content.insert("application/json".to_string(), MediaType {
                                schema: Schema {
                                    schema_type: Some("object".to_string()),
                                    properties: Some({
                                        let mut props = HashMap::new();
                                        props.insert("access_token".to_string(), Schema {
                                            schema_type: Some("string".to_string()),
                                            description: Some("JWT access token".to_string()),
                                            ..Default::default()
                                        });
                                        props.insert("refresh_token".to_string(), Schema {
                                            schema_type: Some("string".to_string()),
                                            description: Some("JWT refresh token".to_string()),
                                            ..Default::default()
                                        });
                                        props.insert("expires_in".to_string(), Schema {
                                            schema_type: Some("integer".to_string()),
                                            description: Some("Token expiry time in seconds".to_string()),
                                            ..Default::default()
                                        });
                                        props.insert("token_type".to_string(), Schema {
                                            schema_type: Some("string".to_string()),
                                            description: Some("Token type".to_string()),
                                            ..Default::default()
                                        });
                                        props
                                    }),
                                    ..Default::default()
                                },
                                ..Default::default()
                            });
                            content
                        }),
                        headers: None,
                    });
                    responses.insert("401".to_string(), Response {
                        description: "Unauthorized".to_string(),
                        content: Some({
                            let mut content = HashMap::new();
                            content.insert("application/json".to_string(), MediaType {
                                schema: Schema {
                                    schema_type: Some("object".to_string()),
                                    properties: Some({
                                        let mut props = HashMap::new();
                                        props.insert("error".to_string(), Schema {
                                            schema_type: Some("string".to_string()),
                                            description: Some("Error code".to_string()),
                                            ..Default::default()
                                        });
                                        props.insert("message".to_string(), Schema {
                                            schema_type: Some("string".to_string()),
                                            description: Some("Error message".to_string()),
                                            ..Default::default()
                                        });
                                        props
                                    }),
                                    ..Default::default()
                                },
                                ..Default::default()
                            });
                            content
                        }),
                        headers: None,
                    });
                    responses
                },
                security: None,
            },
        );
    }

    /// Add payment endpoints
    pub fn add_payment_endpoints(&mut self) {
        // GET /balance
        self.add_endpoint(
            "/balance",
            "get",
            Operation {
                summary: "Get wallet balance".to_string(),
                description: Some("Get user's current wallet balance in both KES and Bitcoin".to_string()),
                operation_id: "get_balance".to_string(),
                tags: Some(vec!["Payments".to_string()]),
                parameters: None,
                request_body: None,
                responses: {
                    let mut responses = HashMap::new();
                    responses.insert("200".to_string(), Response {
                        description: "Balance retrieved successfully".to_string(),
                        content: Some({
                            let mut content = HashMap::new();
                            content.insert("application/json".to_string(), MediaType {
                                schema: Schema {
                                    schema_type: Some("object".to_string()),
                                    properties: Some({
                                        let mut props = HashMap::new();
                                        props.insert("balance_sats".to_string(), Schema {
                                            schema_type: Some("integer".to_string()),
                                            description: Some("Bitcoin balance in satoshis".to_string()),
                                            example: Some(serde_json::Value::Number(1000000.into())),
                                            ..Default::default()
                                        });
                                        props.insert("balance_kes".to_string(), Schema {
                                            schema_type: Some("number".to_string()),
                                            format: Some("decimal".to_string()),
                                            description: Some("M-Pesa balance in Kenyan Shillings".to_string()),
                                            example: Some(serde_json::Value::Number(5000.into())),
                                            ..Default::default()
                                        });
                                        props.insert("pending_balance_sats".to_string(), Schema {
                                            schema_type: Some("integer".to_string()),
                                            description: Some("Pending Lightning balance in satoshis".to_string()),
                                            example: Some(serde_json::Value::Number(0.into())),
                                            ..Default::default()
                                        });
                                        props
                                    }),
                                    ..Default::default()
                                },
                                example: Some(serde_json::json!({
                                    "balance_sats": 1000000,
                                    "balance_kes": 5000.00,
                                    "pending_balance_sats": 0
                                })),
                            });
                            content
                        }),
                        headers: None,
                    });
                    responses
                },
                security: Some(vec![
                    SecurityRequirement {
                        requirements: {
                            let mut req = HashMap::new();
                            req.insert("BearerAuth".to_string(), vec![]);
                            req
                        },
                    },
                ]),
            },
        );
    }

    /// Add security schemes
    pub fn add_security_schemes(&mut self) {
        if let Some(ref mut components) = self.components {
            if let Some(ref mut security_schemes) = components.security_schemes {
                security_schemes.insert(
                    "BearerAuth".to_string(),
                    SecurityScheme {
                        scheme_type: "http".to_string(),
                        description: Some("JWT Bearer token authentication".to_string()),
                        scheme: Some("bearer".to_string()),
                        bearer_format: Some("JWT".to_string()),
                        name: None,
                        r#in: None,
                    },
                );
            }
        }
    }

    /// Add an endpoint to the specification
    fn add_endpoint(&mut self, path: &str, method: &str, operation: Operation) {
        let path_item = self.paths.entry(path.to_string()).or_insert(PathItem {
            get: None,
            post: None,
            put: None,
            delete: None,
            patch: None,
            options: None,
        });

        match method {
            "get" => path_item.get = Some(operation),
            "post" => path_item.post = Some(operation),
            "put" => path_item.put = Some(operation),
            "delete" => path_item.delete = Some(operation),
            "patch" => path_item.patch = Some(operation),
            "options" => path_item.options = Some(operation),
            _ => {}
        }
    }

    /// Generate the complete OpenAPI specification
    pub fn generate() -> Self {
        let mut spec = Self::new();
        spec.add_auth_endpoints();
        spec.add_payment_endpoints();
        spec.add_security_schemes();
        spec
    }
}

impl Default for Schema {
    fn default() -> Self {
        Self {
            schema_type: None,
            format: None,
            description: None,
            properties: None,
            items: None,
            required: None,
            example: None,
            enum_values: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_spec_creation() {
        let spec = OpenApiSpec::generate();
        assert_eq!(spec.openapi, "3.0.3");
        assert_eq!(spec.info.title, "PesaBit API");
        assert!(!spec.paths.is_empty());
    }

    #[test]
    fn test_auth_endpoints() {
        let mut spec = OpenApiSpec::new();
        spec.add_auth_endpoints();
        
        assert!(spec.paths.contains_key("/auth/register"));
        assert!(spec.paths.contains_key("/auth/verify-otp"));
        assert!(spec.paths.contains_key("/auth/login"));
    }

    #[test]
    fn test_payment_endpoints() {
        let mut spec = OpenApiSpec::new();
        spec.add_payment_endpoints();
        
        assert!(spec.paths.contains_key("/balance"));
    }

    #[test]
    fn test_security_schemes() {
        let mut spec = OpenApiSpec::new();
        spec.add_security_schemes();
        
        assert!(spec.components.as_ref().unwrap().security_schemes.as_ref().unwrap().contains_key("BearerAuth"));
    }
}
