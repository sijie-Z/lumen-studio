use crate::state::AppState;
use axum::{
    extract::{ConnectInfo, Request, State},
    middleware::Next,
    response::Response,
};
use common::AppError;
use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter};
use std::{
    net::{IpAddr, SocketAddr},
    num::NonZeroU32,
    sync::Arc,
};

pub type KeyedRateLimiter = DefaultKeyedRateLimiter<IpAddr>;

pub fn from_env() -> anyhow::Result<Option<Arc<KeyedRateLimiter>>> {
    let enabled = std::env::var("RATE_LIMIT_ENABLED")
        .map(|value| {
            !matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "0" | "false" | "off"
            )
        })
        .unwrap_or(true);
    if !enabled {
        return Ok(None);
    }

    let requests_per_second = parse_positive_env("RATE_LIMIT_REQUESTS_PER_SECOND", 20)?;
    let burst = parse_positive_env(
        "RATE_LIMIT_BURST",
        requests_per_second.get().saturating_mul(2),
    )?;
    let quota = Quota::per_second(requests_per_second).allow_burst(burst);
    Ok(Some(Arc::new(RateLimiter::keyed(quota))))
}

pub fn check(limiter: Option<&KeyedRateLimiter>, client_ip: IpAddr) -> Result<(), AppError> {
    match limiter {
        Some(limiter) => limiter
            .check_key(&client_ip)
            .map_err(|_| AppError::RateLimited),
        None => Ok(()),
    }
}

pub async fn enforce(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    if let Err(error) = check(state.rate_limiter.as_deref(), addr.ip()) {
        return error.into_response();
    }
    next.run(request).await
}

fn parse_positive_env(name: &str, default: u32) -> anyhow::Result<NonZeroU32> {
    let value = std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(|value| value.parse::<u32>())
        .transpose()
        .map_err(|error| anyhow::anyhow!("{name} must be a positive integer: {error}"))?
        .unwrap_or(default);
    NonZeroU32::new(value).ok_or_else(|| anyhow::anyhow!("{name} must be greater than zero"))
}

use axum::response::IntoResponse;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limiter_allows_burst_then_rejects_same_ip() {
        let limiter = RateLimiter::keyed(
            Quota::per_second(NonZeroU32::new(1).unwrap()).allow_burst(NonZeroU32::new(1).unwrap()),
        );
        let ip = "127.0.0.1".parse().unwrap();
        assert!(check(Some(&limiter), ip).is_ok());
        assert!(matches!(
            check(Some(&limiter), ip),
            Err(AppError::RateLimited)
        ));
    }
}
