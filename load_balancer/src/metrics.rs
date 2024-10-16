use lazy_static::lazy_static;
use prometheus::{HistogramTimer, register_histogram_vec, register_int_counter_vec};
use prometheus::{HistogramVec, IntCounterVec};

use crate::consts::{COUNTER_LATENCY, COUNTER_QUERY};

lazy_static! {
    static ref QUERY_COUNTER_VEC: IntCounterVec =
        register_int_counter_vec!(COUNTER_QUERY, "query count", &["server_name", "handler", "success"]).unwrap();
    static ref LATENCY_COUNTER_VEC: HistogramVec =
        register_histogram_vec!(COUNTER_LATENCY, "server latency", &["server_name", "handler", "success"]).unwrap();
}



pub struct QueryCounter {
    query_success: bool,
    server_name: String,
    handler: String,
    success_timer: Option<HistogramTimer>,
    failed_timer: Option<HistogramTimer>,
}

impl QueryCounter {
    pub fn new(server_name: &str, handler: &str) -> Self {
        Self {
            query_success: false,
            server_name: String::from(server_name),
            handler: String::from(handler),
            success_timer: Some(LATENCY_COUNTER_VEC.with_label_values(&[&server_name, &handler, "true"]).start_timer()),
            failed_timer: Some(LATENCY_COUNTER_VEC.with_label_values(&[&server_name, &handler, "false"]).start_timer()),
        }
    }

    pub fn mark_success(&mut self) {
        self.query_success = true;
    }
}

impl Drop for QueryCounter {
    fn drop(&mut self) {
        if self.query_success {
            QUERY_COUNTER_VEC.with_label_values(&[&self.server_name, &self.handler, "true"]).inc();
            self.success_timer.take().unwrap().observe_duration();
            self.failed_timer.take().unwrap().stop_and_discard();
        } else {
            QUERY_COUNTER_VEC.with_label_values(&[&self.server_name, &self.handler, "false"]).inc();
            self.failed_timer.take().unwrap().observe_duration();
            self.success_timer.take().unwrap().stop_and_discard();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::metrics::QueryCounter;

    use super::*;

    #[test]
    fn test_guard() {
        test_guard_query_success();
        test_guard_query_failed();
    }

    fn test_guard_query_failed() {
        QUERY_COUNTER_VEC.reset();
        LATENCY_COUNTER_VEC.reset();
        let guard = QueryCounter::new("server1", "HealthCheck");
        drop(guard);

        assert_eq!(QUERY_COUNTER_VEC.with_label_values(&["server1", "HealthCheck", "false"]).get(), 1);
        assert_eq!(QUERY_COUNTER_VEC.with_label_values(&["server1", "HealthCheck", "true"]).get(), 0);

        assert_eq!(LATENCY_COUNTER_VEC.with_label_values(&["server1", "HealthCheck", "false"]).get_sample_count(), 1);
        assert_eq!(LATENCY_COUNTER_VEC.with_label_values(&["server1", "HealthCheck", "true"]).get_sample_count(), 0);
    }

    fn test_guard_query_success() {
        QUERY_COUNTER_VEC.reset();
        LATENCY_COUNTER_VEC.reset();
        let mut guard = QueryCounter::new("server1", "HealthCheck");
        guard.mark_success();
        drop(guard);

        assert_eq!(QUERY_COUNTER_VEC.with_label_values(&["server1", "HealthCheck", "true"]).get(), 1);
        assert_eq!(QUERY_COUNTER_VEC.with_label_values(&["server1", "HealthCheck", "false"]).get(), 0);

        assert_eq!(LATENCY_COUNTER_VEC.with_label_values(&["server1", "HealthCheck", "true"]).get_sample_count(), 1);
        assert_eq!(LATENCY_COUNTER_VEC.with_label_values(&["server1", "HealthCheck", "false"]).get_sample_count(), 0);
    }
}