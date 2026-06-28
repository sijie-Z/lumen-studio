use serde::{Deserialize, Serialize};

/// Phone 值对象 — 中国手机号：1开头的11位数字
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Phone(String);

#[derive(Debug, thiserror::Error)]
pub enum PhoneError {
    #[error("Invalid phone number: {0}")]
    InvalidFormat(String),
}

impl Phone {
    pub fn new(phone: impl Into<String>) -> Result<Self, PhoneError> {
        let phone: String = phone.into();
        let trimmed: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

        if trimmed.len() != 11 || !trimmed.starts_with('1') {
            return Err(PhoneError::InvalidFormat(phone));
        }

        Ok(Self(trimmed))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 脱敏输出：138****1234
    pub fn masked(&self) -> String {
        format!("{}****{}", &self.0[..3], &self.0[7..])
    }
}

impl std::fmt::Display for Phone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
