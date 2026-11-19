use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceType {
    Apache,
    Php,
    Mariadb,
    Mysql,
    Postgresql,
    Tool,
}

impl ServiceType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ServiceType::Apache => "Apache HTTP Server",
            ServiceType::Php => "PHP Engine (FPM / CLI)",
            ServiceType::Mariadb => "MariaDB Database",
            ServiceType::Mysql => "MySQL Database",
            ServiceType::Postgresql => "PostgreSQL Database",
            ServiceType::Tool => "Tools & Runtimes",
        }
    }

    pub fn default_port(&self) -> u16 {
        match self {
            ServiceType::Apache => 8080,
            ServiceType::Php => 9000,
            ServiceType::Mariadb => 3307,
            ServiceType::Mysql => 3306,
            ServiceType::Postgresql => 5432,
            ServiceType::Tool => 0,
        }
    }

    pub fn folder_name(&self) -> &'static str {
        match self {
            ServiceType::Apache => "apache",
            ServiceType::Php => "php",
            ServiceType::Mariadb => "mariadb",
            ServiceType::Mysql => "mysql",
            ServiceType::Postgresql => "postgresql",
            ServiceType::Tool => "tools",
        }
    }

    pub fn badge_label(&self) -> &'static str {
        match self {
            ServiceType::Apache => "WEB",
            ServiceType::Php => "PHP",
            ServiceType::Mariadb => "MARIADB",
            ServiceType::Mysql => "MYSQL",
            ServiceType::Postgresql => "POSTGRES",
            ServiceType::Tool => "TOOL",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ServiceType::Apache => "🌐",
            ServiceType::Php => "🐘",
            ServiceType::Mariadb => "🗄️",
            ServiceType::Mysql => "🐬",
            ServiceType::Postgresql => "🐘",
            ServiceType::Tool => "🛠️",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionSource {
    Downloaded,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageKind {
    Binary,
    Source,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceError {
    PortInUse { port: u16 },
    BinaryNotFound { service: ServiceType, path: String },
    PhpCliNotFound,
    ExecutionFailed { service: ServiceType, details: String },
    PhpServerFailed { name: String, details: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogEvent {
    Starting { service: ServiceType, port: u16 },
    Started { service: ServiceType, pid: u32, port: u16 },
    Stopping { service: ServiceType },
    Stopped { service: ServiceType },
    StartingPhpServer { name: String, host: String, port: u16, doc_root: String },
    PhpServerStarted { name: String, host: String, port: u16, pid: u32 },
    PhpServerStopped { name: String },
    Error(ServiceError),
    DownloadStarted { item_id: String, url: String },
    DownloadFailed { item_id: String, url: String, error: String },
    DownloadCompleted { item_id: String, url: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    Stopped,
    Starting,
    Running { pid: u32, port: u16 },
    Error(ServiceError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub service_type: ServiceType,
    pub status: ServiceStatus,
    pub active_version: Option<String>,
    pub version_source: Option<VersionSource>,
    pub port: u16,
    pub installed_versions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpServerInstance {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub doc_root: String,
    pub php_version: Option<String>,
    pub auto_start: bool,
}

impl Default for PhpServerInstance {
    fn default() -> Self {
        Self {
            id: "default-php-server".to_string(),
            name: "Default PHP Server".to_string(),
            host: "127.0.0.1".to_string(),
            port: 8000,
            doc_root: "www".to_string(),
            php_version: None,
            auto_start: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub language: String,
    pub theme: String, // "dark", "light", "system"
    pub root_www_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub apache_port: u16,
    pub apache_ssl_port: u16,
    pub php_fpm_port: u16,
    pub mysql_port: u16,
    pub mariadb_port: u16,
    pub postgresql_port: u16,
    pub active_apache_version: Option<String>,
    pub active_php_version: Option<String>,
    pub active_mysql_version: Option<String>,
    pub active_mariadb_version: Option<String>,
    pub active_postgresql_version: Option<String>,
    pub remote_manifest_url: String,
    pub php_servers: Vec<PhpServerInstance>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: "pt-PT".to_string(),
            theme: "dark".to_string(),
            root_www_dir: PathBuf::from("www"),
            bin_dir: PathBuf::from("bin"),
            logs_dir: PathBuf::from("logs"),
            apache_port: 8080,
            apache_ssl_port: 8443,
            php_fpm_port: 9000,
            mysql_port: 3306,
            mariadb_port: 3307,
            postgresql_port: 5432,
            active_apache_version: None,
            active_php_version: None,
            active_mysql_version: None,
            active_mariadb_version: None,
            active_postgresql_version: None,
            remote_manifest_url: "https://raw.githubusercontent.com/antigravity-community/amp-control/main/assets/versions.json".to_string(),
            php_servers: vec![PhpServerInstance::default()],
        }
    }
}

impl AppConfig {
    pub fn load_or_default(path: &Path) -> Self {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                    return cfg;
                }
            }
        }
        let default_cfg = Self::default();
        let _ = default_cfg.save(path);
        default_cfg
    }

    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn reset_to_defaults(&mut self) {
        *self = Self::default();
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
    #[serde(default = "default_package_kind")]
    pub package_kind: PackageKind,
    pub version: String,
    pub platform: String,
    pub url: String,
    pub sha256: Option<String>,
    pub size_bytes: Option<u64>,
    pub recommended: bool,
    pub description: Option<String>,
    #[serde(default)]
    pub external_link: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OsFilter {
    All,
    Current,
    Linux,
    Windows,
    MacOs,
}

impl Default for OsFilter {
    fn default() -> Self {
        Self::All
    }
}

impl PackageItem {
    pub fn matches_os(&self, filter: OsFilter) -> bool {
        match filter {
            OsFilter::All => true,
            OsFilter::Current => {
                let current_os = if cfg!(target_os = "windows") {
                    "windows"
                } else if cfg!(target_os = "macos") {
                    "macos"
                } else {
                    "linux"
                };
                let plat = self.platform.to_lowercase();
                plat == "any" || plat.starts_with(current_os)
            }
            OsFilter::Linux => {
                let plat = self.platform.to_lowercase();
                plat == "any" || plat.starts_with("linux")
            }
            OsFilter::Windows => {
                let plat = self.platform.to_lowercase();
                plat == "any" || plat.starts_with("windows")
            }
            OsFilter::MacOs => {
                let plat = self.platform.to_lowercase();
                plat == "any" || plat.starts_with("macos")
            }
        }
    }
}

fn default_package_kind() -> PackageKind {
    PackageKind::Binary
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_filter_matching() {
        let linux_pkg = PackageItem {
            id: "mariadb-linux".to_string(),
            service_type: ServiceType::Mariadb,
            package_kind: PackageKind::Binary,
            version: "11.4.3".to_string(),
            platform: "linux-x86_64".to_string(),
            url: "https://example.com/mariadb.tar.gz".to_string(),
            sha256: None,
            size_bytes: None,
            recommended: true,
            description: None,
            external_link: false,
        };

        let windows_pkg = PackageItem {
            id: "mariadb-win".to_string(),
            service_type: ServiceType::Mariadb,
            package_kind: PackageKind::Binary,
            version: "11.4.3".to_string(),
            platform: "windows-x86_64".to_string(),
            url: "https://example.com/mariadb.zip".to_string(),
            sha256: None,
            size_bytes: None,
            recommended: false,
            description: None,
            external_link: false,
        };

        let any_pkg = PackageItem {
            id: "source-code".to_string(),
            service_type: ServiceType::Mariadb,
            package_kind: PackageKind::Source,
            version: "11.4.3".to_string(),
            platform: "any".to_string(),
            url: "https://example.com/source.tar.gz".to_string(),
            sha256: None,
            size_bytes: None,
            recommended: false,
            description: None,
            external_link: false,
        };

        // All filter
        assert!(linux_pkg.matches_os(OsFilter::All));
        assert!(windows_pkg.matches_os(OsFilter::All));
        assert!(any_pkg.matches_os(OsFilter::All));

        // Linux filter
        assert!(linux_pkg.matches_os(OsFilter::Linux));
        assert!(!windows_pkg.matches_os(OsFilter::Linux));
        assert!(any_pkg.matches_os(OsFilter::Linux));

        // Windows filter
        assert!(!linux_pkg.matches_os(OsFilter::Windows));
        assert!(windows_pkg.matches_os(OsFilter::Windows));
        assert!(any_pkg.matches_os(OsFilter::Windows));

        // MacOs filter
        assert!(!linux_pkg.matches_os(OsFilter::MacOs));
        assert!(!windows_pkg.matches_os(OsFilter::MacOs));
        assert!(any_pkg.matches_os(OsFilter::MacOs));
    }
}
