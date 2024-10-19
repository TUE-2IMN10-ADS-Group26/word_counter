use std::option::Option;
use std::sync::Arc;

use mockall::automock;

use crate::endpoint::Endpoint;
use crate::strategy::context::StrategyContext;

pub mod round_robin;
pub mod weighted_round_robin;
pub mod hash_lb;
pub mod context;

#[automock]
pub trait RouteStrategy: Sync + Send {
    #[allow(dead_code)]
    fn name(&self) -> String;
    fn pick(&mut self, ctx: &StrategyContext, endpoints: &Vec<Arc<Box<dyn Endpoint>>>) -> Option<Arc<Box<dyn Endpoint>>>;
}

