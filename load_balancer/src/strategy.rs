use std::option::Option;
use std::sync::Arc;

use mockall::automock;

use crate::endpoint::Endpoint;

pub mod round_robin;
pub mod weighted_round_robin;

#[automock]
pub trait RouteStrategy: Sync + Send {
    #[allow(dead_code)]
    fn name(&self) -> String;
    fn pick(&mut self, endpoints: &Vec<Arc<Box<dyn Endpoint>>>) -> Option<Arc<Box<dyn Endpoint>>>;
}

