use std::fs;

pub struct Configs {
    pub developer: bool,
}

impl Configs {
    pub fn load() -> Self {
        let text = fs::read_to_string("desktop.toml").unwrap_or_default();
        let developer = text
            .lines()
            .find(|l| l.trim().starts_with("developer"))
            .and_then(|l| l.split('=').nth(1))
            .map(|v| v.trim() == "true")
            .unwrap_or(false);
        Self { developer }
    }
}