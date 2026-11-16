use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceType {
    Apache,
    Database,
    Php,
    Tool,
}

impl ServiceType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ServiceType::Apache => "Apache HTTP Server",
            ServiceType::Database => "MariaDB / MySQL",
            ServiceType::Php => "PHP Engine",
            ServiceType::Tool => "Ferramenta / Runtime",
        }
    }

    pub fn default_port(&self) -> u16 {
        match self {
            ServiceType::Apache => 8080,
            ServiceType::Database => 3306,
            ServiceType::Php => 9000,
            ServiceType::Tool => 0,
        }
    }

    pub fn folder_name(&self) -> &'static str {
        match self {
            ServiceType::Apache => "apache",
            ServiceType::Database => "database",
            ServiceType::Php => "php",
            ServiceType::Tool => "tools",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    Stopped,
    Starting,
    Running { pid: u32, port: u16 },
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub service_type: ServiceType,
    pub status: ServiceStatus,
    pub active_version: Option<String>,
    pub port: u16,
    pub installed_versions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub language: String,
    pub theme: String, // "dark", "light", "system"
    pub root_www_dir: PathBuf,
    pub apache_port: u16,
    pub apache_ssl_port: u16,
    pub mysql_port: u16,
    pub active_apache_version: Option<String>,
    pub active_mysql_version: Option<String>,
    pub active_php_version: Option<String>,
    pub remote_manifest_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: "pt-PT".to_string(),
            theme: "dark".to_string(),
            root_www_dir: PathBuf::from("www"),
            apache_port: 8080,
            apache_ssl_port: 8443,
            mysql_port: 3306,
            active_apache_version: None,
            active_mysql_version: None,
            active_php_version: None,
            remote_manifest_url: "https://raw.githubusercontent.com/antigravity-community/amp-control/main/assets/versions.json".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    pub updated_at: String,
    pub packages: Vec<PackageItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageItem {
    pub id: String,
    pub service_type: ServiceType,
    pub version: String,
    pub platform: String, // e.g. "linux-x86_64", "windows-x86_64", "any"
    pub url: String,
    pub sha256: Option<String>,
    pub size_bytes: Option<u64>,
    pub recommended: bool,
    pub description: Option<String>,
    #[serde(default)]
    pub external_link: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadState {
    Idle,
    Downloading {
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
        percent: f32,
    },
    Extracting,
    Completed,
    Failed(String),
}
