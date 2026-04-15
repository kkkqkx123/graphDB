use std::time::Duration;

pub fn calculate_cache_hit_rate(hits: u64, misses: u64) -> f64 {
    let total = hits + misses;
    if total > 0 {
        hits as f64 / total as f64
    } else {
        0.0
    }
}

pub trait CacheMetrics {
    fn cache_hits(&self) -> u64;
    fn cache_misses(&self) -> u64;

    fn cache_hit_rate(&self) -> f64 {
        calculate_cache_hit_rate(self.cache_hits(), self.cache_misses())
    }
}

pub fn calculate_average(total: f64, count: u64) -> f64 {
    if count == 0 {
        0.0
    } else {
        total / count as f64
    }
}

pub fn micros_to_millis(micros: u64) -> f64 {
    micros as f64 / 1000.0
}

pub fn duration_to_micros(duration: Duration) -> u64 {
    duration.as_micros() as u64
}

pub fn format_duration(micros: u64) -> String {
    if micros >= 1_000_000 {
        format!("{:.2}s", micros as f64 / 1_000_000.0)
    } else if micros >= 1_000 {
        format!("{:.2}ms", micros as f64 / 1_000.0)
    } else {
        format!("{}us", micros)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_cache_hit_rate() {
        assert_eq!(calculate_cache_hit_rate(90, 10), 0.9);
        assert_eq!(calculate_cache_hit_rate(0, 0), 0.0);
        assert_eq!(calculate_cache_hit_rate(100, 0), 1.0);
    }

    #[test]
    fn test_calculate_average() {
        assert_eq!(calculate_average(100.0, 10), 10.0);
        assert_eq!(calculate_average(0.0, 0), 0.0);
    }

    #[test]
    fn test_micros_to_millis() {
        assert_eq!(micros_to_millis(1000), 1.0);
        assert_eq!(micros_to_millis(500), 0.5);
        assert_eq!(micros_to_millis(0), 0.0);
    }

    #[test]
    fn test_duration_to_micros() {
        assert_eq!(duration_to_micros(Duration::from_micros(100)), 100);
        assert_eq!(duration_to_micros(Duration::from_millis(1)), 1000);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(500), "500us");
        assert_eq!(format_duration(1500), "1.50ms");
        assert_eq!(format_duration(1_500_000), "1.50s");
    }
}