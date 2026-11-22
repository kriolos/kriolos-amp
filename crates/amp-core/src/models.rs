use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
    CommandExecuted { command: String },
    ProcessOutput { source: String, line: String, is_stderr: bool },
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

pub mod defaults {
    pub const DEFAULT_HOST: &str = "127.0.0.1";
    pub const DEFAULT_DOC_ROOT: &str = "www";
    pub const DEFAULT_BIN_DIR: &str = "bin";
    pub const DEFAULT_DATA_DIR: &str = "data";
    pub const DEFAULT_LOGS_DIR: &str = "logs";
    pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1";
    pub const DEFAULT_APACHE_PORT: u16 = 8080;
    pub const DEFAULT_APACHE_SSL_PORT: u16 = 8443;
    pub const DEFAULT_PHP_FPM_PORT: u16 = 9000;
    pub const DEFAULT_MYSQL_PORT: u16 = 3306;
    pub const DEFAULT_MARIADB_PORT: u16 = 3307;
    pub const DEFAULT_POSTGRESQL_PORT: u16 = 5432;
    pub const DEFAULT_PHP_SERVER_PORT: u16 = 8000;
    pub const DEFAULT_DNS_PORT: u16 = 5353;
    pub const DEFAULT_DNS_TLD: &str = "test";
    pub const DEFAULT_DNS_UPSTREAM: &str = "1.1.1.1:53";
    pub const DEFAULT_LANGUAGE: &str = "pt-PT";
    pub const DEFAULT_THEME: &str = "dark";
    pub const DEFAULT_MANIFEST_URL: &str =
        "https://raw.githubusercontent.com/kriolos/kriolos-amp/main/assets/versions.json";
}

pub fn default_host() -> String {
    defaults::DEFAULT_HOST.to_string()
}

fn default_language() -> String {
    defaults::DEFAULT_LANGUAGE.to_string()
}

fn default_theme() -> String {
    defaults::DEFAULT_THEME.to_string()
}

fn default_root_www_dir() -> PathBuf {
    PathBuf::from(defaults::DEFAULT_DOC_ROOT)
}

fn default_bin_dir() -> PathBuf {
    PathBuf::from(defaults::DEFAULT_BIN_DIR)
}

fn default_logs_dir() -> PathBuf {
    PathBuf::from(defaults::DEFAULT_LOGS_DIR)
}

fn default_data_dir() -> PathBuf {
    PathBuf::from(defaults::DEFAULT_DATA_DIR)
}

fn default_bind_address() -> String {
    defaults::DEFAULT_BIND_ADDRESS.to_string()
}

fn default_apache_port() -> u16 {
    defaults::DEFAULT_APACHE_PORT
}

fn default_apache_ssl_port() -> u16 {
    defaults::DEFAULT_APACHE_SSL_PORT
}

fn default_php_fpm_port() -> u16 {
    defaults::DEFAULT_PHP_FPM_PORT
}

fn default_mysql_port() -> u16 {
    defaults::DEFAULT_MYSQL_PORT
}

fn default_mariadb_port() -> u16 {
    defaults::DEFAULT_MARIADB_PORT
}

fn default_postgresql_port() -> u16 {
    defaults::DEFAULT_POSTGRESQL_PORT
}

fn default_dns_port() -> u16 {
    defaults::DEFAULT_DNS_PORT
}

fn default_dns_tld() -> String {
    defaults::DEFAULT_DNS_TLD.to_string()
}

fn default_dns_upstream() -> String {
    defaults::DEFAULT_DNS_UPSTREAM.to_string()
}

fn default_manifest_url() -> String {
    defaults::DEFAULT_MANIFEST_URL.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpServerInstance {
    pub id: String,
    pub name: String,
    #[serde(default = "default_host")]
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
            host: defaults::DEFAULT_HOST.to_string(),
            port: defaults::DEFAULT_PHP_SERVER_PORT,
            doc_root: defaults::DEFAULT_DOC_ROOT.to_string(),
            php_version: None,
            auto_start: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceVersionEntry {
    pub binary_path: String,
    #[serde(default = "default_downloaded_source")]
    pub source: VersionSource,
}

fn default_downloaded_source() -> VersionSource {
    VersionSource::Downloaded
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ServiceConfigEntry {
    #[serde(default)]
    pub active_version: Option<String>,
    #[serde(default)]
    pub versions: BTreeMap<String, ServiceVersionEntry>,
}

fn default_services_config() -> BTreeMap<String, ServiceConfigEntry> {
    let mut map = BTreeMap::new();
    map.insert("apache".to_string(), ServiceConfigEntry::default());
    map.insert("php".to_string(), ServiceConfigEntry::default());
    map.insert("mariadb".to_string(), ServiceConfigEntry::default());
    map.insert("mysql".to_string(), ServiceConfigEntry::default());
    map.insert("postgresql".to_string(), ServiceConfigEntry::default());
    map
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_theme")]
    pub theme: String, // "dark", "light", "system"
    #[serde(default = "default_root_www_dir")]
    pub root_www_dir: PathBuf,
    #[serde(default = "default_bin_dir")]
    pub bin_dir: PathBuf,
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
    #[serde(default = "default_logs_dir")]
    pub logs_dir: PathBuf,
    #[serde(default = "default_host")]
    pub default_host: String,
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
    #[serde(default = "default_apache_port")]
    pub apache_port: u16,
    #[serde(default = "default_apache_ssl_port")]
    pub apache_ssl_port: u16,
    #[serde(default = "default_php_fpm_port")]
    pub php_fpm_port: u16,
    #[serde(default = "default_mysql_port")]
    pub mysql_port: u16,
    #[serde(default = "default_mariadb_port")]
    pub mariadb_port: u16,
    #[serde(default = "default_postgresql_port")]
    pub postgresql_port: u16,
    #[serde(default)]
    pub dns_enabled: bool,
    #[serde(default = "default_dns_port")]
    pub dns_port: u16,
    #[serde(default = "default_dns_tld")]
    pub dns_tld: String,
    #[serde(default = "default_dns_upstream")]
    pub dns_upstream: String,
    #[serde(default)]
    pub dns_sync_hosts: bool,
    pub active_apache_version: Option<String>,
    pub active_php_version: Option<String>,
    pub active_mysql_version: Option<String>,
    pub active_mariadb_version: Option<String>,
    pub active_postgresql_version: Option<String>,
    #[serde(default = "default_manifest_url")]
    pub remote_manifest_url: String,
    #[serde(default)]
    pub php_servers: Vec<PhpServerInstance>,
    #[serde(default = "default_services_config")]
    pub services: BTreeMap<String, ServiceConfigEntry>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: defaults::DEFAULT_LANGUAGE.to_string(),
            theme: defaults::DEFAULT_THEME.to_string(),
            root_www_dir: PathBuf::from(defaults::DEFAULT_DOC_ROOT),
            bin_dir: PathBuf::from(defaults::DEFAULT_BIN_DIR),
            data_dir: PathBuf::from(defaults::DEFAULT_DATA_DIR),
            logs_dir: PathBuf::from(defaults::DEFAULT_LOGS_DIR),
            default_host: defaults::DEFAULT_HOST.to_string(),
            bind_address: defaults::DEFAULT_BIND_ADDRESS.to_string(),
            apache_port: defaults::DEFAULT_APACHE_PORT,
            apache_ssl_port: defaults::DEFAULT_APACHE_SSL_PORT,
            php_fpm_port: defaults::DEFAULT_PHP_FPM_PORT,
            mysql_port: defaults::DEFAULT_MYSQL_PORT,
            mariadb_port: defaults::DEFAULT_MARIADB_PORT,
            postgresql_port: defaults::DEFAULT_POSTGRESQL_PORT,
            dns_enabled: false,
            dns_port: defaults::DEFAULT_DNS_PORT,
            dns_tld: defaults::DEFAULT_DNS_TLD.to_string(),
            dns_upstream: defaults::DEFAULT_DNS_UPSTREAM.to_string(),
            dns_sync_hosts: false,
            active_apache_version: None,
            active_php_version: None,
            active_mysql_version: None,
            active_mariadb_version: None,
            active_postgresql_version: None,
            remote_manifest_url: defaults::DEFAULT_MANIFEST_URL.to_string(),
            php_servers: vec![PhpServerInstance::default()],
            services: default_services_config(),
        }
    }
}

impl AppConfig {
    pub fn get_service_entry(&self, service: ServiceType) -> Option<&ServiceConfigEntry> {
        self.services.get(service.folder_name())
    }

    pub fn get_service_entry_mut(&mut self, service: ServiceType) -> &mut ServiceConfigEntry {
        self.services.entry(service.folder_name().to_string()).or_default()
    }

    pub fn get_active_version(&self, service: ServiceType) -> Option<&str> {
        if let Some(entry) = self.services.get(service.folder_name()) {
            if let Some(active) = &entry.active_version {
                return Some(active.as_str());
            }
        }
        match service {
            ServiceType::Apache => self.active_apache_version.as_deref(),
            ServiceType::Php => self.active_php_version.as_deref(),
            ServiceType::Mysql => self.active_mysql_version.as_deref(),
            ServiceType::Mariadb => self.active_mariadb_version.as_deref(),
            ServiceType::Postgresql => self.active_postgresql_version.as_deref(),
            ServiceType::Tool => None,
        }
    }

    pub fn set_active_version(&mut self, service: ServiceType, version: Option<String>) {
        let entry = self.get_service_entry_mut(service);
        entry.active_version = version.clone();
        match service {
            ServiceType::Apache => self.active_apache_version = version,
            ServiceType::Php => self.active_php_version = version,
            ServiceType::Mysql => self.active_mysql_version = version,
            ServiceType::Mariadb => self.active_mariadb_version = version,
            ServiceType::Postgresql => self.active_postgresql_version = version,
            ServiceType::Tool => {}
        }
    }

    pub fn register_version(
        &mut self,
        service: ServiceType,
        version: String,
        binary_path: String,
        source: VersionSource,
    ) {
        let entry = self.get_service_entry_mut(service);
        let set_active = entry.active_version.is_none();
        entry.versions.insert(
            version.clone(),
            ServiceVersionEntry {
                binary_path,
                source,
            },
        );
        if set_active {
            entry.active_version = Some(version.clone());
            match service {
                ServiceType::Apache => self.active_apache_version = Some(version),
                ServiceType::Php => self.active_php_version = Some(version),
                ServiceType::Mysql => self.active_mysql_version = Some(version),
                ServiceType::Mariadb => self.active_mariadb_version = Some(version),
                ServiceType::Postgresql => self.active_postgresql_version = Some(version),
                ServiceType::Tool => {}
            }
        }
    }

    pub fn get_binary_path(&self, service: ServiceType, version: &str) -> Option<PathBuf> {
        if let Some(entry) = self.services.get(service.folder_name()) {
            if let Some(v_entry) = entry.versions.get(version) {
                return Some(PathBuf::from(&v_entry.binary_path));
            }
        }
        None
    }

    pub fn get_active_binary_path(&self, service: ServiceType) -> Option<PathBuf> {
        let active_ver = self.get_active_version(service)?;
        self.get_binary_path(service, active_ver)
    }

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
    #[serde(default)]
    pub binary_path: Option<String>,
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
            binary_path: None,
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
            binary_path: None,
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
            binary_path: None,
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

    #[test]
    fn test_app_config_service_version_management() {
        let mut config = AppConfig::default();
        assert_eq!(config.get_active_version(ServiceType::Php), None);

        config.register_version(
            ServiceType::Php,
            "8.3.13".to_string(),
            "bin/php/8.3.13/usr/bin/php8.3".to_string(),
            VersionSource::Downloaded,
        );

        // First registered becomes active
        assert_eq!(config.get_active_version(ServiceType::Php), Some("8.3.13"));
        assert_eq!(
            config.get_active_binary_path(ServiceType::Php),
            Some(PathBuf::from("bin/php/8.3.13/usr/bin/php8.3"))
        );

        // Register second version
        config.register_version(
            ServiceType::Php,
            "8.5.7".to_string(),
            "bin/php/8.5.7/php".to_string(),
            VersionSource::Downloaded,
        );

        // Active still 8.3.13 until explicitly switched
        assert_eq!(config.get_active_version(ServiceType::Php), Some("8.3.13"));

        // Switch to 8.5.7
        config.set_active_version(ServiceType::Php, Some("8.5.7".to_string()));
        assert_eq!(config.get_active_version(ServiceType::Php), Some("8.5.7"));
        assert_eq!(
            config.get_active_binary_path(ServiceType::Php),
            Some(PathBuf::from("bin/php/8.5.7/php"))
        );
    }

    #[test]
    fn test_version_manifest_static_php_packages() {
        let manifest_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("assets")
            .join("versions.json");

        let content = std::fs::read_to_string(&manifest_path).expect("versions.json must exist");
        let manifest: VersionManifest = serde_json::from_str(&content).expect("manifest must parse");

        let static_php_pkgs: Vec<_> = manifest
            .packages
            .iter()
            .filter(|p| p.service_type == ServiceType::Php && p.version.starts_with("8.3.11-static"))
            .collect();

        assert!(!static_php_pkgs.is_empty(), "Should contain Static PHP packages under ServiceType::Php");

        let linux_pkg = static_php_pkgs.iter().find(|p| p.platform == "linux-x86_64").expect("Linux x86_64 static package missing");
        assert_eq!(linux_pkg.version, "8.3.11-static-linux-x64");
        assert!(linux_pkg.recommended, "Static PHP should be recommended for linux");
        assert_eq!(linux_pkg.binary_path.as_deref(), Some("php"));
        assert!(linux_pkg.url.contains("static-php-cli"));

        let php86_pkg = manifest
            .packages
            .iter()
            .find(|p| p.service_type == ServiceType::Php && p.version == "8.6.0-dev-src")
            .expect("PHP 8.6 source package must exist in manifest");
        assert_eq!(php86_pkg.platform, "any");
    }

    #[test]
    fn test_app_config_defaults_and_custom_host() {
        // Test default instance
        let default_config = AppConfig::default();
        assert_eq!(default_config.default_host, defaults::DEFAULT_HOST);
        assert_eq!(default_config.bind_address, defaults::DEFAULT_BIND_ADDRESS);
        assert_eq!(default_config.root_www_dir, PathBuf::from(defaults::DEFAULT_DOC_ROOT));
        assert_eq!(default_config.data_dir, PathBuf::from(defaults::DEFAULT_DATA_DIR));

        // Test backward-compatibility deserialization (empty JSON object)
        let parsed: AppConfig = serde_json::from_str("{}").expect("empty json should deserialize with defaults");
        assert_eq!(parsed.default_host, defaults::DEFAULT_HOST);
        assert_eq!(parsed.bind_address, defaults::DEFAULT_BIND_ADDRESS);
        assert_eq!(parsed.data_dir, PathBuf::from(defaults::DEFAULT_DATA_DIR));
        assert_eq!(parsed.apache_port, defaults::DEFAULT_APACHE_PORT);

        // Test custom host & bind_address serialization/deserialization
        let custom_json = r#"{"default_host": "0.0.0.0", "bind_address": "192.168.1.100"}"#;
        let custom_parsed: AppConfig = serde_json::from_str(custom_json).expect("custom json should deserialize");
        assert_eq!(custom_parsed.default_host, "0.0.0.0");
        assert_eq!(custom_parsed.bind_address, "192.168.1.100");
    }
}
