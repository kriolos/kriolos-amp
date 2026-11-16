use crate::models::{PhpServerInstance, ServiceInfo, ServiceStatus, ServiceType};
use crate::port_checker::PortChecker;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub struct LogMessage {
    pub service_label: String,
    pub timestamp: String,
    pub message: String,
    pub is_error: bool,
}

pub struct ServiceManager {
    base_bin_dir: PathBuf,
    active_processes: HashMap<ServiceType, Child>,
    active_ports: HashMap<ServiceType, u16>,
    service_errors: HashMap<ServiceType, String>,
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

    pub fn add_log(&self, label: &str, msg: &str, is_error: bool) {
        let now = chrono_lite();
        let log_entry = LogMessage {
            service_label: label.to_string(),
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

        self.service_errors.remove(&service);

        if PortChecker::is_port_in_use(port) {
            let err = format!(
                "Porta {} já ocupada por outro processo ou serviço!",
                port
            );
            self.service_errors.insert(service, err.clone());
            self.add_log(service.display_name(), &format!("⚠ {}", err), true);
            return Err(err);
        }

        let bin_path = match self.locate_binary(service, version) {
            Ok(p) => p,
            Err(e) => {
                self.service_errors.insert(service, e.clone());
                self.add_log(service.display_name(), &format!("⚠ {}", e), true);
                return Err(e);
            }
        };

        self.add_log(
            service.display_name(),
            &format!("Iniciando {} (v{}) na porta {}...", service.display_name(), version, port),
            false,
        );

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
                self.add_log(
                    service.display_name(),
                    &format!("Iniciado com sucesso (PID: {}) na porta {}", pid, port),
                    false,
                );
                Ok(())
            }
            Err(e) => {
                let err = format!("Falha de execução de {}: {}", service.display_name(), e);
                self.service_errors.insert(service, err.clone());
                self.add_log(service.display_name(), &format!("⚠ {}", err), true);
                Err(err)
            }
        }
    }

    pub fn stop_service(&mut self, service: ServiceType) -> Result<(), String> {
        self.service_errors.remove(&service);
        if let Some(mut child) = self.active_processes.remove(&service) {
            self.add_log(
                service.display_name(),
                &format!("A parar serviço {}...", service.display_name()),
                false,
            );
            let _ = child.kill();
            let _ = child.wait();
            self.active_ports.remove(&service);
            self.add_log(
                service.display_name(),
                &format!("Serviço {} parado.", service.display_name()),
                false,
            );
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
    ) -> Result<(), String> {
        if self.php_server_processes.contains_key(&instance.id) {
            return Ok(());
        }

        if PortChecker::is_port_in_use(instance.port) {
            let err = format!("Porta {} já ocupada para '{}'!", instance.port, instance.name);
            self.add_log(&instance.name, &format!("⚠ {}", err), true);
            return Err(err);
        }

        let php_bin = match self.locate_php_cli(php_version) {
            Ok(p) => p,
            Err(e) => {
                self.add_log(&instance.name, &format!("⚠ {}", e), true);
                return Err(e);
            }
        };

        let bind_addr = format!("{}:{}", instance.host, instance.port);

        self.add_log(
            &instance.name,
            &format!(
                "Iniciando PHP Embedded Server em http://{} (Root: {})...",
                bind_addr, instance.doc_root
            ),
            false,
        );

        let mut cmd = Command::new(&php_bin);
        cmd.args(["-S", &bind_addr, "-t", &instance.doc_root]);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                self.php_server_processes
                    .insert(instance.id.clone(), (child, instance.port));
                self.add_log(
                    &instance.name,
                    &format!("Servidor PHP ativo em http://{} (PID: {})", bind_addr, pid),
                    false,
                );
                Ok(())
            }
            Err(e) => {
                let err = format!("Erro ao iniciar servidor PHP '{}': {}", instance.name, e);
                self.add_log(&instance.name, &format!("⚠ {}", err), true);
                Err(err)
            }
        }
    }

    pub fn stop_php_server(&mut self, instance_id: &str) -> Result<(), String> {
        if let Some((mut child, _)) = self.php_server_processes.remove(instance_id) {
            let _ = child.kill();
            let _ = child.wait();
            self.add_log(instance_id, "Servidor PHP embutido parado.", false);
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

    fn locate_binary(&self, service: ServiceType, version: &str) -> Result<PathBuf, String> {
        let dir = self
            .base_bin_dir
            .join(service.folder_name())
            .join(version);

        #[cfg(windows)]
        let candidates = match service {
            ServiceType::Apache => vec!["bin/httpd.exe", "httpd.exe"],
            ServiceType::Php => vec!["php.exe", "php-cgi.exe"],
            ServiceType::Mysql => vec!["bin/mysqld.exe", "mysqld.exe"],
            ServiceType::Mariadb => vec!["bin/mariadbd.exe", "bin/mysqld.exe", "mariadbd.exe"],
            ServiceType::Postgresql => vec!["bin/postgres.exe", "postgres.exe"],
            ServiceType::Tool => vec![],
        };

        #[cfg(not(windows))]
        let candidates = match service {
            ServiceType::Apache => vec!["bin/httpd", "bin/apache2", "httpd", "apache2"],
            ServiceType::Php => vec!["bin/php", "sbin/php-fpm", "php", "php-fpm"],
            ServiceType::Mysql => vec!["bin/mysqld", "mysqld"],
            ServiceType::Mariadb => vec!["bin/mariadbd", "bin/mysqld", "mariadbd", "mysqld"],
            ServiceType::Postgresql => vec!["bin/postgres", "postgres"],
            ServiceType::Tool => vec![],
        };

        for candidate in candidates {
            let path = dir.join(candidate);
            if path.exists() {
                return Ok(path);
            }
        }

        // On Linux fallback to system binaries
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

        Err(format!(
            "Binário não encontrado em '{}'. Descarregue em 'Versões & Downloads' ou instale no sistema.",
            dir.display()
        ))
    }

    fn locate_php_cli(&self, version: Option<&str>) -> Result<PathBuf, String> {
        if let Some(ver) = version {
            let dir = self.base_bin_dir.join("php").join(ver);
            #[cfg(windows)]
            let candidates = ["php.exe"];
            #[cfg(not(windows))]
            let candidates = ["bin/php", "php"];

            for c in candidates {
                let p = dir.join(c);
                if p.exists() {
                    return Ok(p);
                }
            }
        }

        // Fallback to system php
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

        Err("Binário do PHP CLI não encontrado. Descarregue o PHP em 'Versões & Downloads'.".to_string())
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
