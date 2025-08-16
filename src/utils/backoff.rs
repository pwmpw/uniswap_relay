use std::time::Duration;

/// Exponential backoff strategy for retries
pub struct ExponentialBackoff {
    initial_delay: Duration,
    max_delay: Duration,
    multiplier: f64,
    max_attempts: u32,
    current_attempt: u32,
}

impl ExponentialBackoff {
    /// Create a new exponential backoff instance
    pub fn new(
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
        max_attempts: u32,
    ) -> Self {
        Self {
            initial_delay,
            max_delay,
            multiplier,
            max_attempts,
            current_attempt: 0,
        }
    }

    /// Get the next delay duration
    pub fn next_delay(&mut self) -> Option<Duration> {
        if self.current_attempt >= self.max_attempts {
            return None;
        }

        let delay = if self.current_attempt == 0 {
            self.initial_delay
        } else {
            let calculated_delay = self
                .initial_delay
                .mul_f64(self.multiplier.powi(self.current_attempt as i32));

            if calculated_delay > self.max_delay {
                self.max_delay
            } else {
                calculated_delay
            }
        };

        self.current_attempt += 1;
        Some(delay)
    }

    /// Reset the backoff counter
    pub fn reset(&mut self) {
        self.current_attempt = 0;
    }

    /// Get current attempt number
    pub fn current_attempt(&self) -> u32 {
        self.current_attempt
    }

    /// Check if max attempts reached
    pub fn exhausted(&self) -> bool {
        self.current_attempt >= self.max_attempts
    }
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self::new(
            Duration::from_millis(1000), // 1 second initial delay
            Duration::from_secs(30),     // 30 seconds max delay
            2.0,                         // Double the delay each time
            5,                           // Max 5 attempts
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff() {
        let mut backoff = ExponentialBackoff::new(
            Duration::from_millis(100),
            Duration::from_millis(1000),
            2.0,
            3,
        );

        // First attempt: 100ms
        assert_eq!(backoff.next_delay(), Some(Duration::from_millis(100)));
        assert_eq!(backoff.current_attempt(), 1);

        // Second attempt: 200ms
        assert_eq!(backoff.next_delay(), Some(Duration::from_millis(200)));
        assert_eq!(backoff.current_attempt(), 2);

        // Third attempt: 400ms
        assert_eq!(backoff.next_delay(), Some(Duration::from_millis(400)));
        assert_eq!(backoff.current_attempt(), 3);

        // Fourth attempt: should be None (max attempts reached)
        assert_eq!(backoff.next_delay(), None);
        assert!(backoff.exhausted());
    }

    #[test]
    fn test_backoff_reset() {
        let mut backoff = ExponentialBackoff::default();

        // Use it once
        backoff.next_delay();
        assert_eq!(backoff.current_attempt(), 1);

        // Reset it
        backoff.reset();
        assert_eq!(backoff.current_attempt(), 0);
        assert!(!backoff.exhausted());

        // Use it again
        backoff.next_delay();
        assert_eq!(backoff.current_attempt(), 1);
    }

    #[test]
    fn test_max_delay_cap() {
        let mut backoff = ExponentialBackoff::new(
            Duration::from_millis(1000),
            Duration::from_millis(2000), // Max 2 seconds
            3.0,                         // Triple each time
            5,
        );

        // First: 1000ms
        assert_eq!(backoff.next_delay(), Some(Duration::from_millis(1000)));

        // Second: 3000ms, but capped at 2000ms
        assert_eq!(backoff.next_delay(), Some(Duration::from_millis(2000)));

        // Third: 9000ms, but capped at 2000ms
        assert_eq!(backoff.next_delay(), Some(Duration::from_millis(2000)));
    }
}
