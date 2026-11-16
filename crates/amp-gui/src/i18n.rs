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
        pt.insert("no_version_installed", "Nenhuma versão instalada");

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
        pt.insert("download_and_install", "Descarregar & Instalar");
        pt.insert("open_in_browser", "Abrir no Navegador");
        pt.insert("installed", "Instalado");
        pt.insert("activate", "Ativar");
        pt.insert("downloading", "A transferir...");
        pt.insert("extracting", "A extrair ficheiros...");
        pt.insert("download_complete", "Instalação concluída!");

        // Filters
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

        // PHP Embedded Server
        pt.insert("php_server_title", "Servidores PHP Embutidos (Built-in Server)");
        pt.insert("add_php_server", "+ Novo Servidor PHP");
        pt.insert("server_name", "Nome da Instância");
        pt.insert("doc_root", "Diretório Raiz");
        pt.insert("host", "Host / IP");
        pt.insert("embedded_php_info", "💡 O PHP possui um servidor web embutido (php -S) ideal para desenvolvimento rápido de múltiplos sites ou APIs sem necessidade do Apache.");

        // Settings
        pt.insert("language", "Idioma");
        pt.insert("theme", "Tema Visual");
        pt.insert("theme_dark", "Escuro");
        pt.insert("theme_light", "Claro");
        pt.insert("root_dir", "Diretório Raiz dos Projetos (www)");
        pt.insert("bin_dir", "Diretório de Binários (bin)");
        pt.insert("logs_dir", "Diretório de Logs (logs)");
        pt.insert("ports_config", "Configuração de Portas de Monitorização");
        pt.insert("remote_manifest_url", "URL do Catálogo Remoto (JSON)");
        pt.insert("browse", "Procurar...");
        pt.insert("save_settings", "💾 Guardar Configurações");
        pt.insert("reset_defaults", "🔄 Repor Valores por Omissão");
        pt.insert("settings_saved", "Configurações guardadas com sucesso!");

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
        en.insert("no_version_installed", "No version installed");

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
        en.insert("download_and_install", "Download & Install");
        en.insert("open_in_browser", "Open in Browser");
        en.insert("installed", "Installed");
        en.insert("activate", "Activate");
        en.insert("downloading", "Downloading...");
        en.insert("extracting", "Extracting files...");
        en.insert("download_complete", "Installation completed!");

        // Filters
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

        // PHP Embedded Server
        en.insert("php_server_title", "PHP Embedded Web Servers (Built-in Server)");
        en.insert("add_php_server", "+ New PHP Server");
        en.insert("server_name", "Instance Name");
        en.insert("doc_root", "Document Root");
        en.insert("host", "Host / IP");
        en.insert("embedded_php_info", "💡 PHP has a built-in web server (php -S) great for lightweight, rapid development of multiple sites or APIs without Apache.");

        // Settings
        en.insert("language", "Language");
        en.insert("theme", "Theme");
        en.insert("theme_dark", "Dark");
        en.insert("theme_light", "Light");
        en.insert("root_dir", "Root Web Directory (www)");
        en.insert("bin_dir", "Binaries Directory (bin)");
        en.insert("logs_dir", "Logs Directory (logs)");
        en.insert("ports_config", "Monitored Ports Configuration");
        en.insert("remote_manifest_url", "Remote Manifest URL (JSON)");
        en.insert("browse", "Browse...");
        en.insert("save_settings", "💾 Save Settings");
        en.insert("reset_defaults", "🔄 Reset to Defaults");
        en.insert("settings_saved", "Settings saved successfully!");

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

    pub fn set_language(&mut self, lang: &str) {
        self.current_lang = lang.to_string();
    }

    pub fn current_language(&self) -> &str {
        &self.current_lang
    }
}
