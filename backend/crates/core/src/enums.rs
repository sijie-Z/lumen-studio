use serde::{Deserialize, Serialize};

/// 用户状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Inactive,
    Banned,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Banned => "banned",
        }
    }
}

/// 预约状态机
/// pending → confirmed → ongoing → completed
///    ↓         ↓          ↓
/// cancelled cancelled  cancelled → refunded
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AppointmentStatus {
    Pending,
    Confirmed,
    Ongoing,
    Completed,
    Cancelled,
    Refunded,
}

impl AppointmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Confirmed => "confirmed",
            Self::Ongoing => "ongoing",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Refunded => "refunded",
        }
    }

    /// 检查是否可以从当前状态转换到目标状态
    pub fn can_transition_to(&self, target: &AppointmentStatus) -> bool {
        matches!(
            (self, target),
            (Self::Pending, Self::Confirmed)
                | (Self::Pending, Self::Cancelled)
                | (Self::Confirmed, Self::Ongoing)
                | (Self::Confirmed, Self::Cancelled)
                | (Self::Ongoing, Self::Completed)
                | (Self::Ongoing, Self::Cancelled)
                | (Self::Cancelled, Self::Refunded)
        )
    }
}

/// 支付状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    Success,
    Failed,
    Refunded,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Refunded => "refunded",
        }
    }
}

/// 支付类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PaymentType {
    Recharge,
    Appointment,
    Withdraw,
}

/// 交付状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Pending,
    InProgress,
    Delivered,
    Failed,
}

/// 认证状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Unverified,
    Pending,
    Verified,
    Rejected,
}

/// 角色类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RoleType {
    User,
    Creator,
    Admin,
    Delivery,
}

impl RoleType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Creator => "creator",
            Self::Admin => "admin",
            Self::Delivery => "delivery",
        }
    }
}

/// 创作者认证等级
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CertificationLevel {
    Standard,
    Premium,
    Elite,
}

/// 通知类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotificationType {
    System,
    Appointment,
    Payment,
    Review,
    Verification,
    Like,
    Certification,
}

/// 通知优先级
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
}
