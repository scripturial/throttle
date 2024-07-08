//! Throttle is an activity counter that can be used to monitor
//! and limit activity such as incoming connections and sign in
//! attempts.
//!
//!
//! # Examples
//!
//! Limit calls to an API to 5 per second, or lockout for one minute
//!
//! ```
//! use throttle2::Throttle;
//!
//! let mut counter = Throttle::new(1000, 5, 1000*60);
//! if counter.is_throttled() {
//!     println!("Try again later")
//! }
//! ```
//!
//! Limit signin attempts on an email address to 5 per minute, or
//! lockout for 5 minutes.
//!
//! ```
//! use throttle2::ThrottleHash;
//!
//! let mut counter = ThrottleHash::new(60*1000, 5, 3*60*1000);
//! let email:String = "john@example.com".to_string();
//! if counter.is_throttled(&email) {
//!     println!("Try again later")
//! }
//! ```
//!

use core::hash::Hash;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Throttle is an activity counter that can be used to monitor
/// and limit activity such as incoming connections and sign in
/// attempts.
pub struct Throttle {
    interval_duration: u128,
    max_hits_in_interval: u64,
    lockout_duration: u128,
    counter: Counter,
}

pub struct Counter {
    interval_start: u128,
    current_hit_counter: u64,
    locked_until: u128,
}

impl Throttle {
    /// Within `interval` only allow `max_hits` or the locked status is set for `lockout_duration`
    pub fn new(interval: u128, max_hits: u64, lockout_duration: u128) -> Throttle {
        //println!("Maximum {} hits in {} millisconds.\n", max_hits, interval);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        Throttle {
            interval_duration: interval,
            max_hits_in_interval: max_hits,
            lockout_duration: lockout_duration,
            counter: Counter {
                interval_start: now,
                current_hit_counter: 0,
                locked_until: 0,
            },
        }
    }

    /// When a monitored activity occurs, `is_throttled()` counts that event and
    /// returns `true` if the activity count has exceeded the limit.
    pub fn is_throttled(&mut self) -> bool {
        self.counter.current_hit_counter += 1;
        let mut now: u128 = 0;
        if self.counter.locked_until != 0 {
            now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            if self.counter.locked_until > now {
                return true;
            }
            self.counter.locked_until = 0;
        }
        if self.counter.current_hit_counter <= self.max_hits_in_interval {
            return false;
        }
        if now == 0 {
            now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
        }
        if self.counter.locked_until > 0 {
            if self.counter.locked_until > now {
                return true;
            }
            //println!("reset all");
            self.counter.interval_start = now;
            self.counter.locked_until = 0;
            self.counter.current_hit_counter = 1;
            return false;
        }
        if now - self.counter.interval_start <= self.interval_duration {
            self.counter.interval_start = now;
            self.counter.current_hit_counter = 1;
            self.counter.locked_until = now + self.lockout_duration;
            return true;
        }
        self.counter.interval_start = now;
        self.counter.current_hit_counter = 1;
        return false;
    }
}

pub struct ThrottleHash<H: Eq + Hash + Clone> {
    interval_duration: u128,
    max_hits_in_interval: u64,
    lockout_duration: u128,
    counters: HashMap<H, Counter>,
}

impl<H: Eq + Hash + Clone> ThrottleHash<H> {
    /// Within `interval` only allow `max_hits` or the locked status is set for `lockout_duration`
    pub fn new(interval: u128, max_hits: u64, lockout_duration: u128) -> Self {
        //println!("Maximum {} hits in {} millisconds.\n", max_hits, interval);
        ThrottleHash {
            interval_duration: interval,
            max_hits_in_interval: max_hits,
            lockout_duration: lockout_duration,
            counters: HashMap::<H, Counter>::new(),
        }
    }

    /// When a monitored activity occurs, `is_throttled()` counts that event and
    /// returns `true` if the activity count has exceeded the limit.
    pub fn is_throttled(&mut self, key: &H) -> bool {
        let counter = match self.counters.get_mut(key) {
            Some(c) => c,
            None => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let c = Counter {
                    interval_start: now,
                    locked_until: 0,
                    current_hit_counter: 0,
                };
                self.counters.insert(key.clone(), c);
                self.counters.get_mut(key).unwrap()
            }
        };

        counter.current_hit_counter += 1;
        let mut now: u128 = 0;
        if counter.locked_until != 0 {
            now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            if counter.locked_until > now {
                return true;
            }
            counter.locked_until = 0;
        }
        if counter.current_hit_counter <= self.max_hits_in_interval {
            return false;
        }
        if now == 0 {
            now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
        }
        if counter.locked_until > 0 {
            if counter.locked_until > now {
                return true;
            }
            //println!("reset all");
            counter.interval_start = now;
            counter.locked_until = 0;
            counter.current_hit_counter = 1;
            return false;
        }
        if now - counter.interval_start <= self.interval_duration {
            counter.interval_start = now;
            counter.current_hit_counter = 1;
            counter.locked_until = now + self.lockout_duration;
            return true;
        }
        counter.interval_start = now;
        counter.current_hit_counter = 1;
        return false;
    }
}

#[cfg(test)]
mod tests {
    use crate::Throttle;
    use crate::ThrottleHash;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_throttle() {
        let mut t = Throttle::new(500, 3, 1000);

        // Slow and study shouldnt lock
        assert!(!t.is_throttled());
        thread::sleep(Duration::from_millis(600));
        assert!(!t.is_throttled());
        thread::sleep(Duration::from_millis(600));
        assert!(!t.is_throttled());
        thread::sleep(Duration::from_millis(600));
        assert!(!t.is_throttled());
        thread::sleep(Duration::from_millis(600));
        assert!(!t.is_throttled());
        thread::sleep(Duration::from_millis(600));
        assert!(!t.is_throttled());
        assert!(!t.is_throttled());
        assert!(!t.is_throttled());
        assert!(!t.is_throttled());

        assert!(t.is_throttled()); // Trigger and stay triggered for the lockout time
        thread::sleep(Duration::from_millis(300));
        assert!(t.is_throttled());
        thread::sleep(Duration::from_millis(300));
        assert!(t.is_throttled());
        thread::sleep(Duration::from_millis(500));
        assert!(!t.is_throttled());

        // Check the throttle still works after the last clear
        assert!(!t.is_throttled());
        assert!(!t.is_throttled());
        assert!(t.is_throttled());
        thread::sleep(Duration::from_millis(1100));
        assert!(!t.is_throttled());
    }

    #[test]
    fn test_throttle_key() {
        let mut t = ThrottleHash::<String>::new(500, 3, 1000);

        let email1 = "bob1@example.com".to_string();

        // Slow and study shouldnt lock
        assert!(!t.is_throttled(&email1));
        thread::sleep(Duration::from_millis(600));
        assert!(!t.is_throttled(&email1));
        thread::sleep(Duration::from_millis(600));
        assert!(!t.is_throttled(&email1));
        thread::sleep(Duration::from_millis(600));
        assert!(!t.is_throttled(&email1));
        thread::sleep(Duration::from_millis(600));
        assert!(!t.is_throttled(&email1));
        thread::sleep(Duration::from_millis(600));
        assert!(!t.is_throttled(&email1));
        assert!(!t.is_throttled(&email1));
        assert!(!t.is_throttled(&email1));
        assert!(!t.is_throttled(&email1));

        assert!(t.is_throttled(&email1));
        thread::sleep(Duration::from_millis(300));
        assert!(t.is_throttled(&email1));
        thread::sleep(Duration::from_millis(300));
        assert!(t.is_throttled(&email1));
        thread::sleep(Duration::from_millis(500));
        assert!(!t.is_throttled(&email1));

        // Check the throttle still works after the last clear
        assert!(!t.is_throttled(&email1));
        assert!(!t.is_throttled(&email1));
        assert!(t.is_throttled(&email1));
        thread::sleep(Duration::from_millis(1100));
        assert!(!t.is_throttled(&email1));
    }

    #[test]
    fn test_throttle_key_overlap() {
        let mut t = ThrottleHash::<String>::new(500, 3, 1000);

        let email1 = "bob1@example.com".to_string();
        let email2 = "bob2@example.com".to_string();

        // Slow and study shouldnt lock
        assert!(!t.is_throttled(&email1));
        assert!(!t.is_throttled(&email2));
        thread::sleep(Duration::from_millis(600));
        assert!(!t.is_throttled(&email1));
        assert!(!t.is_throttled(&email2));
        thread::sleep(Duration::from_millis(600));
        assert!(!t.is_throttled(&email1));
        assert!(!t.is_throttled(&email2));
        thread::sleep(Duration::from_millis(600));
        assert!(!t.is_throttled(&email1));
        assert!(!t.is_throttled(&email2));
        thread::sleep(Duration::from_millis(600));
        assert!(!t.is_throttled(&email1));
        assert!(!t.is_throttled(&email2));
        thread::sleep(Duration::from_millis(600));
        assert!(!t.is_throttled(&email1));
        assert!(!t.is_throttled(&email2));
        assert!(!t.is_throttled(&email1));
        assert!(!t.is_throttled(&email2));
        assert!(!t.is_throttled(&email1));
        assert!(!t.is_throttled(&email2));
        assert!(!t.is_throttled(&email1));
        assert!(!t.is_throttled(&email2));

        assert!(t.is_throttled(&email1));
        assert!(t.is_throttled(&email2));
        thread::sleep(Duration::from_millis(300));
        assert!(t.is_throttled(&email1));
        assert!(t.is_throttled(&email2));
        thread::sleep(Duration::from_millis(300));
        assert!(t.is_throttled(&email1));
        assert!(t.is_throttled(&email2));
        thread::sleep(Duration::from_millis(500));
        assert!(!t.is_throttled(&email1));
        assert!(!t.is_throttled(&email2));

        // Check the throttle still works after the last clear
        assert!(!t.is_throttled(&email1));
        assert!(!t.is_throttled(&email2));
        assert!(!t.is_throttled(&email1));
        assert!(!t.is_throttled(&email2));
        assert!(t.is_throttled(&email1));
        assert!(t.is_throttled(&email2));
        thread::sleep(Duration::from_millis(1100));
        assert!(!t.is_throttled(&email1));
        assert!(!t.is_throttled(&email2));
    }
}
