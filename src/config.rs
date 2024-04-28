use std::path::Path;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub style_name: String,
    pub terminal_app: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            style_name: "crates.io".into(),
            terminal_app: String::new(),
        }
    }
}

const FILENAME: &str = "config.toml";

impl Config {
    fn load(dir: &Path) -> anyhow::Result<Self> {
        let path = dir.join(FILENAME);
        let data = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&data)?)
    }
    pub fn load_or_default(dir: &Path) -> Self {
        match Self::load(dir) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Error loading config: {e}. Using default.");
                Self::default()
            }
        }
    }
    pub fn save(&self, dir: &Path) -> anyhow::Result<()> {
        if !dir.exists() {
            std::fs::create_dir_all(dir)?;
        }
        let path = dir.join(FILENAME);
        Ok(std::fs::write(path, toml::to_string_pretty(self)?)?)
    }
}
