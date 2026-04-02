use std::sync::atomic::{AtomicU32, Ordering};

pub struct RateLimiter {
    remaining: AtomicU32,
    max: u32,
}

impl RateLimiter {
    pub fn new(rpm: u32) -> Self {
        Self {
            remaining: AtomicU32::new(rpm),
            max: rpm,
        }
    }

    pub fn try_acquire(&self) -> bool {
        loop {
            let current = self.remaining.load(Ordering::Relaxed);
            if current == 0 {
                return false;
            }
            if self
                .remaining
                .compare_exchange_weak(current, current - 1, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                return true;
            }
        }
    }

    pub fn replenish(&self, count: u32) {
        let current = self.remaining.load(Ordering::Relaxed);
        let new = (current + count).min(self.max);
        self.remaining.store(new, Ordering::Relaxed);
    }

    pub fn reset(&self) {
        self.remaining.store(self.max, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_up_to_limit() {
        let limiter = RateLimiter::new(3);
        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
    }

    #[test]
    fn rejects_over_limit() {
        let limiter = RateLimiter::new(2);
        limiter.try_acquire();
        limiter.try_acquire();
        assert!(!limiter.try_acquire());
    }

    #[test]
    fn replenishes_over_time() {
        let limiter = RateLimiter::new(2);
        limiter.try_acquire();
        limiter.try_acquire();
        assert!(!limiter.try_acquire());
        limiter.replenish(1);
        assert!(limiter.try_acquire());
    }

    #[test]
    fn reset_restores_full_capacity() {
        let limiter = RateLimiter::new(5);
        for _ in 0..5 {
            limiter.try_acquire();
        }
        assert!(!limiter.try_acquire());
        limiter.reset();
        assert!(limiter.try_acquire());
    }
}
