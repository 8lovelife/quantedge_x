pub struct BookInfo {
    pub info: String,
}

impl BookInfo {
    pub fn new(info: String) -> Self {
        Self { info }
    }
}
