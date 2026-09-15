use serde::{Deserialize, Serialize};

/// Email 值对象 — 编译期保证格式合法
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email(String);

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("Invalid email address: {0}")]
    InvalidFormat(String),
    #[error("Email is too long: {0}")]
    TooLong(usize),
}

impl Email {
    pub const MAX_LEN: usize = 120;

    pub fn new(email: impl Into<String>) -> Result<Self, EmailError> {
        let email: String = email.into();
        let trimmed = email.trim().to_lowercase();

        if trimmed.len() > Self::MAX_LEN {
            return Err(EmailError::TooLong(trimmed.len()));
        }

        // 基本格式校验
        if !trimmed.contains('@') || !trimmed.contains('.') {
            return Err(EmailError::InvalidFormat(trimmed));
        }

        let parts: Vec<&str> = trimmed.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(EmailError::InvalidFormat(trimmed));
        }

        Ok(Self(trimmed))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_email_and_normalizes_case() {
        let email = Email::new("User@Example.COM").unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn rejects_missing_at() {
        assert!(Email::new("invalid.example.com").is_err());
    }
}
