use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::Context as _;
use sha2::Digest as _;

use crate::user_error::UserError;

pub(super) const CONFIRM_TOKEN_TTL: Duration = Duration::from_secs(10 * 60);
const CONFIRM_TOKEN_LEN_BYTES: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(super) struct ConfirmTokenBinding {
    repo: Option<String>,
    profile: Option<String>,
    target: Option<String>,
    machine: Option<String>,
}

impl From<&super::tools::CommonArgs> for ConfirmTokenBinding {
    fn from(value: &super::tools::CommonArgs) -> Self {
        Self {
            repo: value.repo.clone(),
            profile: value.profile.clone(),
            target: value.target.clone(),
            machine: value.machine.clone(),
        }
    }
}

#[derive(Debug, Clone)]
struct ConfirmTokenEntry {
    binding: ConfirmTokenBinding,
    plan_hash: String,
    expires_at: Instant,
}

#[derive(Debug, Default)]
pub(super) struct ConfirmTokenStore {
    tokens: HashMap<String, ConfirmTokenEntry>,
}

impl ConfirmTokenStore {
    fn cleanup_expired(&mut self, now: Instant) {
        // Keep recently expired tokens for a short grace period so callers can get a more
        // actionable `E_CONFIRM_TOKEN_EXPIRED` instead of an unknown-token mismatch.
        self.tokens
            .retain(|_, entry| entry.expires_at + CONFIRM_TOKEN_TTL > now);
    }
}

pub(super) fn insert_token(
    store: &mut ConfirmTokenStore,
    token: String,
    binding: ConfirmTokenBinding,
    plan_hash: String,
    now: Instant,
) {
    let expires_at = now + CONFIRM_TOKEN_TTL;
    store.cleanup_expired(now);
    store.tokens.insert(
        token,
        ConfirmTokenEntry {
            binding,
            plan_hash,
            expires_at,
        },
    );
}

pub(super) fn validate_token(
    store: &mut ConfirmTokenStore,
    token: &str,
    binding: &ConfirmTokenBinding,
    now: Instant,
) -> Result<String, UserError> {
    let Some(entry) = store.tokens.get(token).cloned() else {
        store.cleanup_expired(now);
        return Err(UserError::confirm_token_mismatch());
    };
    if entry.expires_at <= now {
        store.tokens.remove(token);
        store.cleanup_expired(now);
        return Err(UserError::confirm_token_expired());
    }
    if &entry.binding != binding {
        store.cleanup_expired(now);
        return Err(UserError::confirm_token_mismatch());
    }

    store.cleanup_expired(now);
    Ok(entry.plan_hash)
}

pub(super) fn consume_token(store: &mut ConfirmTokenStore, token: &str) {
    store.tokens.remove(token);
}

pub(super) fn compute_confirm_plan_hash(
    binding: &ConfirmTokenBinding,
    envelope: &serde_json::Value,
) -> anyhow::Result<String> {
    let data = envelope
        .get("data")
        .cloned()
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    let hash_input = serde_json::json!({
        "binding": binding,
        "data": data,
    });
    let bytes = serde_json::to_vec(&hash_input).context("serialize confirm_plan_hash input")?;
    Ok(hex::encode(sha2::Sha256::digest(bytes)))
}

pub(super) fn generate_confirm_token() -> anyhow::Result<String> {
    let mut bytes = [0u8; CONFIRM_TOKEN_LEN_BYTES];
    getrandom::fill(&mut bytes).map_err(|e| anyhow::anyhow!("generate confirm_token: {e}"))?;
    Ok(hex::encode(bytes))
}
