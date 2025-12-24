use crate::core::error::DBError;
use crate::utils::{expect_max, expect_min, safe_lock};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Statistics counter for a specific metric
#[derive(Debug, Clone)]
pub struct Counter {
    pub name: String,
    pub description: String,
    value: Arc<Mutex<u64>>,
    
    created_at: std::time::SystemTime,
}

impl Counter {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            value: Arc::new(Mutex::new(0)),
            created_at: SystemTime::now(),
        }
    }

    pub fn inc(&self) -> Result<(), DBError> {
        let mut val = safe_lock(&self.value)?;
        *val += 1;
        Ok(())
    }

    pub fn inc_by(&self, amount: u64) -> Result<(), DBError> {
        let mut val = safe_lock(&self.value)?;
        *val += amount;
        Ok(())
    }

    pub fn get(&self) -> Result<u64, DBError> {
        let val = safe_lock(&self.value)?;
        Ok(*val)
    }
}

/// Statistics gauge for a specific metric
#[derive(Debug, Clone)]
pub struct Gauge {
    pub name: String,
    pub description: String,
    value: Arc<Mutex<f64>>,
    
    created_at: std::time::SystemTime,
}

impl Gauge {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            value: Arc::new(Mutex::new(0.0)),
            created_at: SystemTime::now(),
        }
    }

    pub fn set(&self, value: f64) -> Result<(), DBError> {
        let mut val = safe_lock(&self.value)?;
        *val = value;
        Ok(())
    }

    pub fn get(&self) -> Result<f64, DBError> {
        let val = safe_lock(&self.value)?;
        Ok(*val)
    }
}

/// Histogram for measuring value distribution
#[derive(Debug, Clone)]
pub struct Histogram {
    pub name: String,
    pub description: String,
    value: Arc<Mutex<Vec<f64>>>,
    buckets: Vec<f64>,
    counts: Arc<Mutex<Vec<u64>>>,
    sum: Arc<Mutex<f64>>,
    
    created_at: std::time::SystemTime,
}

impl Histogram {
    pub fn new(name: &str, description: &str, buckets: Vec<f64>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            value: Arc::new(Mutex::new(Vec::new())),
            buckets: buckets.clone(),
            counts: Arc::new(Mutex::new(vec![0; buckets.len()])),
            sum: Arc::new(Mutex::new(0.0)),
            created_at: SystemTime::now(),
        }
    }

    pub fn observe(&self, value: f64) -> Result<(), DBError> {
        let mut vals = safe_lock(&self.value)?;
        vals.push(value);
        let mut sum = safe_lock(&self.sum)?;
        *sum += value;

        // Update bucket counts
        let mut counts = safe_lock(&self.counts)?;
        for (i, &bucket) in self.buckets.iter().enumerate() {
            if value <= bucket {
                counts[i] += 1;
            }
        }
        Ok(())
    }

    pub fn get_summary(&self) -> Result<(f64, f64, Vec<(f64, u64)>), DBError> {
        // (avg, sum, bucket_counts)
        let vals = safe_lock(&self.value)?;
        let sum = *safe_lock(&self.sum)?;
        let counts = safe_lock(&self.counts)?;

        let avg = if vals.len() > 0 {
            sum / vals.len() as f64
        } else {
            0.0
        };

        let bucket_counts: Vec<(f64, u64)> = self
            .buckets
            .iter()
            .zip(counts.iter())
            .map(|(bucket, &count)| (*bucket, count))
            .collect();

        Ok((avg, sum, bucket_counts))
    }
}

/// Timer for measuring execution time
#[derive(Debug, Clone)]
pub struct Timer {
    pub name: String,
    pub description: String,
    value: Arc<Mutex<Vec<Duration>>>,
    
    created_at: std::time::SystemTime,
}

impl Timer {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            value: Arc::new(Mutex::new(Vec::new())),
            created_at: SystemTime::now(),
        }
    }

    pub fn record(&self, duration: Duration) -> Result<(), DBError> {
        let mut vals = safe_lock(&self.value)?;
        vals.push(duration);
        Ok(())
    }

    pub fn record_fn<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = std::time::Instant::now();
        let result = f();
        let duration = start.elapsed();
        let _ = self.record(duration);
        result
    }

    pub fn get_stats(&self) -> Result<(Duration, Duration, Duration, usize), DBError> {
        // (avg, min, max, count)
        let vals = safe_lock(&self.value)?;
        if vals.is_empty() {
            return Ok((
                Duration::from_nanos(0),
                Duration::from_nanos(0),
                Duration::from_nanos(0),
                0,
            ));
        }

        let sum = vals.iter().sum::<Duration>();
        let avg = Duration::from_nanos(sum.as_nanos() as u64 / vals.len() as u64);

        let min = *expect_min(
            vals.iter(),
            "Timer values should not be empty when calculating min",
        )?;
        let max = *expect_max(
            vals.iter(),
            "Timer values should not be empty when calculating max",
        )?;

        Ok((avg, min, max, vals.len()))
    }
}

/// Registry for all statistics
#[derive(Debug, Clone)]
pub struct StatsRegistry {
    counters: Arc<Mutex<HashMap<String, Counter>>>,
    gauges: Arc<Mutex<HashMap<String, Gauge>>>,
    histograms: Arc<Mutex<HashMap<String, Histogram>>>,
    timers: Arc<Mutex<HashMap<String, Timer>>>,
    
    created_at: std::time::SystemTime,
}

impl StatsRegistry {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(Mutex::new(HashMap::new())),
            gauges: Arc::new(Mutex::new(HashMap::new())),
            histograms: Arc::new(Mutex::new(HashMap::new())),
            timers: Arc::new(Mutex::new(HashMap::new())),
            created_at: SystemTime::now(),
        }
    }

    pub fn register_counter(&self, name: &str, description: &str) -> Result<Counter, DBError> {
        let counter = Counter::new(name, description);
        let mut counters = safe_lock(&self.counters)?;
        counters.insert(name.to_string(), counter.clone());
        Ok(counter)
    }

    pub fn register_gauge(&self, name: &str, description: &str) -> Result<Gauge, DBError> {
        let gauge = Gauge::new(name, description);
        let mut gauges = safe_lock(&self.gauges)?;
        gauges.insert(name.to_string(), gauge.clone());
        Ok(gauge)
    }

    pub fn register_histogram(
        &self,
        name: &str,
        description: &str,
        buckets: Vec<f64>,
    ) -> Result<Histogram, DBError> {
        let histogram = Histogram::new(name, description, buckets);
        let mut histograms = safe_lock(&self.histograms)?;
        histograms.insert(name.to_string(), histogram.clone());
        Ok(histogram)
    }

    pub fn register_timer(&self, name: &str, description: &str) -> Result<Timer, DBError> {
        let timer = Timer::new(name, description);
        let mut timers = safe_lock(&self.timers)?;
        timers.insert(name.to_string(), timer.clone());
        Ok(timer)
    }

    pub fn get_counter(&self, name: &str) -> Result<Option<Counter>, DBError> {
        let counters = safe_lock(&self.counters)?;
        Ok(counters.get(name).cloned())
    }

    pub fn get_gauge(&self, name: &str) -> Result<Option<Gauge>, DBError> {
        let gauges = safe_lock(&self.gauges)?;
        Ok(gauges.get(name).cloned())
    }

    pub fn get_histogram(&self, name: &str) -> Result<Option<Histogram>, DBError> {
        let histograms = safe_lock(&self.histograms)?;
        Ok(histograms.get(name).cloned())
    }

    pub fn get_timer(&self, name: &str) -> Result<Option<Timer>, DBError> {
        let timers = safe_lock(&self.timers)?;
        Ok(timers.get(name).cloned())
    }

    pub fn snapshot(&self) -> Result<StatsSnapshot, DBError> {
        let counters = safe_lock(&self.counters)?;
        let gauges = safe_lock(&self.gauges)?;
        let histograms = safe_lock(&self.histograms)?;
        let timers = safe_lock(&self.timers)?;

        let counter_values: HashMap<String, u64> = counters
            .iter()
            .filter_map(|(name, counter)| counter.get().ok().map(|value| (name.clone(), value)))
            .collect();

        let gauge_values: HashMap<String, f64> = gauges
            .iter()
            .filter_map(|(name, gauge)| gauge.get().ok().map(|value| (name.clone(), value)))
            .collect();

        let histogram_values: HashMap<String, (f64, f64, Vec<(f64, u64)>)> = histograms
            .iter()
            .filter_map(|(name, histogram)| {
                histogram
                    .get_summary()
                    .ok()
                    .map(|summary| (name.clone(), summary))
            })
            .collect();

        let timer_values: HashMap<String, (Duration, Duration, Duration, usize)> = timers
            .iter()
            .filter_map(|(name, timer)| timer.get_stats().ok().map(|stats| (name.clone(), stats)))
            .collect();

        Ok(StatsSnapshot {
            counters: counter_values,
            gauges: gauge_values,
            histograms: histogram_values,
            timers: timer_values,
            snapshot_time: SystemTime::now(),
        })
    }
}

/// Snapshot of statistics at a point in time
#[derive(Debug)]
pub struct StatsSnapshot {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub histograms: HashMap<String, (f64, f64, Vec<(f64, u64)>)>, // (avg, sum, bucket_counts)
    pub timers: HashMap<String, (Duration, Duration, Duration, usize)>, // (avg, min, max, count)
    pub snapshot_time: SystemTime,
}

impl StatsSnapshot {
    pub fn print_summary(&self) {
        println!("=== Statistics Snapshot ===");
        println!(
            "Snapshot time: {:?}",
            self.snapshot_time
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs()
        );

        if !self.counters.is_empty() {
            println!("\nCounters:");
            for (name, value) in &self.counters {
                println!("  {}: {}", name, value);
            }
        }

        if !self.gauges.is_empty() {
            println!("\nGauges:");
            for (name, value) in &self.gauges {
                println!("  {}: {:.2}", name, value);
            }
        }

        if !self.histograms.is_empty() {
            println!("\nHistograms:");
            for (name, (avg, sum, bucket_counts)) in &self.histograms {
                println!("  {}: avg={:.2}, sum={:.2}", name, avg, sum);
                for (bucket, count) in bucket_counts {
                    println!("    <= {}: {}", bucket, count);
                }
            }
        }

        if !self.timers.is_empty() {
            println!("\nTimers:");
            for (name, (avg, min, max, count)) in &self.timers {
                println!(
                    "  {}: avg={:?}, min={:?}, max={:?}, count={}",
                    name, avg, min, max, count
                );
            }
        }
    }
}

/// Global statistics registry
static GLOBAL_REGISTRY: once_cell::sync::Lazy<StatsRegistry> =
    once_cell::sync::Lazy::new(StatsRegistry::new);

/// Get the global statistics registry
pub fn global_registry() -> &'static StatsRegistry {
    &GLOBAL_REGISTRY
}

/// A helper to time an async function
pub async fn time_async<F, Fut, T>(name: &str, f: F) -> Result<T, DBError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let timer = GLOBAL_REGISTRY.register_timer(name, "Timer for async function")?;
    let start = std::time::Instant::now();
    let result = f().await;
    let duration = start.elapsed();
    timer.record(duration)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[test]
    fn test_counter() {
        let counter = Counter::new("test_counter", "A test counter");
        counter.inc().expect("Counter increment should succeed");
        counter
            .inc_by(5)
            .expect("Counter increment by should succeed");
        assert_eq!(counter.get().expect("Counter get should succeed"), 6);
    }

    #[test]
    fn test_gauge() {
        let gauge = Gauge::new("test_gauge", "A test gauge");
        gauge.set(3.14).expect("Gauge set should succeed");
        assert_eq!(gauge.get().expect("Gauge get should succeed"), 3.14);
    }

    #[test]
    fn test_histogram() {
        let histogram = Histogram::new("test_histogram", "A test histogram", vec![1.0, 2.0, 5.0]);

        histogram
            .observe(0.5)
            .expect("Histogram observe should succeed");
        histogram
            .observe(1.5)
            .expect("Histogram observe should succeed");
        histogram
            .observe(4.0)
            .expect("Histogram observe should succeed");

        let (avg, sum, bucket_counts) = histogram
            .get_summary()
            .expect("Histogram get_summary should succeed");
        assert_eq!(sum, 6.0); // 0.5 + 1.5 + 4.0
        assert!((avg - 2.0).abs() < f64::EPSILON); // 6.0 / 3

        // Check bucket counts: <=1.0: 1, <=2.0: 2, <=5.0: 3
        assert_eq!(bucket_counts[0], (1.0, 1)); // <= 1.0: 1 observation
        assert_eq!(bucket_counts[1], (2.0, 2)); // <= 2.0: 2 observations
        assert_eq!(bucket_counts[2], (5.0, 3)); // <= 5.0: 3 observations
    }

    #[test]
    fn test_timer() {
        let timer = Timer::new("test_timer", "A test timer");
        timer
            .record(Duration::from_millis(100))
            .expect("Timer record should succeed");
        timer
            .record(Duration::from_millis(200))
            .expect("Timer record should succeed");

        let (_avg, min, max, count) = timer.get_stats().expect("Timer get_stats should succeed");
        assert_eq!(count, 2);
        assert_eq!(min, Duration::from_millis(100));
        assert_eq!(max, Duration::from_millis(200));
    }

    #[tokio::test]
    async fn test_stats_registry() {
        let registry = StatsRegistry::new();

        let counter = registry
            .register_counter("req_count", "Request count")
            .expect("Registry register_counter should succeed");
        let gauge = registry
            .register_gauge("mem_usage", "Memory usage")
            .expect("Registry register_gauge should succeed");
        let histogram = registry
            .register_histogram("query_time", "Query execution time", vec![1.0, 5.0, 10.0])
            .expect("Registry register_histogram should succeed");
        let timer = registry
            .register_timer("proc_time", "Processing time")
            .expect("Registry register_timer should succeed");

        counter
            .inc_by(5)
            .expect("Counter increment by should succeed");
        gauge.set(25.6).expect("Gauge set should succeed");
        histogram
            .observe(3.5)
            .expect("Histogram observe should succeed");
        timer
            .record(Duration::from_millis(150))
            .expect("Timer record should succeed");

        // Test getting the registered stats
        assert_eq!(
            registry
                .get_counter("req_count")
                .expect("Registry get_counter should succeed")
                .expect("Counter should exist")
                .get()
                .expect("Counter get should succeed"),
            5
        );
        assert!(
            (registry
                .get_gauge("mem_usage")
                .expect("Registry get_gauge should succeed")
                .expect("Gauge should exist")
                .get()
                .expect("Gauge get should succeed")
                - 25.6)
                .abs()
                < f64::EPSILON
        );

        let snapshot = registry
            .snapshot()
            .expect("Registry snapshot should succeed");
        assert_eq!(snapshot.counters.get("req_count"), Some(&5));
        assert_eq!(snapshot.gauges.get("mem_usage"), Some(&25.6));

        // Check the snapshot has the right values
        let hist_summary = snapshot
            .histograms
            .get("query_time")
            .expect("Histogram should exist in snapshot");
        assert_eq!(hist_summary.1, 3.5); // sum
    }

    #[tokio::test]
    async fn test_global_registry() {
        let registry = global_registry();

        // Clean up in case of previous tests
        {
            let mut counters = safe_lock(&registry.counters)
                .expect("Registry counters lock should not be poisoned");
            counters.clear();
        }

        let counter = registry
            .register_counter("global_test", "Global test counter")
            .expect("Registry register_counter should succeed");
        counter
            .inc_by(10)
            .expect("Counter increment by should succeed");

        assert_eq!(
            registry
                .get_counter("global_test")
                .expect("Registry get_counter should succeed")
                .expect("Counter should exist")
                .get()
                .expect("Counter get should succeed"),
            10
        );
    }
}
