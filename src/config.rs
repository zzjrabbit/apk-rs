use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub root: Option<String>,
    pub image: Option<ImageConfig>,
    pub package: PackageConfig,
}

#[derive(Deserialize)]
pub struct ImageConfig {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct PackageConfig {
    pub list: Vec<String>,
    pub mirror: Option<String>,
}
