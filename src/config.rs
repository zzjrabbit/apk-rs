use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub root: Option<String>,
    pub image: Option<ImageConfig>,
    pub package: PackageConfig,
    pub files: Option<Vec<File>>,
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

#[derive(Deserialize)]
pub struct File {
    pub source: String,
    pub dest: String,
}
