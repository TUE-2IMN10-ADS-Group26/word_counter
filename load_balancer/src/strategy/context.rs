pub struct StrategyContext {
    req: String,
}

impl StrategyContext {
    pub fn new(req: String) -> Self {
        StrategyContext {
            req
        }
    }

    pub fn req(&self) -> &str {
        &self.req
    }
}