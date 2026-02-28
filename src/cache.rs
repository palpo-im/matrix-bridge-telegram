use std::time::{Duration, Instant};

pub struct TimedCache<T> {
    data: Option<T>,
    expires_at: Option<Instant>,
    ttl: Duration,
}

impl<T> TimedCache<T> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            data: None,
            expires_at: None,
            ttl,
        }
    }

    pub fn get(&self) -> Option<&T> {
        if let Some(expires_at) = self.expires_at {
            if Instant::now() < expires_at {
                return self.data.as_ref();
            }
        }
        None
    }

    pub fn set(&mut self, value: T) {
        self.data = Some(value);
        self.expires_at = Some(Instant::now() + self.ttl);
    }

    pub fn invalidate(&mut self) {
        self.data = None;
        self.expires_at = None;
    }
}

impl<T: Clone> TimedCache<T> {
    pub fn get_or_update<F>(&mut self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        if let Some(ref data) = self.data {
            if let Some(expires_at) = self.expires_at {
                if Instant::now() < expires_at {
                    return data.clone();
                }
            }
        }
        let value = f();
        self.set(value.clone());
        value
    }
}
