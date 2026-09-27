//! In-memory, best-effort per-IP limiters, ported from the maps in `server.ts`.
//!
//! These are process-local (like the originals). The DB-backed global hourly cap
//! remains the hard ceiling; these bound what a single source can do and make an
//! attack visible without a shared store.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

struct Entry {
    count: u32,
    reset_at: Instant,
}

/// A keyed sliding-ish window counter. Entries reset once their window elapses.
///
/// The map is bounded to `max_keys` distinct keys so an attacker rotating source
/// IPs (or an IPv6 range) can't grow it without limit between the periodic sweeps.
/// When full, we first drop expired entries; if still full, we evict the entry
/// closest to expiring to make room. Under a genuine flood this trades a little
/// per-IP accuracy for a hard memory ceiling — the service's stated
/// cost-over-availability posture.
pub struct WindowMap {
    map: Mutex<HashMap<String, Entry>>,
    max_keys: usize,
}

impl WindowMap {
    pub fn new(max_keys: usize) -> WindowMap {
        WindowMap {
            map: Mutex::new(HashMap::new()),
            max_keys: max_keys.max(1),
        }
    }

    /// Ensure there is room for one more key, evicting if necessary. Caller holds
    /// the lock. Returns with `map.len() < max_keys` guaranteed (max_keys >= 1).
    fn make_room(map: &mut HashMap<String, Entry>, max_keys: usize, now: Instant) {
        if map.len() < max_keys {
            return;
        }
        map.retain(|_, e| now <= e.reset_at);
        if map.len() < max_keys {
            return;
        }
        // Still full of live entries — evict the one expiring soonest.
        if let Some(key) = map
            .iter()
            .min_by_key(|(_, e)| e.reset_at)
            .map(|(k, _)| k.clone())
        {
            map.remove(&key);
        }
    }

    /// Increment the counter for `key` and report whether it now exceeds `limit`.
    /// A fresh window starts at count 1 (never over a limit >= 1). Mirrors the
    /// `/subscribe` limiter: the Nth request passes, the (N+1)th is limited.
    pub fn hit_and_check(&self, key: &str, window: Duration, limit: u32) -> bool {
        let now = Instant::now();
        let mut map = self.map.lock().unwrap();
        match map.get_mut(key) {
            Some(e) if now <= e.reset_at => {
                e.count += 1;
                e.count > limit
            }
            _ => {
                Self::make_room(&mut map, self.max_keys, now);
                map.insert(
                    key.to_string(),
                    Entry {
                        count: 1,
                        reset_at: now + window,
                    },
                );
                1 > limit
            }
        }
    }

    /// True if `key` has reached `threshold` within its live window. Does not
    /// increment. An expired window reports "not capped".
    pub fn is_capped(&self, key: &str, threshold: u32) -> bool {
        let now = Instant::now();
        let map = self.map.lock().unwrap();
        matches!(map.get(key), Some(e) if now <= e.reset_at && e.count >= threshold)
    }

    /// Increment the counter for `key`, starting a fresh `window` if none is live.
    pub fn record(&self, key: &str, window: Duration) {
        let now = Instant::now();
        let mut map = self.map.lock().unwrap();
        match map.get_mut(key) {
            Some(e) if now <= e.reset_at => e.count += 1,
            _ => {
                Self::make_room(&mut map, self.max_keys, now);
                map.insert(
                    key.to_string(),
                    Entry {
                        count: 1,
                        reset_at: now + window,
                    },
                );
            }
        }
    }

    /// Forget `key` (e.g. clear an IP's failed-auth count on a success).
    pub fn clear(&self, key: &str) {
        self.map.lock().unwrap().remove(key);
    }

    /// Drop expired entries so the map doesn't grow without bound.
    pub fn sweep(&self) {
        let now = Instant::now();
        self.map.lock().unwrap().retain(|_, e| now <= e.reset_at);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN: Duration = Duration::from_secs(60);

    #[test]
    fn nth_request_passes_and_the_next_is_limited() {
        let rl = WindowMap::new(100);
        for _ in 0..3 {
            assert!(!rl.hit_and_check("ip", MIN, 3));
        }
        assert!(rl.hit_and_check("ip", MIN, 3));
        assert!(
            !rl.hit_and_check("other-ip", MIN, 3),
            "keys are independent"
        );
    }

    #[test]
    fn window_resets_after_it_elapses() {
        let rl = WindowMap::new(100);
        let window = Duration::from_millis(20);
        assert!(!rl.hit_and_check("ip", window, 1));
        assert!(rl.hit_and_check("ip", window, 1));
        std::thread::sleep(Duration::from_millis(40));
        assert!(!rl.hit_and_check("ip", window, 1));

        rl.record("fails", window);
        assert!(rl.is_capped("fails", 1));
        std::thread::sleep(Duration::from_millis(40));
        assert!(!rl.is_capped("fails", 1), "expired window is not capped");
    }

    #[test]
    fn record_and_clear() {
        let rl = WindowMap::new(100);
        rl.record("ip", MIN);
        assert!(!rl.is_capped("ip", 2));
        rl.record("ip", MIN);
        assert!(rl.is_capped("ip", 2));
        rl.clear("ip");
        assert!(!rl.is_capped("ip", 1));
    }

    #[test]
    fn full_map_evicts_the_entry_expiring_soonest() {
        let rl = WindowMap::new(2);
        rl.record("soon", Duration::from_secs(1));
        rl.record("late", MIN);
        rl.record("new", MIN);
        assert!(!rl.is_capped("soon", 1), "evicted");
        assert!(rl.is_capped("late", 1));
        assert!(rl.is_capped("new", 1));
    }
}
