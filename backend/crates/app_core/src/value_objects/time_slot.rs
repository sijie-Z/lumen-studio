use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// TimeSlot 值对象 — 编译期保证结束>开始，时长1-8小时，不能是过去
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeSlot {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum TimeSlotError {
    #[error("End time must be after start time")]
    EndBeforeStart,
    #[error("Duration must be between 1 and 8 hours, got {0} minutes")]
    InvalidDuration(i64),
    #[error("Cannot create a time slot in the past")]
    TimeInPast,
}

impl TimeSlot {
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Self, TimeSlotError> {
        let now = Utc::now();
        if start < now || end < now {
            return Err(TimeSlotError::TimeInPast);
        }

        if end <= start {
            return Err(TimeSlotError::EndBeforeStart);
        }

        let duration_minutes = (end - start).num_minutes();
        if duration_minutes < 60 || duration_minutes > 480 {
            return Err(TimeSlotError::InvalidDuration(duration_minutes));
        }

        Ok(Self { start, end })
    }

    /// 从起始时间+分钟创建
    pub fn from_duration(start: DateTime<Utc>, duration_minutes: i64) -> Result<Self, TimeSlotError> {
        Self::new(start, start + Duration::minutes(duration_minutes))
    }

    pub fn duration_minutes(&self) -> i64 {
        (self.end - self.start).num_minutes()
    }

    /// 检查两个时段是否有交集
    pub fn overlaps(&self, other: &TimeSlot) -> bool {
        self.start < other.end && other.start < self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(start: DateTime<Utc>, duration_minutes: i64) -> TimeSlot {
        TimeSlot::from_duration(start, duration_minutes).unwrap()
    }

    #[test]
    fn detects_overlapping_slots() {
        let base = Utc::now() + Duration::days(1);
        let a = slot(base, 120);
        let b = slot(base + Duration::minutes(60), 120);
        assert!(a.overlaps(&b));
    }

    #[test]
    fn rejects_slots_shorter_than_one_hour() {
        let base = Utc::now() + Duration::days(1);
        assert!(TimeSlot::from_duration(base, 30).is_err());
    }
}
