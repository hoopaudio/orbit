use std::future::Future;

pub trait OrbitAgentTrait {
    fn ask_orbit(&self, question: &str) -> impl Future<Output = anyhow::Result<String>> + Send;
}
