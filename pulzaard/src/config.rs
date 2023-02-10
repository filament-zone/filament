pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_ABCI_PORT: &str = "26658";
pub const DEFAULT_METRICS_PORT: &str = "12121";

pub struct Config {
    pub host: String,
    pub abci_port: String,
    pub metrics_port: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_string(),
            abci_port: DEFAULT_ABCI_PORT.to_string(),
            metrics_port: DEFAULT_METRICS_PORT.to_string(),
        }
    }
}
