use crate::models::{ServiceInfo, ServiceStatus, ServiceType};
use crate::port_checker::PortChecker;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub struct LogMessage {
    pub service: ServiceType,
    pub timestamp: String,
    pub message: String,
    pub is_error: bool,
}

pub struct ServiceManager {
    base_bin_dir: PathBuf,
    active_processes: HashMap<ServiceType, Child>,
    active_ports: HashMap<ServiceType, u16>,
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

    pub fn add_log(&self, service: ServiceType, msg: &str, is_error: bool) {
        let now = chrono_lite();
        let log_entry = LogMessage {
            service,
            timestamp: now,
            message: msg.to_string(),
            is_error,
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

    pub fn get_service_info(&self, service: ServiceType) -> ServiceInfo {
        let port = *self.active_ports.get(&service).unwrap_or(&service.default_port());
        let installed = self.list_installed_versions(service);

        let status = if let Some(child) = self.active_processes.get(&service) {
            let pid = child.id();
            if PortChecker::is_port_in_use(port) {
                ServiceStatus::Running { pid, port }
            } else {
                ServiceStatus::Starting
            }
        } else if PortChecker::is_port_in_use(port) {
            ServiceStatus::Running { pid: 0, port }
        } else {
            ServiceStatus::Stopped
        };

        ServiceInfo {
            service_type: service,
            status,
            active_version: installed.first().cloned(),
            port,
            installed_versions: installed,
        }
    }

    pub fn start_service(
        &mut self,
        service: ServiceType,
        version: &str,
        port: u16,
    ) -> Result<(), String> {
        if self.active_processes.contains_key(&service) {
            return Ok(());
        }

        if PortChecker::is_port_in_use(port) {
            let err = format!("Porta {} já está ocupada por outro processo!", port);
            self.add_log(service, &err, true);
            return Err(err);
        }

        let bin_path = self.locate_binary(service, version)?;
        self.add_log(
            service,
            &format!("Iniciando {} (v{}) na porta {}...", service.display_name(), version, port),
            false,
        );

        let mut cmd = Command::new(&bin_path);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        // Platform and service specific args
        match service {
            ServiceType::Apache => {
                // e.g. -D FOREGROUND or -k start
            }
            ServiceType::Database => {
                // e.g. --port=3306
            }
            ServiceType::Php => {
                // e.g. php-fpm
            }
        }

        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                self.active_processes.insert(service, child);
                self.active_ports.insert(service, port);
                self.add_log(
                    service,
                    &format!("Serviço {} iniciado com sucesso (PID: {})", service.display_name(), pid),
                    false,
                );
                Ok(())
            }
            Err(e) => {
                let err = format!("Falha ao iniciar {}: {}", service.display_name(), e);
                self.add_log(service, &err, true);
                Err(err)
            }
        }
    }

    pub fn stop_service(&mut self, service: ServiceType) -> Result<(), String> {
        if let Some(mut child) = self.active_processes.remove(&service) {
            self.add_log(
                service,
                &format!("Parando serviço {}...", service.display_name()),
                false,
            );
            let _ = child.kill();
            let _ = child.wait();
            self.active_ports.remove(&service);
            self.add_log(
                service,
                &format!("Serviço {} parado.", service.display_name()),
                false,
            );
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn stop_all(&mut self) {
        let services = vec![ServiceType::Apache, ServiceType::Database, ServiceType::Php];
        for s in services {
            let _ = self.stop_service(s);
        }
    }

    fn locate_binary(&self, service: ServiceType, version: &str) -> Result<PathBuf, String> {
        let dir = self
            .base_bin_dir
            .join(service.folder_name())
            .join(version);

        #[cfg(windows)]
        let candidates = match service {
            ServiceType::Apache => vec!["bin/httpd.exe", "httpd.exe"],
            ServiceType::Database => vec!["bin/mysqld.exe", "mysqld.exe", "bin/mariadbd.exe"],
            ServiceType::Php => vec!["php-cgi.exe", "php.exe"],
        };

        #[cfg(not(windows))]
        let candidates = match service {
            ServiceType::Apache => vec!["bin/httpd", "bin/apache2", "httpd", "apache2"],
            ServiceType::Database => vec!["bin/mysqld", "bin/mariadbd", "mysqld", "mariadbd"],
            ServiceType::Php => vec!["sbin/php-fpm", "bin/php", "php-fpm", "php"],
        };

        for candidate in candidates {
            let path = dir.join(candidate);
            if path.exists() {
                return Ok(path);
            }
        }

        // On Linux/Unix, if no local binary in bin folder, check system paths as fallback
        #[cfg(not(windows))]
        {
            let sys_binary = match service {
                ServiceType::Apache => "apache2",
                ServiceType::Database => "mariadbd",
                ServiceType::Php => "php",
            };
            if let Ok(output) = Command::new("which").arg(sys_binary).output() {
                if output.status.success() {
                    let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path_str.is_empty() {
                        return Ok(PathBuf::from(path_str));
                    }
                }
            }
        }

        Err(format!(
            "Binário para {} (versão {}) não foi encontrado em {:?}",
            service.display_name(),
            version,
            dir
        ))
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
