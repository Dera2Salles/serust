#[derive(Debug, Clone)]
pub struct Session {
    pub username: Option<String>,
}

impl Session {
    pub fn new() -> Self {
        Session { username: None }
    }
}
