pub trait ChatBotTrait {
    async fn ask_orbit(&self, question: &str) -> anyhow::Result<String>;
}
