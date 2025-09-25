use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthEnum {
    Login,
    Register
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    Password,
    Sms,
    Email
}

impl From<String> for AuthType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "password" => AuthType::Password,
            "sms" => AuthType::Sms,
            "email" => AuthType::Email,
            _ => AuthType::Password,
        }
    }
}