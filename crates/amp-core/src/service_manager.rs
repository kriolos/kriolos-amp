use crate::models::{
    LogEvent, PhpServerInstance, ServiceError, ServiceInfo, ServiceStatus, ServiceType,
    VersionSource,
};
use crate::port_checker::PortChecker;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub struct LogMessage {
    pub timestamp: String,
    pub event: LogEvent,
}

pub struct ServiceManager {
    base_bin_dir: PathBuf,
    active_processes: HashMap<ServiceType, Child>,
    active_ports: HashMap<ServiceType, u16>,
    service_errors: HashMap<ServiceType, ServiceError>,
    php_server_processes: HashMap<String, (Child, u16)>, // instance_id -> (Child, port)
    log_sender: broadcast::Sender<LogMessage>,
    recent_logs: Arc<Mutex<Vec<LogMessage>>>,
}

impl ServiceManager {
    pub fn new(base_bin_dir: PathBuf) -> Self {
        let (log_sender, _) = broadcast::channel(500);
        Self {
            base_bin_dir,
            active_processes: HashMap::new(),
            active_ports: HashMap::new(),
            service_errors: HashMap::new(),
            php_server_processes: HashMap::new(),
            log_sender,
            recent_logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn subscribe_logs(&self) -> broadcast::Receiver<LogMessage> {
        self.log_sender.subscribe()
    }

    pub fn get_recent_logs(&self) -> Vec<LogMessage> {
        self.recent_logs.lock().unwrap().clone()
    }

    pub fn add_log_event(&self, event: LogEvent) {
        let now = chrono_lite();
        let log_entry = LogMessage {
            timestamp: now,
            event,
        };

        let mut logs = self.recent_logs.lock().unwrap();
        logs.push(log_entry.clone());
        if logs.len() > 1000 {
            logs.remove(0);
        }
        let _ = self.log_sender.send(log_entry);
    }

    pub fn list_installed_versions(&self, service: ServiceType) -> Vec<String> {
        let folder = self.base_bin_dir.join(service.folder_name());
        let mut versions = Vec::new();
        if let Ok(entries) = fs::read_dir(folder) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    if let Ok(name) = entry.file_name().into_string() {
                        versions.push(name);
                    }
                }
            }
        }
        versions.sort();
        versions
    }

    pub fn get_service_info(
        &self,
        service: ServiceType,
        configured_version: Option<&str>,
        configured_source: Option<VersionSource>,
    ) -> ServiceInfo {
        let port = *self.active_ports.get(&service).unwrap_or(&service.default_port());
        let local_versions = self.list_installed_versions(service);

        let (active_version, version_source) = if let Some(ver) = configured_version {
            (Some(ver.to_string()), configured_source.or(Some(VersionSource::Downloaded)))
        } else if let Some(first_local) = local_versions.first() {
            (Some(first_local.clone()), Some(VersionSource::Downloaded))
        } else if let Some(sys_ver) = Self::detect_system_version(service) {
            (Some(sys_ver), Some(VersionSource::System))
        } else {
            (None, None)
        };

        let status = if let Some(child) = self.active_processes.get(&service) {
            let pid = child.id();
            if PortChecker::is_port_in_use(port) {
                ServiceStatus::Running { pid, port }
            } else {
                ServiceStatus::Starting
            }
        } else if let Some(err) = self.service_errors.get(&service) {
            ServiceStatus::Error(err.clone())
        } else if PortChecker::is_port_in_use(port) {
            ServiceStatus::Running { pid: 0, port }
        } else {
            ServiceStatus::Stopped
        };

        ServiceInfo {
            service_type: service,
            status,
            active_version,
            version_source,
            port,
            installed_versions: local_versions,
        }
    }

    pub fn detect_system_version(service: ServiceType) -> Option<String> {
        #[cfg(not(windows))]
        {
            let (cmd_name, args) = match service {
                ServiceType::Apache => ("apache2", vec!["-v"]),
                ServiceType::Php => ("php", vec!["-v"]),
                ServiceType::Mysql => ("mysqld", vec!["--version"]),
                ServiceType::Mariadb => ("mariadbd", vec!["--version"]),
                ServiceType::Postgresql => ("postgres", vec!["--version"]),
                ServiceType::Tool => return None,
            };

            if let Ok(output) = Command::new(cmd_name).args(&args).output() {
                if output.status.success() {
                    let out_str = String::from_utf8_lossy(&output.stdout);
                    for word in out_str.split_whitespace() {
                        let clean = word.trim_matches(|c: char| !c.is_numeric() && c != '.');
                        if clean.contains('.') && clean.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
                            return Some(clean.to_string());
                        }
                    }
                    return Some("Detect".to_string());
                }
            }
        }

        #[cfg(windows)]
        {
            let cmd_name = match service {
                ServiceType::Php => "php.exe",
                ServiceType::Mysql => "mysqld.exe",
                ServiceType::Mariadb => "mariadbd.exe",
                _ => return None,
            };
            if let Ok(output) = Command::new(cmd_name).arg("-v").output() {
                if output.status.success() {
                    return Some("Detect".to_string());
                }
            }
        }

        None
    }

    pub fn start_service(
        &mut self,
        service: ServiceType,
        version: &str,
        port: u16,
        custom_bin_path: Option<&str>,
    ) -> Result<(), ServiceError> {
        if self.active_processes.contains_key(&service) {
            return Ok(());
        }

        self.service_errors.remove(&service);

        if PortChecker::is_port_in_use(port) {
            let err = ServiceError::PortInUse { port };
            self.service_errors.insert(service, err.clone());
            self.add_log_event(LogEvent::Error(err.clone()));
            return Err(err);
        }

        let bin_path = match self.locate_binary(service, version, custom_bin_path) {
            Ok(p) => p,
            Err(e) => {
                self.service_errors.insert(service, e.clone());
                self.add_log_event(LogEvent::Error(e.clone()));
                return Err(e);
            }
        };

        self.add_log_event(LogEvent::Starting { service, port });

        let mut cmd = Command::new(&bin_path);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        match service {
            ServiceType::Apache => {
                cmd.args(["-D", "FOREGROUND"]);
            }
            ServiceType::Mysql | ServiceType::Mariadb => {
                cmd.arg(format!("--port={}", port));
            }
            ServiceType::Postgresql => {
                cmd.args(["-p", &port.to_string()]);
            }
            ServiceType::Php => {
                // php-fpm
            }
            ServiceType::Tool => {}
        }

        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                self.active_processes.insert(service, child);
                self.active_ports.insert(service, port);
                self.service_errors.remove(&service);
                self.add_log_event(LogEvent::Started { service, pid, port });
                Ok(())
            }
            Err(e) => {
                let err = ServiceError::ExecutionFailed {
                    service,
                    details: e.to_string(),
                };
                self.service_errors.insert(service, err.clone());
                self.add_log_event(LogEvent::Error(err.clone()));
                Err(err)
            }
        }
    }

    pub fn stop_service(&mut self, service: ServiceType) -> Result<(), ServiceError> {
        self.service_errors.remove(&service);
        if let Some(mut child) = self.active_processes.remove(&service) {
            self.add_log_event(LogEvent::Stopping { service });
            let _ = child.kill();
            let _ = child.wait();
            self.active_ports.remove(&service);
            self.add_log_event(LogEvent::Stopped { service });
            Ok(())
        } else {
            Ok(())
        }
    }

    // --- PHP Embedded Web Server (php -S) Management ---

    pub fn is_php_server_running(&self, instance_id: &str) -> bool {
        self.php_server_processes.contains_key(instance_id)
    }

    pub fn start_php_server(
        &mut self,
        instance: &PhpServerInstance,
        php_version: Option<&str>,
        custom_bin_path: Option<&str>,
    ) -> Result<(), ServiceError> {
        if self.php_server_processes.contains_key(&instance.id) {
            return Ok(());
        }

        if PortChecker::is_port_in_use(instance.port) {
            let err = ServiceError::PortInUse { port: instance.port };
            self.add_log_event(LogEvent::Error(err.clone()));
            return Err(err);
        }

        let php_bin = match self.locate_php_cli(php_version, custom_bin_path) {
            Ok(p) => p,
            Err(e) => {
                self.add_log_event(LogEvent::Error(e.clone()));
                return Err(e);
            }
        };

        let bind_addr = format!("{}:{}", instance.host, instance.port);

        self.add_log_event(LogEvent::StartingPhpServer {
            name: instance.name.clone(),
            host: instance.host.clone(),
            port: instance.port,
            doc_root: instance.doc_root.clone(),
        });

        let mut cmd = Command::new(&php_bin);
        cmd.args(["-S", &bind_addr, "-t", &instance.doc_root]);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                self.php_server_processes
                    .insert(instance.id.clone(), (child, instance.port));
                self.add_log_event(LogEvent::PhpServerStarted {
                    name: instance.name.clone(),
                    host: instance.host.clone(),
                    port: instance.port,
                    pid,
                });
                Ok(())
            }
            Err(e) => {
                let err = ServiceError::PhpServerFailed {
                    name: instance.name.clone(),
                    details: e.to_string(),
                };
                self.add_log_event(LogEvent::Error(err.clone()));
                Err(err)
            }
        }
    }

    pub fn stop_php_server(&mut self, instance_id: &str) -> Result<(), ServiceError> {
        if let Some((mut child, _)) = self.php_server_processes.remove(instance_id) {
            let _ = child.kill();
            let _ = child.wait();
            self.add_log_event(LogEvent::PhpServerStopped {
                name: instance_id.to_string(),
            });
        }
        Ok(())
    }

    pub fn stop_all(&mut self) {
        let services = [
            ServiceType::Apache,
            ServiceType::Php,
            ServiceType::Mariadb,
            ServiceType::Mysql,
            ServiceType::Postgresql,
        ];
        for s in services {
            let _ = self.stop_service(s);
        }

        let instance_ids: Vec<String> = self.php_server_processes.keys().cloned().collect();
        for id in instance_ids {
            let _ = self.stop_php_server(&id);
        }
    }

    pub fn discover_binary_path(service: ServiceType, version_dir: &Path) -> Option<PathBuf> {
        if !version_dir.exists() {
            return None;
        }

        // 1. Check common candidates first
        #[cfg(windows)]
        let candidates = match service {
            ServiceType::Apache => vec!["bin/httpd.exe", "httpd.exe"],
            ServiceType::Php => vec!["php.exe", "bin/php.exe", "php-cgi.exe"],
            ServiceType::Mysql => vec!["bin/mysqld.exe", "mysqld.exe"],
            ServiceType::Mariadb => vec!["bin/mariadbd.exe", "bin/mysqld.exe", "mariadbd.exe"],
            ServiceType::Postgresql => vec!["bin/postgres.exe", "postgres.exe"],
            ServiceType::Tool => vec![],
        };

        #[cfg(not(windows))]
        let candidates = match service {
            ServiceType::Apache => vec!["bin/httpd", "bin/apache2", "httpd", "apache2"],
            ServiceType::Php => vec!["usr/bin/php8.3", "usr/bin/php8.2", "usr/bin/php8.1", "usr/bin/php8.0", "usr/bin/php", "bin/php", "php", "sbin/php-fpm", "php-fpm"],
            ServiceType::Mysql => vec!["bin/mysqld", "mysqld"],
            ServiceType::Mariadb => vec!["bin/mariadbd", "bin/mysqld", "mariadbd", "mysqld"],
            ServiceType::Postgresql => vec!["bin/postgres", "postgres"],
            ServiceType::Tool => vec![],
        };

        for candidate in candidates {
            let path = version_dir.join(candidate);
            if path.is_file() {
                Self::ensure_executable(&path);
                return Some(path);
            }
        }

        // 2. Recursive search up to depth 4
        Self::find_executable_recursively(service, version_dir, 0, 4)
    }

    fn ensure_executable(_path: &Path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = fs::metadata(_path) {
                if meta.permissions().mode() & 0o111 == 0 {
                    let mut perms = meta.permissions();
                    perms.set_mode(perms.mode() | 0o755);
                    let _ = fs::set_permissions(_path, perms);
                }
            }
        }
    }

    fn find_executable_recursively(
        service: ServiceType,
        current_dir: &Path,
        depth: usize,
        max_depth: usize,
    ) -> Option<PathBuf> {
        if depth > max_depth {
            return None;
        }
        let entries = fs::read_dir(current_dir).ok()?;
        let mut subdirs = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name != "tests" && name != "test" && name != "docs" && name != "doc" && name != "ext" {
                    subdirs.push(path);
                }
            } else if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if Self::is_matching_binary(service, file_name) {
                    Self::ensure_executable(&path);
                    return Some(path);
                }
            }
        }

        for subdir in subdirs {
            if let Some(found) = Self::find_executable_recursively(service, &subdir, depth + 1, max_depth) {
                return Some(found);
            }
        }

        None
    }

    fn is_matching_binary(service: ServiceType, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        match service {
            ServiceType::Php => {
                #[cfg(windows)]
                { name_lower == "php.exe" || name_lower == "php-cgi.exe" }
                #[cfg(not(windows))]
                {
                    (name_lower == "php" || name_lower.starts_with("php8.") || name_lower.starts_with("php7."))
                        && !name_lower.contains("config")
                        && !name_lower.contains("ize")
                        && !name_lower.ends_with(".phar")
                        && !name_lower.ends_with(".h")
                        && !name_lower.ends_with(".c")
                }
            }
            ServiceType::Apache => {
                #[cfg(windows)]
                { name_lower == "httpd.exe" }
                #[cfg(not(windows))]
                { name_lower == "httpd" || name_lower == "apache2" }
            }
            ServiceType::Mariadb => {
                #[cfg(windows)]
                { name_lower == "mariadbd.exe" || name_lower == "mysqld.exe" }
                #[cfg(not(windows))]
                { name_lower == "mariadbd" || name_lower == "mysqld" }
            }
            ServiceType::Mysql => {
                #[cfg(windows)]
                { name_lower == "mysqld.exe" }
                #[cfg(not(windows))]
                { name_lower == "mysqld" }
            }
            ServiceType::Postgresql => {
                #[cfg(windows)]
                { name_lower == "postgres.exe" }
                #[cfg(not(windows))]
                { name_lower == "postgres" }
            }
            ServiceType::Tool => {
                #[cfg(windows)]
                { name_lower == "spc.exe" || name_lower.ends_with(".exe") }
                #[cfg(not(windows))]
                { name_lower == "spc" }
            }
        }
    }

    pub fn locate_binary(&self, service: ServiceType, version: &str, custom_bin_path: Option<&str>) -> Result<PathBuf, ServiceError> {
        // 1. Explicit path in config.json
        if let Some(custom) = custom_bin_path {
            let p = PathBuf::from(custom);
            if p.is_file() {
                Self::ensure_executable(&p);
                return Ok(p);
            }
        }

        let dir = self
            .base_bin_dir
            .join(service.folder_name())
            .join(version);

        // 2. Discover in downloaded folder
        if let Some(p) = Self::discover_binary_path(service, &dir) {
            return Ok(p);
        }

        // 3. Fallback to system binaries on Linux/macOS
        #[cfg(not(windows))]
        {
            let sys_binary = match service {
                ServiceType::Apache => "apache2",
                ServiceType::Php => "php",
                ServiceType::Mysql => "mysqld",
                ServiceType::Mariadb => "mariadbd",
                ServiceType::Postgresql => "postgres",
                ServiceType::Tool => "",
            };
            if !sys_binary.is_empty() {
                if let Ok(output) = Command::new("which").arg(sys_binary).output() {
                    if output.status.success() {
                        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        if !path_str.is_empty() {
                            return Ok(PathBuf::from(path_str));
                        }
                    }
                }
            }
        }

        Err(ServiceError::BinaryNotFound {
            service,
            path: dir.to_string_lossy().to_string(),
        })
    }

    pub fn locate_php_cli(&self, version: Option<&str>, custom_bin_path: Option<&str>) -> Result<PathBuf, ServiceError> {
        // 1. Explicit path in config.json
        if let Some(custom) = custom_bin_path {
            let p = PathBuf::from(custom);
            if p.is_file() {
                Self::ensure_executable(&p);
                return Ok(p);
            }
        }

        // 2. Discover in version folder
        if let Some(ver) = version {
            let dir = self.base_bin_dir.join("php").join(ver);
            if let Some(p) = Self::discover_binary_path(ServiceType::Php, &dir) {
                return Ok(p);
            }
        }

        // 3. Fallback to system php
        #[cfg(not(windows))]
        {
            if let Ok(output) = Command::new("which").arg("php").output() {
                if output.status.success() {
                    let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path_str.is_empty() {
                        return Ok(PathBuf::from(path_str));
                    }
                }
            }
        }

        #[cfg(windows)]
        {
            if let Ok(output) = Command::new("where").arg("php.exe").output() {
                if output.status.success() {
                    let path_str = String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or("").trim().to_string();
                    if !path_str.is_empty() {
                        return Ok(PathBuf::from(path_str));
                    }
                }
            }
        }

        Err(ServiceError::PhpCliNotFound)
    }
}

fn chrono_lite() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_binary_path_with_temp_dir() {
        let temp_dir = std::env::temp_dir().join("amp_test_php_discovery");
        let nested_bin = temp_dir.join("usr").join("bin");
        let _ = fs::create_dir_all(&nested_bin);

        #[cfg(not(windows))]
        let php_file = nested_bin.join("php8.3");
        #[cfg(windows)]
        let php_file = nested_bin.join("php.exe");

        let _ = fs::write(&php_file, b"mock_executable");

        let discovered = ServiceManager::discover_binary_path(ServiceType::Php, &temp_dir);
        assert!(discovered.is_some());
        assert_eq!(discovered.unwrap(), php_file);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
