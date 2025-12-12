//! Prometheus-compatible metrics for the bridge

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;
use parking_lot::RwLock;

/// Global metrics registry
static METRICS: OnceLock<Metrics> = OnceLock::new();

/// Get the global metrics instance
pub fn metrics() -> &'static Metrics {
    METRICS.get_or_init(Metrics::new)
}

/// Metrics collection for the bridge
pub struct Metrics {
    // Counters
    mqtt_messages_received: AtomicU64,
    mqtt_messages_sent: AtomicU64,
    zmq_messages_received: AtomicU64,
    zmq_messages_sent: AtomicU64,
    errors_total: AtomicU64,
    
    // Latency tracking (simplified histogram using buckets)
    latency_samples: RwLock<Vec<f64>>,
    
    // Start time for uptime calculation
    start_time: Instant,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            mqtt_messages_received: AtomicU64::new(0),
            mqtt_messages_sent: AtomicU64::new(0),
            zmq_messages_received: AtomicU64::new(0),
            zmq_messages_sent: AtomicU64::new(0),
            errors_total: AtomicU64::new(0),
            latency_samples: RwLock::new(Vec::with_capacity(1000)),
            start_time: Instant::now(),
        }
    }

    /// Record MQTT message received
    pub fn record_mqtt_received(&self) {
        self.mqtt_messages_received.fetch_add(1, Ordering::Relaxed);
    }

    /// Record MQTT message sent
    pub fn record_mqtt_sent(&self) {
        self.mqtt_messages_sent.fetch_add(1, Ordering::Relaxed);
    }

    /// Record ZMQ message received
    pub fn record_zmq_received(&self) {
        self.zmq_messages_received.fetch_add(1, Ordering::Relaxed);
    }

    /// Record ZMQ message sent
    pub fn record_zmq_sent(&self) {
        self.zmq_messages_sent.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an error
    pub fn record_error(&self) {
        self.errors_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Record message forwarding latency in milliseconds
    pub fn record_latency(&self, latency_ms: f64) {
        let mut samples = self.latency_samples.write();
        // Keep last 1000 samples for histogram
        if samples.len() >= 1000 {
            samples.remove(0);
        }
        samples.push(latency_ms);
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// Get total messages forwarded
    pub fn total_forwarded(&self) -> u64 {
        self.mqtt_messages_sent.load(Ordering::Relaxed) + 
        self.zmq_messages_sent.load(Ordering::Relaxed)
    }

    /// Generate Prometheus-compatible metrics output
    pub fn render_prometheus(&self) -> String {
        let mqtt_rx = self.mqtt_messages_received.load(Ordering::Relaxed);
        let mqtt_tx = self.mqtt_messages_sent.load(Ordering::Relaxed);
        let zmq_rx = self.zmq_messages_received.load(Ordering::Relaxed);
        let zmq_tx = self.zmq_messages_sent.load(Ordering::Relaxed);
        let errors = self.errors_total.load(Ordering::Relaxed);
        let uptime = self.uptime_seconds();

        // Calculate latency percentiles
        let samples = self.latency_samples.read();
        let (p50, p95, p99) = if samples.is_empty() {
            (0.0, 0.0, 0.0)
        } else {
            let mut sorted: Vec<f64> = samples.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let len = sorted.len();
            let p50 = sorted[len * 50 / 100];
            let p95 = sorted[len * 95 / 100];
            let p99 = sorted.get(len * 99 / 100).copied().unwrap_or(sorted[len - 1]);
            (p50, p95, p99)
        };

        format!(
r#"# HELP zeromqtt_mqtt_messages_received_total Total MQTT messages received
# TYPE zeromqtt_mqtt_messages_received_total counter
zeromqtt_mqtt_messages_received_total {}

# HELP zeromqtt_mqtt_messages_sent_total Total MQTT messages sent
# TYPE zeromqtt_mqtt_messages_sent_total counter
zeromqtt_mqtt_messages_sent_total {}

# HELP zeromqtt_zmq_messages_received_total Total ZeroMQ messages received
# TYPE zeromqtt_zmq_messages_received_total counter
zeromqtt_zmq_messages_received_total {}

# HELP zeromqtt_zmq_messages_sent_total Total ZeroMQ messages sent
# TYPE zeromqtt_zmq_messages_sent_total counter
zeromqtt_zmq_messages_sent_total {}

# HELP zeromqtt_errors_total Total errors encountered
# TYPE zeromqtt_errors_total counter
zeromqtt_errors_total {}

# HELP zeromqtt_uptime_seconds Uptime in seconds
# TYPE zeromqtt_uptime_seconds gauge
zeromqtt_uptime_seconds {:.2}

# HELP zeromqtt_messages_forwarded_total Total messages forwarded
# TYPE zeromqtt_messages_forwarded_total counter
zeromqtt_messages_forwarded_total {}

# HELP zeromqtt_latency_milliseconds Message forwarding latency
# TYPE zeromqtt_latency_milliseconds summary
zeromqtt_latency_milliseconds{{quantile="0.5"}} {:.3}
zeromqtt_latency_milliseconds{{quantile="0.95"}} {:.3}
zeromqtt_latency_milliseconds{{quantile="0.99"}} {:.3}
"#,
            mqtt_rx, mqtt_tx, zmq_rx, zmq_tx, errors, uptime, 
            mqtt_tx + zmq_tx, p50, p95, p99
        )
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_counters() {
        let m = Metrics::new();
        m.record_mqtt_received();
        m.record_mqtt_received();
        m.record_mqtt_sent();
        
        assert_eq!(m.mqtt_messages_received.load(Ordering::Relaxed), 2);
        assert_eq!(m.mqtt_messages_sent.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_prometheus_output() {
        let m = Metrics::new();
        m.record_mqtt_sent();
        m.record_zmq_sent();
        
        let output = m.render_prometheus();
        assert!(output.contains("zeromqtt_mqtt_messages_sent_total 1"));
        assert!(output.contains("zeromqtt_zmq_messages_sent_total 1"));
    }
}
