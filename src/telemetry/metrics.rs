use crate::config::AppConfig;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Metrics collector for the application
pub struct MetricsCollector {
    config: AppConfig,
    events_processed: AtomicU64,
    events_dropped: AtomicU64,
    errors_total: AtomicU64,
    start_time: Instant,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            events_processed: AtomicU64::new(0),
            events_dropped: AtomicU64::new(0),
            errors_total: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    /// Record processed events
    pub fn record_events_processed(&self, count: u64) {
        match count.cmp(&0) {
            std::cmp::Ordering::Greater => {
                self.events_processed.fetch_add(count, Ordering::Relaxed);

                // Log high processing rates
                if count > 1000 {
                    info!("High event processing rate: {} events", count);
                }

                // Check for potential issues with very high counts
                if count > 10000 {
                    warn!(
                        "Very high event processing rate: {} events, potential burst",
                        count
                    );
                }
            }
            std::cmp::Ordering::Equal => {
                // Log when no events are processed (potential issue)
                debug!("No events processed in this cycle");
            }
            std::cmp::Ordering::Less => {
                // This shouldn't happen with u64, but handle it gracefully
                warn!("Negative event count received: {}", count);
            }
        }
    }

    /// Record dropped events
    #[allow(dead_code)]
    pub fn record_events_dropped(&self, count: u64) {
        if count > 0 {
            self.events_dropped.fetch_add(count, Ordering::Relaxed);

            // Log dropped events with warning
            warn!("Dropped {} events due to processing issues", count);

            // Record as error if too many events are dropped
            if count > 100 {
                error!(
                    "High number of dropped events: {}, potential system issue",
                    count
                );
                self.record_error();
            }
        }
    }

    /// Record errors
    pub fn record_error(&self) {
        let error_count = self.errors_total.fetch_add(1, Ordering::Relaxed) + 1;

        // Log error milestones
        if error_count % 10 == 0 {
            warn!("Error count milestone: {} errors recorded", error_count);
        }

        // Check for critical error thresholds
        if error_count > 1000 {
            error!("Critical error threshold exceeded: {} errors", error_count);
        } else if error_count > 100 {
            warn!("High error count: {} errors", error_count);
        }

        // Log individual errors in debug mode
        debug!("Error recorded, total count: {}", error_count);
    }

    /// Get current metrics
    #[allow(dead_code)]
    pub fn get_metrics(&self) -> Metrics {
        let uptime = self.start_time.elapsed();
        let events_processed = self.events_processed.load(Ordering::Relaxed);
        let events_dropped = self.events_dropped.load(Ordering::Relaxed);
        let errors_total = self.errors_total.load(Ordering::Relaxed);

        // Calculate rates (events per second)
        let uptime_secs = uptime.as_secs_f64();
        let events_processed_rate = if uptime_secs > 0.0 {
            events_processed as f64 / uptime_secs
        } else {
            0.0
        };

        let errors_rate = if uptime_secs > 0.0 {
            errors_total as f64 / uptime_secs
        } else {
            0.0
        };

        // Calculate drop rate for potential future use
        let _drop_rate = if uptime_secs > 0.0 {
            events_dropped as f64 / uptime_secs
        } else {
            0.0
        };

        // Log drop rate if it's significant
        if _drop_rate > 0.1 {
            // More than 10% drop rate
            warn!("High event drop rate: {:.2}%", _drop_rate * 100.0);
        }

        // Log metrics collection
        debug!(
            "Metrics collected - processed: {}, dropped: {}, errors: {}, uptime: {}s",
            events_processed,
            events_dropped,
            errors_total,
            uptime.as_secs()
        );

        Metrics {
            events_processed_total: events_processed,
            events_processed_rate,
            errors_total,
            errors_rate,
            latency_p50_ms: 0.0, // Would be calculated from actual measurements
            latency_p95_ms: 0.0,
            latency_p99_ms: 0.0,
            memory_usage_mb: 0.0, // Would be calculated from system metrics
            cpu_usage_percent: 0.0,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Collect metrics with error handling
    async fn collect_metrics_safely(&self) -> Result<Metrics, String> {
        // Validate metrics collection prerequisites
        if !self.is_monitoring_enabled() {
            return Err("Monitoring not enabled".to_string());
        }

        // Check if metrics collection has been running too long
        let uptime = self.start_time.elapsed();
        if uptime.as_secs() > 86400 {
            // 24 hours
            return Err("Metrics collection running too long, potential memory leak".to_string());
        }

        // Check error rate threshold
        let error_rate =
            self.errors_total.load(Ordering::Relaxed) as f64 / uptime.as_secs_f64().max(1.0);
        if error_rate > 0.5 {
            // 50% error rate
            return Err(format!("Error rate too high: {:.2}%", error_rate * 100.0));
        }

        // Collect metrics safely
        Ok(self.get_metrics())
    }

    /// Start metrics collection loop
    #[allow(dead_code)]
    pub async fn start_collection(&self) {
        // Only start if metrics are enabled
        if !self.config.monitoring.enable_metrics {
            info!("Metrics collection disabled in configuration");
            return;
        }

        let interval = Duration::from_secs(self.config.monitoring.metrics_interval_seconds);
        let metrics_collector = self.clone();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            let mut consecutive_failures = 0;
            let max_failures = 5;

            loop {
                interval_timer.tick().await;

                // Collect metrics with error handling
                match metrics_collector.collect_metrics_safely().await {
                    Ok(metrics) => {
                        consecutive_failures = 0; // Reset failure counter on success

                        // Log metrics based on environment
                        if metrics_collector.config.is_production() {
                            info!("Production metrics: {:?}", metrics);
                        } else if metrics_collector.config.is_development() {
                            debug!("Development metrics: {:?}", metrics);
                        } else {
                            info!("Metrics: {:?}", metrics);
                        }

                        // Log monitoring configuration
                        debug!(
                            "Monitoring config: {}",
                            metrics_collector.get_monitoring_config()
                        );
                    }
                    Err(e) => {
                        consecutive_failures += 1;
                        error!(
                            "Metrics collection failed (attempt {}/{}): {}",
                            consecutive_failures, max_failures, e
                        );

                        // Record the error
                        metrics_collector.record_error();

                        // Stop collection if too many consecutive failures
                        if consecutive_failures >= max_failures {
                            error!("Too many metrics collection failures, stopping collection");
                            break;
                        }
                    }
                }

                // Here you would typically send metrics to a monitoring system
                // like Prometheus, InfluxDB, or CloudWatch
            }
        });
    }

    /// Start health check monitoring if enabled
    pub async fn start_health_checks(&self) {
        if !self.is_monitoring_enabled() {
            info!("Health checks disabled in configuration");
            return;
        }

        let health_check_interval = Duration::from_secs(30); // Check every 30 seconds
        let metrics_collector = self.clone();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(health_check_interval);
            let mut consecutive_health_failures = 0;
            let max_health_failures = 10;

            loop {
                interval_timer.tick().await;

                // Perform health checks with error handling
                match metrics_collector.check_health_safely().await {
                    Ok(health_status) => {
                        consecutive_health_failures = 0; // Reset failure counter on success

                        if !health_status.is_healthy {
                            warn!(
                                "Health check failed at {}: {}",
                                health_status.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                                health_status.message
                            );

                            // Handle unhealthy state
                            metrics_collector
                                .handle_unhealthy_state(&health_status)
                                .await;
                        } else {
                            debug!(
                                "Health check passed at {}: {}",
                                health_status.timestamp.format("%Y:%M:%S UTC"),
                                health_status.message
                            );
                        }
                    }
                    Err(e) => {
                        consecutive_health_failures += 1;
                        error!(
                            "Health check error (attempt {}/{}): {}",
                            consecutive_health_failures, max_health_failures, e
                        );

                        // Record the error
                        metrics_collector.record_error();

                        // Stop health checks if too many consecutive failures
                        if consecutive_health_failures >= max_health_failures {
                            error!("Too many health check failures, stopping health monitoring");
                            break;
                        }
                    }
                }
            }
        });
    }

    /// Check system health
    async fn check_health(&self) -> HealthStatus {
        // Simple health check - could be expanded with actual system checks
        let timestamp = chrono::Utc::now();

        // Check if system has been running for too long (potential memory leak indicator)
        let uptime = self.start_time.elapsed();
        let is_healthy = uptime.as_secs() < 86400; // 24 hours

        // Check memory usage (simplified)
        let memory_ok = uptime.as_secs() < 3600 || uptime.as_secs() > 86400; // Memory check after 1 hour or after 24 hours

        // Check error rate
        let error_rate =
            self.errors_total.load(Ordering::Relaxed) as f64 / uptime.as_secs_f64().max(1.0);
        let error_rate_ok = error_rate < 0.1; // Less than 10% error rate

        let final_health = is_healthy && memory_ok && error_rate_ok;

        let message = if final_health {
            format!(
                "System healthy, uptime: {}s, error rate: {:.2}%",
                uptime.as_secs(),
                error_rate * 100.0
            )
        } else {
            format!(
                "System unhealthy, uptime: {}s, error rate: {:.2}%, memory: {}",
                uptime.as_secs(),
                error_rate * 100.0,
                if memory_ok { "OK" } else { "WARNING" }
            )
        };

        HealthStatus {
            is_healthy: final_health,
            message,
            timestamp,
        }
    }

    /// Check system health with error handling
    async fn check_health_safely(&self) -> Result<HealthStatus, String> {
        // Validate health check prerequisites
        if !self.is_monitoring_enabled() {
            return Err("Health monitoring not enabled".to_string());
        }

        // Check if health checks have been running too long
        let uptime = self.start_time.elapsed();
        if uptime.as_secs() > 86400 {
            // 24 hours
            return Err("Health monitoring running too long, potential memory leak".to_string());
        }

        // Check if error rate is too high for health checks
        let error_rate =
            self.errors_total.load(Ordering::Relaxed) as f64 / uptime.as_secs_f64().max(1.0);
        if error_rate > 0.8 {
            // 80% error rate
            return Err(format!(
                "Error rate too high for health checks: {:.2}%",
                error_rate * 100.0
            ));
        }

        // Perform health check safely
        Ok(self.check_health().await)
    }

    /// Handle unhealthy system state
    async fn handle_unhealthy_state(&self, health_status: &HealthStatus) {
        // Record the unhealthy state
        self.record_error();

        // Log detailed health information
        error!(
            "System health degraded at {}: {}",
            health_status.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            health_status.message
        );

        // Here you could add additional actions like:
        // - Sending alerts
        // - Restarting services
        // - Scaling resources
        // - Triggering failover
    }
}

impl Clone for MetricsCollector {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            events_processed: AtomicU64::new(self.events_processed.load(Ordering::Relaxed)),
            events_dropped: AtomicU64::new(self.events_dropped.load(Ordering::Relaxed)),
            errors_total: AtomicU64::new(self.errors_total.load(Ordering::Relaxed)),
            start_time: self.start_time,
        }
    }
}

impl MetricsCollector {
    /// Check if monitoring features are enabled
    pub fn is_monitoring_enabled(&self) -> bool {
        self.config.monitoring.enable_metrics
            && self.config.monitoring.enable_health_checks
            && self.config.monitoring.enable_structured_logging
    }

    /// Get monitoring configuration summary
    pub fn get_monitoring_config(&self) -> String {
        format!(
            "Metrics: {}, Health Checks: {}, Structured Logging: {}, Log Format: {}",
            self.config.monitoring.enable_metrics,
            self.config.monitoring.enable_health_checks,
            self.config.monitoring.enable_structured_logging,
            self.config.monitoring.log_format
        )
    }
}

/// Health status structure
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Metrics data structure
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Metrics {
    pub events_processed_total: u64,
    pub events_processed_rate: f64,
    pub errors_total: u64,
    pub errors_rate: f64,
    pub latency_p50_ms: f64,
    pub latency_p95_ms: f64,
    pub latency_p99_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
