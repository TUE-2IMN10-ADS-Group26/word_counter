use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use crate::consts::HASH_BY_REQUEST;
use crate::endpoint::Endpoint;
use crate::strategy::context::StrategyContext;
use crate::strategy::RouteStrategy;

pub struct HashByRequest;

impl HashByRequest {
    pub fn new() -> Self {
        HashByRequest {}
    }

    pub fn hash<T: Hash>(t: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        t.hash(&mut hasher);
        hasher.finish()
    }
}

impl RouteStrategy for HashByRequest {
    fn name(&self) -> String {
        String::from(HASH_BY_REQUEST)
    }

    fn pick(&mut self, ctx: &StrategyContext, endpoints: &Vec<Arc<Box<dyn Endpoint>>>) -> Option<Arc<Box<dyn Endpoint>>> {
        let hash = Self::hash(&ctx.req());
        let server_idx = (hash as usize) % endpoints.len();
        endpoints.get(server_idx).map(|endpoint| { Arc::clone(endpoint) })
    }
}

#[cfg(test)]
mod round_robin_test {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use crate::consts::HASH_BY_REQUEST;
    use crate::endpoint::MockEndpoint;

    use super::*;

    struct TestData {
        name: String,
        endpoints: Vec<Arc<Box<dyn Endpoint>>>,
        ctx_pair: Vec<(StrategyContext, StrategyContext)>,
    }

    #[test]
    fn test_name() {
        assert_eq!(HASH_BY_REQUEST, HashByRequest::new().name());
    }

    #[test]
    fn test_pick() {
        let mut endpoint1_1 = MockEndpoint::new();
        endpoint1_1.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080));
        let mut endpoint1_2 = MockEndpoint::new();
        endpoint1_2.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081));
        let mut endpoint1_3 = MockEndpoint::new();
        endpoint1_3.expect_addr().returning(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8082));

        let dataset = vec![
            TestData {
                name: "test_idempotency".to_string(),
                endpoints: vec![
                    Arc::new(Box::new(endpoint1_1)),
                    Arc::new(Box::new(endpoint1_2)),
                    Arc::new(Box::new(endpoint1_3)),
                ],
                ctx_pair: vec![
                    (StrategyContext::new(String::from("test req1")), StrategyContext::new(String::from("test req1"))),
                    (StrategyContext::new(String::from("{\"word\": \"hello\"}")), StrategyContext::new(String::from("{\"word\": \"hello\"}"))),
                    (StrategyContext::new(String::from("{\"word\": \"hello\", \"file\": \"Titanic.txt\"}")), StrategyContext::new(String::from("{\"word\": \"hello\", \"file\": \"Titanic.txt\"}"))),
                ],
            },
        ];
        for data in dataset {
            let mut hash_lb = HashByRequest::new();
            for (ctx1, ctx2) in data.ctx_pair {
                let target1 = hash_lb.pick(&ctx1, &data.endpoints);
                let target2 = hash_lb.pick(&ctx2, &data.endpoints);
                assert_eq!(target1.unwrap().addr(), target2.unwrap().addr(), "test set: {}", data.name)
            }
        }
    }
}