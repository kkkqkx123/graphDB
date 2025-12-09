use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::fmt::Debug;

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

    pub fn inc(&self) {
        let mut val = self.value.lock().unwrap();
        *val += 1;
    }

    pub fn inc_by(&self, amount: u64) {
        let mut val = self.value.lock().unwrap();
        *val += amount;
    }

    pub fn get(&self) -> u64 {
        *self.value.lock().unwrap()
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

    pub fn set(&self, value: f64) {
        let mut val = self.value.lock().unwrap();
        *val = value;
    }

    pub fn get(&self) -> f64 {
        *self.value.lock().unwrap()
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

    pub fn observe(&self, value: f64) {
        let mut vals = self.value.lock().unwrap();
        vals.push(value);
        let mut sum = self.sum.lock().unwrap();
        *sum += value;

        // Update bucket counts
        let mut counts = self.counts.lock().unwrap();
        for (i, &bucket) in self.buckets.iter().enumerate() {
            if value <= bucket {
                counts[i] += 1;
            }
        }
    }

    pub fn get_summary(&self) -> (f64, f64, Vec<(f64, u64)>) { // (avg, sum, bucket_counts)
        let vals = self.value.lock().unwrap();
        let sum = *self.sum.lock().unwrap();
        let counts = self.counts.lock().unwrap();
        
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
            
        (avg, sum, bucket_counts)
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

    pub fn record(&self, duration: Duration) {
        let mut vals = self.value.lock().unwrap();
        vals.push(duration);
    }

    pub fn record_fn<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = std::time::Instant::now();
        let result = f();
        let duration = start.elapsed();
        self.record(duration);
        result
    }

    pub fn get_stats(&self) -> (Duration, Duration, Duration, usize) { // (avg, min, max, count)
        let vals = self.value.lock().unwrap();
        if vals.is_empty() {
            return (Duration::from_nanos(0), Duration::from_nanos(0), Duration::from_nanos(0), 0);
        }

        let sum = vals.iter().sum::<Duration>();
        let avg = Duration::from_nanos(sum.as_nanos() as u64 / vals.len() as u64);
        
        let min = *vals.iter().min().unwrap();
        let max = *vals.iter().max().unwrap();
        
        (avg, min, max, vals.len())
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

    pub fn register_counter(&self, name: &str, description: &str) -> Counter {
        let counter = Counter::new(name, description);
        let mut counters = self.counters.lock().unwrap();
        counters.insert(name.to_string(), counter.clone());
        counter
    }

    pub fn register_gauge(&self, name: &str, description: &str) -> Gauge {
        let gauge = Gauge::new(name, description);
        let mut gauges = self.gauges.lock().unwrap();
        gauges.insert(name.to_string(), gauge.clone());
        gauge
    }

    pub fn register_histogram(&self, name: &str, description: &str, buckets: Vec<f64>) -> Histogram {
        let histogram = Histogram::new(name, description, buckets);
        let mut histograms = self.histograms.lock().unwrap();
        histograms.insert(name.to_string(), histogram.clone());
        histogram
    }

    pub fn register_timer(&self, name: &str, description: &str) -> Timer {
        let timer = Timer::new(name, description);
        let mut timers = self.timers.lock().unwrap();
        timers.insert(name.to_string(), timer.clone());
        timer
    }

    pub fn get_counter(&self, name: &str) -> Option<Counter> {
        let counters = self.counters.lock().unwrap();
        counters.get(name).cloned()
    }

    pub fn get_gauge(&self, name: &str) -> Option<Gauge> {
        let gauges = self.gauges.lock().unwrap();
        gauges.get(name).cloned()
    }

    pub fn get_histogram(&self, name: &str) -> Option<Histogram> {
        let histograms = self.histograms.lock().unwrap();
        histograms.get(name).cloned()
    }

    pub fn get_timer(&self, name: &str) -> Option<Timer> {
        let timers = self.timers.lock().unwrap();
        timers.get(name).cloned()
    }

    pub fn snapshot(&self) -> StatsSnapshot {
        let counters = self.counters.lock().unwrap();
        let gauges = self.gauges.lock().unwrap();
        let histograms = self.histograms.lock().unwrap();
        let timers = self.timers.lock().unwrap();

        let counter_values: HashMap<String, u64> = counters
            .iter()
            .map(|(name, counter)| (name.clone(), counter.get()))
            .collect();

        let gauge_values: HashMap<String, f64> = gauges
            .iter()
            .map(|(name, gauge)| (name.clone(), gauge.get()))
            .collect();

        let histogram_values: HashMap<String, (f64, f64, Vec<(f64, u64)>)> = histograms
            .iter()
            .map(|(name, histogram)| (name.clone(), histogram.get_summary()))
            .collect();

        let timer_values: HashMap<String, (Duration, Duration, Duration, usize)> = timers
            .iter()
            .map(|(name, timer)| (name.clone(), timer.get_stats()))
            .collect();

        StatsSnapshot {
            counters: counter_values,
            gauges: gauge_values,
            histograms: histogram_values,
            timers: timer_values,
            snapshot_time: SystemTime::now(),
        }
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
        println!("Snapshot time: {:?}", self.snapshot_time.duration_since(UNIX_EPOCH).unwrap().as_secs());
        
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
                println!("  {}: avg={:?}, min={:?}, max={:?}, count={}", name, avg, min, max, count);
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
pub async fn time_async<F, Fut, T>(name: &str, f: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let timer = GLOBAL_REGISTRY.register_timer(name, "Timer for async function");
    let start = std::time::Instant::now();
    let result = f().await;
    let duration = start.elapsed();
    timer.record(duration);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[test]
    fn test_counter() {
        let counter = Counter::new("test_counter", "A test counter");
        counter.inc();
        counter.inc_by(5);
        assert_eq!(counter.get(), 6);
    }

    #[test]
    fn test_gauge() {
        let gauge = Gauge::new("test_gauge", "A test gauge");
        gauge.set(3.14);
        assert_eq!(gauge.get(), 3.14);
    }

    #[test]
    fn test_histogram() {
        let histogram = Histogram::new("test_histogram", "A test histogram", vec![1.0, 2.0, 5.0]);
        
        histogram.observe(0.5);
        histogram.observe(1.5);
        histogram.observe(4.0);
        
        let (avg, sum, bucket_counts) = histogram.get_summary();
        assert_eq!(sum, 6.0);  // 0.5 + 1.5 + 4.0
        assert!((avg - 2.0).abs() < f64::EPSILON);  // 6.0 / 3
        
        // Check bucket counts: <=1.0: 1, <=2.0: 2, <=5.0: 3
        assert_eq!(bucket_counts[0], (1.0, 1)); // <= 1.0: 1 observation
        assert_eq!(bucket_counts[1], (2.0, 2)); // <= 2.0: 2 observations
        assert_eq!(bucket_counts[2], (5.0, 3)); // <= 5.0: 3 observations
    }

    #[test]
    fn test_timer() {
        let timer = Timer::new("test_timer", "A test timer");
        timer.record(Duration::from_millis(100));
        timer.record(Duration::from_millis(200));
        
        let (avg, min, max, count) = timer.get_stats();
        assert_eq!(count, 2);
        assert_eq!(min, Duration::from_millis(100));
        assert_eq!(max, Duration::from_millis(200));
    }

    #[tokio::test]
    async fn test_stats_registry() {
        let registry = StatsRegistry::new();
        
        let counter = registry.register_counter("req_count", "Request count");
        let gauge = registry.register_gauge("mem_usage", "Memory usage");
        let histogram = registry.register_histogram("query_time", "Query execution time", vec![1.0, 5.0, 10.0]);
        let timer = registry.register_timer("proc_time", "Processing time");
        
        counter.inc_by(5);
        gauge.set(25.6);
        histogram.observe(3.5);
        timer.record(Duration::from_millis(150));
        
        // Test getting the registered stats
        assert_eq!(registry.get_counter("req_count").unwrap().get(), 5);
        assert!((registry.get_gauge("mem_usage").unwrap().get() - 25.6).abs() < f64::EPSILON);
        
        let snapshot = registry.snapshot();
        assert_eq!(snapshot.counters.get("req_count"), Some(&5));
        assert_eq!(snapshot.gauges.get("mem_usage"), Some(&25.6));
        
        // Check the snapshot has the right values
        let hist_summary = snapshot.histograms.get("query_time").unwrap();
        assert_eq!(hist_summary.1, 3.5);  // sum
    }

    #[tokio::test]
    async fn test_global_registry() {
        let registry = global_registry();
        
        // Clean up in case of previous tests
        {
            let mut counters = registry.counters.lock().unwrap();
            counters.clear();
        }
        
        let counter = registry.register_counter("global_test", "Global test counter");
        counter.inc_by(10);
        
        assert_eq!(registry.get_counter("global_test").unwrap().get(), 10);
    }
}