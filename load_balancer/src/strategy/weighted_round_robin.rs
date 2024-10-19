use std::sync::{Arc, Mutex};

use crate::consts;
use crate::endpoint::Endpoint;
use crate::strategy::context::StrategyContext;
use crate::strategy::RouteStrategy;

pub struct WeightedRoundRobin {
    idx: usize,
    curr_weight: i64,
    guard: Mutex<()>,
}

impl WeightedRoundRobin {
    pub fn new() -> Self {
        WeightedRoundRobin {
            idx: 0,
            curr_weight: 0,
            guard: Mutex::new(()),
        }
    }

    fn gcd(mut x: u8, mut y: u8) -> u8 {
        let mut t;
        loop {
            t = x % y;
            if t > 0 {
                x = y;
                y = t;
            } else {
                return y;
            }
        }
    }

    fn cal_gcd(endpoints: &Vec<Arc<Box<dyn Endpoint>>>) -> Option<u8> {
        let weights: Vec<u8> = endpoints
            .iter()
            .filter_map(|endpoint| endpoint.weight())
            .collect();

        if weights.is_empty() {
            return None;
        }

        let mut result = *weights.first().unwrap();

        if let Some(weights) = weights.get(1..) {
            for weight in weights {
                result = Self::gcd(result.clone(), weight.clone())
            }
        }

        Some(result)
    }

    fn max(endpoints: &Vec<Arc<Box<dyn Endpoint>>>) -> Option<u8> {
        endpoints
            .iter()
            .filter_map(|endpoint| endpoint.weight())
            .max()
    }
}

impl RouteStrategy for WeightedRoundRobin {
    fn name(&self) -> String {
        return String::from(consts::WEIGHTED_ROUND_ROBIN);
    }

    fn pick(&mut self, _ctx: &StrategyContext, endpoints: &Vec<Arc<Box<dyn Endpoint>>>) -> Option<Arc<Box<dyn Endpoint>>> {
        let guard = self.guard.lock();
        if guard.is_err() {
            return None;
        }
        let gcd = Self::cal_gcd(endpoints).unwrap_or_default();
        loop {
            let curr_idx = self.idx;
            self.idx = (self.idx + 1) % endpoints.len();
            if curr_idx == 0 {
                self.curr_weight -= gcd as i64;
                if self.curr_weight <= 0 {
                    self.curr_weight = Self::max(endpoints).unwrap_or_default() as i64;
                }
                if self.curr_weight == 0 {
                    return None;
                }
            }
            if let Some(endpoint) = endpoints.get(curr_idx) {
                if endpoint.weight().unwrap_or_default() >= self.curr_weight as u8 {
                    return Some(Arc::clone(endpoint));
                }
            }
        }
    }
}

#[cfg(test)]
mod round_robin_test {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use crate::consts::WEIGHTED_ROUND_ROBIN;
    use crate::endpoint::MockEndpoint;

    use super::*;

    struct TestData {
        name: String,
        endpoints: Vec<Arc<Box<dyn Endpoint>>>,
        addr_expectation: Vec<SocketAddr>,
    }

    #[test]
    fn test_name() {
        assert_eq!(WEIGHTED_ROUND_ROBIN, WeightedRoundRobin::new().name());
    }
    #[test]
    fn test_pick() {
        let mut endpoint1_1 = MockEndpoint::new();
        endpoint1_1.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080));
        endpoint1_1.expect_weight().returning(|| Some(5));
        let mut endpoint1_2 = MockEndpoint::new();
        endpoint1_2.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081));
        endpoint1_2.expect_weight().returning(|| Some(2));
        let mut endpoint1_3 = MockEndpoint::new();
        endpoint1_3.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8082));
        endpoint1_3.expect_weight().returning(|| Some(3));

        let dataset = vec![
            TestData {
                name: "start_from_0".to_string(),
                endpoints: vec![
                    Arc::new(Box::new(endpoint1_1)),
                    Arc::new(Box::new(endpoint1_2)),
                    Arc::new(Box::new(endpoint1_3)),
                ],
                addr_expectation: vec![
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8082),
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
                ],
            },
        ];
        for data in dataset {
            let mut weighted_round_robin = WeightedRoundRobin::new();
            for expectation in data.addr_expectation {
                let target = weighted_round_robin.pick(&StrategyContext::new(String::new()), &data.endpoints);
                assert_eq!(target.unwrap().addr(), expectation, "test set: {}", data.name)
            }
        }
    }
}