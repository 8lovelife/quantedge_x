pub trait Storage {
    fn save_snapshot(&self, data: &[u8]) -> anyhow::Result<()>;
    fn load_latest_snapshot(&self) -> anyhow::Result<Option<Vec<u8>>>;
}
