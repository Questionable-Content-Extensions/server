//! Per-token permission cache to avoid a DB round-trip on every request.

use dashmap::DashMap;
use std::collections::HashSet;
use std::time::{Duration, Instant};

const DEFAULT_TTL: Duration = Duration::from_mins(5);

/// A short-lived in-memory cache mapping token strings to their permission sets.
///
/// The token table changes rarely and the set of tokens is small, so a simple
/// TTL cache with no background eviction is sufficient.
#[derive(Debug)]
pub struct TokenPermissionsCache {
    ttl: Duration,
    inner: DashMap<String, (HashSet<String>, Instant)>,
}

impl Default for TokenPermissionsCache {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenPermissionsCache {
    /// Creates a new cache with a 60-second TTL.
    #[must_use]
    pub fn new() -> Self {
        Self::with_ttl(DEFAULT_TTL)
    }

    fn with_ttl(ttl: Duration) -> Self {
        Self {
            ttl,
            inner: DashMap::new(),
        }
    }

    /// Returns the cached permissions for `token`, or `None` if absent or expired.
    #[must_use]
    pub fn get(&self, token: &str) -> Option<HashSet<String>> {
        self.inner.get(token).and_then(|entry| {
            let (perms, inserted_at) = entry.value();
            if inserted_at.elapsed() < self.ttl {
                Some(perms.clone())
            } else {
                None
            }
        })
    }

    /// Stores `permissions` for `token`, replacing any existing entry.
    pub fn set(&self, token: String, permissions: HashSet<String>) {
        self.inner.insert(token, (permissions, Instant::now()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_returns_none_for_unknown_token() {
        let cache = TokenPermissionsCache::new();
        assert!(cache.get("unknown").is_none());
    }

    #[test]
    fn get_returns_cached_permissions_within_ttl() {
        let cache = TokenPermissionsCache::new();
        let perms: HashSet<String> = ["read".to_string(), "write".to_string()].into();
        cache.set("tok1".to_string(), perms.clone());
        assert_eq!(cache.get("tok1"), Some(perms));
    }

    #[test]
    fn get_returns_none_after_ttl_expires() {
        let cache = TokenPermissionsCache::with_ttl(Duration::from_millis(1));
        cache.set("tok2".to_string(), HashSet::from(["perm".to_string()]));
        std::thread::sleep(Duration::from_millis(5));
        assert!(cache.get("tok2").is_none());
    }

    #[test]
    fn set_overwrites_existing_entry() {
        let cache = TokenPermissionsCache::new();
        cache.set("tok3".to_string(), HashSet::from(["old".to_string()]));
        let new_perms = HashSet::from(["new".to_string()]);
        cache.set("tok3".to_string(), new_perms.clone());
        assert_eq!(cache.get("tok3"), Some(new_perms));
    }
}
