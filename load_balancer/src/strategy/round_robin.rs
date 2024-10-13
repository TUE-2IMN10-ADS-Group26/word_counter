use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::consts;
use crate::endpoint::Endpoint;
use crate::strategy::RouteStrategy;

#[derive(Default)]
pub struct RoundRobin {
    idx: AtomicUsize,
}

impl RoundRobin {
    pub fn new(idx: Option<AtomicUsize>) -> Self {
        RoundRobin {
            idx: idx.unwrap_or_default()
        }
    }

    #[allow(dead_code)]
    pub fn name() -> String {
        String::from(consts::ROUND_ROBIN)
    }
}

impl RouteStrategy for RoundRobin {
    fn name(&self) -> String {
        String::from(consts::ROUND_ROBIN)
    }
    fn pick(&mut self, endpoints: &Vec<Arc<Box<dyn Endpoint>>>) -> Option<Arc<Box<dyn Endpoint>>> {
        let curr_idx = self.idx.fetch_add(1, Ordering::SeqCst) % endpoints.len();
        endpoints.get(curr_idx).map(|endpoint| { Arc::clone(endpoint) })
    }
}

#[cfg(test)]
mod round_robin_test {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use crate::endpoint::MockEndpoint;

    use super::*;

    struct TestData {
        name: String,
        endpoints: Vec<Arc<Box<dyn Endpoint>>>,
        start_idx: Option<AtomicUsize>,
        addr_expectation: SocketAddr,
    }

    #[test]
    fn test_name() {
        assert_eq!(consts::ROUND_ROBIN, RoundRobin::name());
        assert_eq!(consts::ROUND_ROBIN, RoundRobin::new(None).name());
    }
    #[test]
    fn test_pick() {
        let mut endpoint1_1 = MockEndpoint::new();
        endpoint1_1.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080));
        let mut endpoint1_2 = MockEndpoint::new();
        endpoint1_2.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081));
        let mut endpoint1_3 = MockEndpoint::new();
        endpoint1_3.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8082));

        let mut endpoint2_1 = MockEndpoint::new();
        endpoint2_1.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080));
        let mut endpoint2_2 = MockEndpoint::new();
        endpoint2_2.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081));
        let mut endpoint2_3 = MockEndpoint::new();
        endpoint2_3.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8082));

        let mut endpoint3_1 = MockEndpoint::new();
        endpoint3_1.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8083));
        let mut endpoint3_2 = MockEndpoint::new();
        endpoint3_2.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8084));
        let mut endpoint3_3 = MockEndpoint::new();
        endpoint3_3.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8085));

        let mut endpoint4_1 = MockEndpoint::new();
        endpoint4_1.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8086));
        let mut endpoint4_2 = MockEndpoint::new();
        endpoint4_2.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8087));
        let mut endpoint4_3 = MockEndpoint::new();
        endpoint4_3.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8088));

        let dataset = vec![
            TestData {
                name: "all_healthy_start_from_0".to_string(),
                endpoints: vec![
                    Arc::new(Box::new(endpoint1_1)),
                    Arc::new(Box::new(endpoint1_2)),
                    Arc::new(Box::new(endpoint1_3)),
                ],
                start_idx: Some(AtomicUsize::new(0)),
                addr_expectation: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            },
            TestData {
                name: "all_healthy_start_from_1".to_string(),
                endpoints: vec![
                    Arc::new(Box::new(endpoint2_1)),
                    Arc::new(Box::new(endpoint2_2)),
                    Arc::new(Box::new(endpoint2_3)),
                ],
                start_idx: Some(AtomicUsize::new(1)),
                addr_expectation: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
            },
            TestData {
                name: "all_healthy_start_from_3".to_string(),
                endpoints: vec![
                    Arc::new(Box::new(endpoint3_1)),
                    Arc::new(Box::new(endpoint3_2)),
                    Arc::new(Box::new(endpoint3_3)),
                ],
                start_idx: Some(AtomicUsize::new(3)),
                addr_expectation: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8083),
            },
            TestData {
                name: "non_start_idx_provided".to_string(),
                endpoints: vec![
                    Arc::new(Box::new(endpoint4_1)),
                    Arc::new(Box::new(endpoint4_2)),
                    Arc::new(Box::new(endpoint4_3)),
                ],
                start_idx: None,
                addr_expectation: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8086),
            },
        ];
        for data in dataset {
            let mut round_robin = RoundRobin::new(data.start_idx);
            let target = round_robin.pick(&data.endpoints);
            assert_eq!(target.unwrap().addr(), data.addr_expectation, "test set: {}", data.name)
        }
    }
}