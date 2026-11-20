use amp_core::models::{LogEvent, ServiceError};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct I18n {
    current_lang: String,
    translations: HashMap<String, HashMap<&'static str, &'static str>>,
}

impl I18n {
    pub fn new(lang: &str) -> Self {
        let mut translations = HashMap::new();

        let mut pt = HashMap::new();
        pt.insert("app_title", "AMP Control");
        pt.insert("dashboard", "Painel Principal");
        pt.insert("versions", "Versões & Downloads");
        pt.insert("php_servers", "Servidores PHP");
        pt.insert("vhosts", "Virtual Hosts");
        pt.insert("configs", "Configurações & Módulos");
        pt.insert("logs", "Logs em Tempo Real");
        pt.insert("settings", "Definições");

        pt.insert("start_all", "Iniciar Todos");
        pt.insert("stop_all", "Parar Todos");
        pt.insert("restart", "Reiniciar");
        pt.insert("start", "Iniciar");
        pt.insert("stop", "Parar");
        pt.insert("status", "Estado");
        pt.insert("port", "Porta");
        pt.insert("version", "Versão");
        pt.insert("active_version", "Versão Ativa");
        pt.insert("no_version_installed", "Nenhum binário disponível");

        pt.insert("online", "ONLINE");
        pt.insert("offline", "PARADO");
        pt.insert("starting", "INICIANDO...");
        pt.insert("error", "ERRO");

        pt.insert("quick_actions", "Acesso Rápido");
        pt.insert("open_www", "Abrir Pasta www");
        pt.insert("open_localhost", "Abrir Localhost");
        pt.insert("open_phpmyadmin", "Abrir phpMyAdmin");
        pt.insert("open_phpinfo", "Ver phpinfo()");

        pt.insert("ports_status", "Estado das Portas Monitorizadas");
        pt.insert("port_available", "Disponível");
        pt.insert("port_in_use", "Em Uso / Ativa");

        pt.insert("download_catalog", "Catálogo de Serviços Disponíveis");
        pt.insert("refresh_catalog", "Atualizar Catálogo");
        pt.insert("download_and_install", "Descarregar");
        pt.insert("redownload", "Re-descarregar");
        pt.insert("open_in_browser", "Abrir no Navegador");
        pt.insert("downloaded", "Baixado");
        pt.insert("source_downloaded", "Baixado");
        pt.insert("source_system", "Sistema");
        pt.insert("activate", "Ativar");
        pt.insert("downloading", "A transferir...");
        pt.insert("extracting", "A extrair ficheiros...");
        pt.insert("download_complete", "Download concluído com sucesso!");

        // Filters & Catalog
        pt.insert("service_label", "Serviço");
        pt.insert("type_label", "Tipo");
        pt.insert("filter_all", "Todos");
        pt.insert("filter_php", "PHP");
        pt.insert("filter_apache", "Apache");
        pt.insert("filter_mariadb", "MariaDB");
        pt.insert("filter_mysql", "MySQL");
        pt.insert("filter_postgresql", "PostgreSQL");
        pt.insert("filter_tools", "Ferramentas & VC++");
        pt.insert("filter_kind_all", "Todos os Tipos");
        pt.insert("filter_kind_binary", "🚀 Apenas Binários");
        pt.insert("filter_kind_source", "📦 Apenas Código-fonte");
        pt.insert("os_label", "Sistema");
        pt.insert("filter_os_all", "Todos");
        pt.insert("filter_os_current", "💻 SO Atual");
        pt.insert("filter_os_linux", "🐧 Linux");
        pt.insert("filter_os_windows", "🪟 Windows");
        pt.insert("filter_os_macos", "🍎 macOS");
        pt.insert("badge_binary", "🚀 Binário");
        pt.insert("badge_source", "📦 Fonte");
        pt.insert("recommended", "Recomendado");
        pt.insert("no_packages_found", "Nenhum pacote encontrado com os filtros selecionados.");
        pt.insert("loading_catalog", "A carregar catálogo de versões...");

        // PHP Embedded Server
        pt.insert("php_server_title", "Servidores PHP Embutidos (Built-in Server)");
        pt.insert("add_php_server", "+ Novo Servidor PHP");
        pt.insert("server_name", "Nome da Instância");
        pt.insert("doc_root", "Diretório Raiz");
        pt.insert("host", "Host / IP");
        pt.insert("embedded_php_info", "💡 O PHP possui um servidor web embutido (php -S) ideal para desenvolvimento rápido de múltiplos sites ou APIs sem necessidade do Apache.");
        pt.insert("static_php_recommendation", "💡 Dica: Se o PHP do sistema falhar por falta de dependências partilhadas, utilize o PHP Estático autónomo (SPC).");
        pt.insert("download_static_php", "📥 Obter PHP Estático (SPC)");
        pt.insert("create_instance", "Criar Instância");
        pt.insert("remove", "Remover");
        pt.insert("port_busy", "Porta Ocupada");

        // VHosts
        pt.insert("add_vhost", "+ Adicionar Novo VHost");
        pt.insert("vhosts_info", "💡 Permite mapear domínios locais personalizados para pastas de projeto (ex.: meuprojeto.local).");

        // Configs & Modules
        pt.insert("php_extensions_title", "Extensões PHP Comuns");
        pt.insert("edit_configs_title", "Editar Ficheiros de Configuração");
        pt.insert("edit_apache_conf", "📝 Editar httpd.conf (Apache)");
        pt.insert("edit_php_ini", "📝 Editar php.ini (PHP)");
        pt.insert("edit_mysql_ini", "📝 Editar my.ini (MySQL)");
        pt.insert("configs_info", "💡 As configurações são geradas dinamicamente com variáveis de ambiente portáteis ({ROOT_DIR}, etc.).");

        // Settings
        pt.insert("language", "Idioma");
        pt.insert("theme", "Tema Visual");
        pt.insert("theme_dark", "Escuro");
        pt.insert("theme_light", "Claro");
        pt.insert("general_appearance", "Geral & Aparência");
        pt.insert("directory_paths", "Caminhos de Diretórios");
        pt.insert("root_dir", "Diretório Raiz dos Projetos (www)");
        pt.insert("bin_dir", "Diretório de Binários (bin)");
        pt.insert("logs_dir", "Diretório de Logs (logs)");
        pt.insert("default_host", "Host / IP Padrão");
        pt.insert("ports_config", "Configuração de Rede e Portas");
        pt.insert("remote_manifest_url", "URL do Catálogo Remoto (JSON)");
        pt.insert("remote_manifest_info", "💡 O catálogo remoto permite carregar novas versões de PHP, Apache e Bancos sem precisar recompilar ou atualizar o programa.");
        pt.insert("browse", "Procurar...");
        pt.insert("save_settings", "💾 Guardar Configurações");
        pt.insert("reset_defaults", "🔄 Repor Valores por Omissão");
        pt.insert("settings_saved", "Configurações guardadas com sucesso!");
        pt.insert("defaults_restored", "Valores por omissão restaurados!");

        // Logs
        pt.insert("filter_logs_hint", "Filtrar logs...");
        pt.insert("no_logs_available", "Nenhum registo de log disponível no momento.");

        // Status bar
        pt.insert("system_ready", "Sistema pronto e inicializado.");
        pt.insert("view_logs", "Ver Logs");
        pt.insert("all_stopped", "Todos os serviços foram parados.");
        pt.insert("all_started", "Todos os serviços foram iniciados com sucesso.");

        translations.insert("pt-PT".to_string(), pt);

        let mut en = HashMap::new();
        en.insert("app_title", "AMP Control");
        en.insert("dashboard", "Dashboard");
        en.insert("versions", "Versions & Downloads");
        en.insert("php_servers", "PHP Servers");
        en.insert("vhosts", "Virtual Hosts");
        en.insert("configs", "Configs & Modules");
        en.insert("logs", "Live Logs");
        en.insert("settings", "Settings");

        en.insert("start_all", "Start All");
        en.insert("stop_all", "Stop All");
        en.insert("restart", "Restart");
        en.insert("start", "Start");
        en.insert("stop", "Stop");
        en.insert("status", "Status");
        en.insert("port", "Port");
        en.insert("version", "Version");
        en.insert("active_version", "Active Version");
        en.insert("no_version_installed", "No binary available");

        en.insert("online", "ONLINE");
        en.insert("offline", "STOPPED");
        en.insert("starting", "STARTING...");
        en.insert("error", "ERROR");

        en.insert("quick_actions", "Quick Actions");
        en.insert("open_www", "Open www Folder");
        en.insert("open_localhost", "Open Localhost");
        en.insert("open_phpmyadmin", "Open phpMyAdmin");
        en.insert("open_phpinfo", "View phpinfo()");

        en.insert("ports_status", "Monitored Ports Status");
        en.insert("port_available", "Available");
        en.insert("port_in_use", "In Use / Active");

        en.insert("download_catalog", "Available Services Catalog");
        en.insert("refresh_catalog", "Refresh Catalog");
        en.insert("download_and_install", "Download");
        en.insert("redownload", "Re-download");
        en.insert("open_in_browser", "Open in Browser");
        en.insert("downloaded", "Downloaded");
        en.insert("source_downloaded", "Downloaded");
        en.insert("source_system", "System");
        en.insert("activate", "Activate");
        en.insert("downloading", "Downloading...");
        en.insert("extracting", "Extracting files...");
        en.insert("download_complete", "Download completed successfully!");

        // Filters & Catalog
        en.insert("service_label", "Service");
        en.insert("type_label", "Type");
        en.insert("filter_all", "All");
        en.insert("filter_php", "PHP");
        en.insert("filter_apache", "Apache");
        en.insert("filter_mariadb", "MariaDB");
        en.insert("filter_mysql", "MySQL");
        en.insert("filter_postgresql", "PostgreSQL");
        en.insert("filter_tools", "Tools & VC++");
        en.insert("filter_kind_all", "All Types");
        en.insert("filter_kind_binary", "🚀 Binaries Only");
        en.insert("filter_kind_source", "📦 Source Only");
        en.insert("os_label", "OS");
        en.insert("filter_os_all", "All");
        en.insert("filter_os_current", "💻 Current OS");
        en.insert("filter_os_linux", "🐧 Linux");
        en.insert("filter_os_windows", "🪟 Windows");
        en.insert("filter_os_macos", "🍎 macOS");
        en.insert("badge_binary", "🚀 Binary");
        en.insert("badge_source", "📦 Source");
        en.insert("recommended", "Recommended");
        en.insert("no_packages_found", "No packages found matching the selected filters.");
        en.insert("loading_catalog", "Loading versions catalog...");

        // PHP Embedded Server
        en.insert("php_server_title", "PHP Embedded Web Servers (Built-in Server)");
        en.insert("add_php_server", "+ New PHP Server");
        en.insert("server_name", "Instance Name");
        en.insert("doc_root", "Document Root");
        en.insert("host", "Host / IP");
        en.insert("embedded_php_info", "💡 PHP has a built-in web server (php -S) great for lightweight, rapid development of multiple sites or APIs without Apache.");
        en.insert("static_php_recommendation", "💡 Tip: If system PHP fails due to missing shared libraries, use the standalone Static PHP (SPC).");
        en.insert("download_static_php", "📥 Get Static PHP (SPC)");
        en.insert("create_instance", "Create Instance");
        en.insert("remove", "Remove");
        en.insert("port_busy", "Port In Use");

        // VHosts
        en.insert("add_vhost", "+ Add New VHost");
        en.insert("vhosts_info", "💡 Map custom local domains to project folders (e.g., myproject.local).");

        // Configs & Modules
        en.insert("php_extensions_title", "Common PHP Extensions");
        en.insert("edit_configs_title", "Edit Configuration Files");
        en.insert("edit_apache_conf", "📝 Edit httpd.conf (Apache)");
        en.insert("edit_php_ini", "📝 Edit php.ini (PHP)");
        en.insert("edit_mysql_ini", "📝 Edit my.ini (MySQL)");
        en.insert("configs_info", "💡 Configurations are dynamically generated with portable environment variables ({ROOT_DIR}, etc.).");

        // Settings
        en.insert("language", "Language");
        en.insert("theme", "Theme");
        en.insert("theme_dark", "Dark");
        en.insert("theme_light", "Light");
        en.insert("general_appearance", "General & Appearance");
        en.insert("directory_paths", "Directory Paths");
        en.insert("root_dir", "Root Web Directory (www)");
        en.insert("bin_dir", "Binaries Directory (bin)");
        en.insert("logs_dir", "Logs Directory (logs)");
        en.insert("default_host", "Default Host / IP");
        en.insert("ports_config", "Network & Ports Configuration");
        en.insert("remote_manifest_url", "Remote Manifest URL (JSON)");
        en.insert("remote_manifest_info", "💡 The remote catalog lets you load new PHP, Apache, and Database versions without recompiling the app.");
        en.insert("browse", "Browse...");
        en.insert("save_settings", "💾 Save Settings");
        en.insert("reset_defaults", "🔄 Reset to Defaults");
        en.insert("settings_saved", "Settings saved successfully!");
        en.insert("defaults_restored", "Defaults restored successfully!");

        // Logs
        en.insert("filter_logs_hint", "Filter logs...");
        en.insert("no_logs_available", "No log records available at the moment.");

        // Status bar
        en.insert("system_ready", "System ready and initialized.");
        en.insert("view_logs", "View Logs");
        en.insert("all_stopped", "All services were stopped.");
        en.insert("all_started", "All services started successfully.");

        translations.insert("en-US".to_string(), en);

        Self {
            current_lang: lang.to_string(),
            translations,
        }
    }

    pub fn t<'a>(&self, key: &'a str) -> &'a str {
        if let Some(map) = self.translations.get(&self.current_lang) {
            if let Some(&text) = map.get(key) {
                return text;
            }
        }
        if let Some(map) = self.translations.get("en-US") {
            if let Some(&text) = map.get(key) {
                return text;
            }
        }
        key
    }

    pub fn format_error(&self, err: &ServiceError) -> String {
        let is_pt = self.current_lang == "pt-PT";
        match err {
            ServiceError::PortInUse { port } => {
                if is_pt {
                    format!("Porta {} já ocupada por outro processo ou serviço!", port)
                } else {
                    format!("Port {} is already in use by another process or service!", port)
                }
            }
            ServiceError::BinaryNotFound { service, path } => {
                if is_pt {
                    format!(
                        "Binário para {} não foi encontrado em '{}' nem no sistema.",
                        service.display_name(), path
                    )
                } else {
                    format!(
                        "Binary for {} was not found at '{}' nor on the system.",
                        service.display_name(), path
                    )
                }
            }
            ServiceError::PhpCliNotFound => {
                if is_pt {
                    "Binário do PHP CLI não encontrado. Descarregue em 'Versões & Downloads'.".to_string()
                } else {
                    "PHP CLI binary not found. Download it in 'Versions & Downloads'.".to_string()
                }
            }
            ServiceError::ExecutionFailed { service, details } => {
                if is_pt {
                    format!("Falha ao iniciar {}: {}", service.display_name(), details)
                } else {
                    format!("Failed to start {}: {}", service.display_name(), details)
                }
            }
            ServiceError::PhpServerFailed { name, details } => {
                if is_pt {
                    format!("Falha no servidor PHP '{}': {}", name, details)
                } else {
                    format!("PHP server '{}' failure: {}", name, details)
                }
            }
        }
    }

    pub fn format_log(&self, event: &LogEvent) -> (String, String, bool) {
        // Returns (service_label, formatted_message, is_error)
        let is_pt = self.current_lang == "pt-PT";
        match event {
            LogEvent::Starting { service, port } => {
                let msg = if is_pt {
                    format!("A iniciar {} na porta {}...", service.display_name(), port)
                } else {
                    format!("Starting {} on port {}...", service.display_name(), port)
                };
                (service.badge_label().to_string(), msg, false)
            }
            LogEvent::Started { service, pid, port } => {
                let msg = if is_pt {
                    format!("Iniciado com sucesso (PID: {}) na porta {}", pid, port)
                } else {
                    format!("Started successfully (PID: {}) on port {}", pid, port)
                };
                (service.badge_label().to_string(), msg, false)
            }
            LogEvent::Stopping { service } => {
                let msg = if is_pt {
                    format!("A parar serviço {}...", service.display_name())
                } else {
                    format!("Stopping service {}...", service.display_name())
                };
                (service.badge_label().to_string(), msg, false)
            }
            LogEvent::Stopped { service } => {
                let msg = if is_pt {
                    format!("Serviço {} parado.", service.display_name())
                } else {
                    format!("Service {} stopped.", service.display_name())
                };
                (service.badge_label().to_string(), msg, false)
            }
            LogEvent::StartingPhpServer { name, host, port, doc_root } => {
                let msg = if is_pt {
                    format!("A iniciar servidor PHP '{}' em http://{}:{} (Root: {})...", name, host, port, doc_root)
                } else {
                    format!("Starting PHP server '{}' at http://{}:{} (Root: {})...", name, host, port, doc_root)
                };
                ("PHP".to_string(), msg, false)
            }
            LogEvent::PhpServerStarted { name, host, port, pid } => {
                let msg = if is_pt {
                    format!("Servidor PHP '{}' ativo em http://{}:{} (PID: {})", name, host, port, pid)
                } else {
                    format!("PHP server '{}' running at http://{}:{} (PID: {})", name, host, port, pid)
                };
                ("PHP".to_string(), msg, false)
            }
            LogEvent::PhpServerStopped { name } => {
                let msg = if is_pt {
                    format!("Servidor PHP '{}' parado.", name)
                } else {
                    format!("PHP server '{}' stopped.", name)
                };
                ("PHP".to_string(), msg, false)
            }
            LogEvent::Error(err) => {
                let label = match err {
                    ServiceError::ExecutionFailed { service, .. } | ServiceError::BinaryNotFound { service, .. } => service.badge_label().to_string(),
                    _ => "SYSTEM".to_string(),
                };
                (label, self.format_error(err), true)
            }
            LogEvent::DownloadStarted { item_id, url } => {
                let msg = if is_pt {
                    format!("A iniciar download de '{}' a partir de: {}", item_id, url)
                } else {
                    format!("Starting download of '{}' from: {}", item_id, url)
                };
                ("DOWNLOAD".to_string(), msg, false)
            }
            LogEvent::DownloadFailed { item_id, url, error } => {
                let msg = if is_pt {
                    format!("⚠ Falha no download de '{}' (URL: {}): {}", item_id, url, error)
                } else {
                    format!("⚠ Download failed for '{}' (URL: {}): {}", item_id, url, error)
                };
                ("DOWNLOAD".to_string(), msg, true)
            }
            LogEvent::DownloadCompleted { item_id, url } => {
                let msg = if is_pt {
                    format!("Download de '{}' concluído com sucesso a partir de: {}", item_id, url)
                } else {
                    format!("Download of '{}' completed successfully from: {}", item_id, url)
                };
                ("DOWNLOAD".to_string(), msg, false)
            }
            LogEvent::CommandExecuted { command } => {
                ("EXEC".to_string(), command.clone(), false)
            }
            LogEvent::ProcessOutput { source, line, is_stderr } => {
                (source.clone(), line.clone(), *is_stderr)
            }
        }
    }

    pub fn set_language(&mut self, lang: &str) {
        self.current_lang = lang.to_string();
    }

    pub fn current_language(&self) -> &str {
        &self.current_lang
    }
}
