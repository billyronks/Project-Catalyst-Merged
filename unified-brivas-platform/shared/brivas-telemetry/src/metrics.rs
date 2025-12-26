//! Metrics primitives

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Simple counter metric
#[derive(Clone, Default)]
pub struct Counter {
    value: Arc<AtomicU64>,
    name: String,
}

impl Counter {
    pub fn new(name: &str) -> Self {
        Self {
            value: Arc::new(AtomicU64::new(0)),
            name: name.to_string(),
        }
    }

    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add(&self, n: u64) {
        self.value.fetch_add(n, Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Simple gauge metric
#[derive(Clone, Default)]
pub struct Gauge {
    value: Arc<AtomicU64>,
    name: String,
}

impl Gauge {
    pub fn new(name: &str) -> Self {
        Self {
            value: Arc::new(AtomicU64::new(0)),
            name: name.to_string(),
        }
    }

    pub fn set(&self, val: u64) {
        self.value.store(val, Ordering::Relaxed);
    }

    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec(&self) {
        self.value.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Simple histogram metric (stores samples for percentile calculation)
#[derive(Clone)]
pub struct Histogram {
    samples: Arc<parking_lot::Mutex<Vec<f64>>>,
    name: String,
    max_samples: usize,
}

impl Histogram {
    pub fn new(name: &str) -> Self {
        Self {
            samples: Arc::new(parking_lot::Mutex::new(Vec::with_capacity(1000))),
            name: name.to_string(),
            max_samples: 10000,
        }
    }

    pub fn record(&self, value: f64) {
        let mut samples = self.samples.lock();
        if samples.len() >= self.max_samples {
            samples.remove(0);
        }
        samples.push(value);
    }

    pub fn percentile(&self, p: f64) -> f64 {
        let mut samples = self.samples.lock();
        if samples.is_empty() {
            return 0.0;
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((samples.len() as f64) * p / 100.0) as usize;
        samples[idx.min(samples.len() - 1)]
    }

    pub fn mean(&self) -> f64 {
        let samples = self.samples.lock();
        if samples.is_empty() {
            return 0.0;
        }
        samples.iter().sum::<f64>() / samples.len() as f64
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let counter = Counter::new("test_counter");
        assert_eq!(counter.get(), 0);
        counter.inc();
        assert_eq!(counter.get(), 1);
        counter.add(5);
        assert_eq!(counter.get(), 6);
    }

    #[test]
    fn test_gauge() {
        let gauge = Gauge::new("test_gauge");
        gauge.set(10);
        assert_eq!(gauge.get(), 10);
        gauge.inc();
        assert_eq!(gauge.get(), 11);
        gauge.dec();
        assert_eq!(gauge.get(), 10);
    }

    #[test]
    fn test_histogram() {
        let hist = Histogram::new("test_histogram");
        hist.record(1.0);
        hist.record(2.0);
        hist.record(3.0);
        hist.record(4.0);
        hist.record(5.0);
        
        assert!((hist.mean() - 3.0).abs() < 0.001);
        assert!((hist.percentile(50.0) - 3.0).abs() < 0.001);
    }
}
