# ⚡ AMP Control (Rust + egui)

Um painel de controlo moderno, leve e multiplataforma para ambientes **AMP** (**Apache** + **MariaDB/MySQL** + **PHP**), desenvolvido em **Rust** utilizando **egui** (`eframe`).

Inspirado no UwAmp, mas reimaginado como uma aplicação **nativa** para **Linux**, **Windows** e **macOS**, eliminando a dependência do Wine ou APIs exclusivas do Windows.

---

## 🌟 Funcionalidades

- **Dashboard Moderno**:
  - Cards de estado em tempo real para Apache, MariaDB/MySQL e PHP.
  - Indicadores de processo (PID, portas ativas e verificação de conflito de portas).
  - Ações rápidas de 1 clique: Iniciar/Parar/Reiniciar tudo, abrir pasta `www`, `phpMyAdmin` e `phpinfo()`.
- **Gestor de Versões & Downloads (Package Manager)**:
  - Catálogo de versões integrado (`assets/versions.json`) com suporte a consulta remota (GitHub Releases / HTTP).
  - Download assíncrono com barra de progresso suave, cálculo de percentagem e velocidade.
  - Extração automática de arquivos compactados (`.zip` e `.tar.gz`) para pastas de binários.
  - Alternador dinâmico de versão ativa.
- **Internacionalização (i18n)**:
  - Suporte completo a múltiplos idiomas (Português `pt-PT` e Inglês `en-US`).
  - Alternância instantânea de idioma diretamente na interface sem reiniciar.
- **Configurações & Extensões**:
  - Seleção visual de extensões do PHP (`curl`, `pdo_mysql`, `mbstring`, `openssl`, etc.).
  - Atalhos para edição de ficheiros de configuração (`httpd.conf`, `php.ini`, `my.ini`).
  - Suporte a templates com substituição de variáveis portáteis (`{ROOT_DIR}`, etc.).
- **Logs em Tempo Real**:
  - Visualizador de logs com rolagem automática, código de cores para erros e filtro de pesquisa instantâneo.
- **Temas**:
  - Suporte a tema Escuro (*Dark Mode*) e Claro (*Light Mode*).

---

## 🏗️ Estrutura do Projeto

```
amp-control/
├── Cargo.toml                 # Workspace Cargo
├── .gitignore
├── assets/
│   └── versions.json          # Catálogo de versões e downloads
├── crates/
│   ├── amp-core/              # Biblioteca Core do sistema
│   │   ├── src/
│   │   │   ├── models.rs      # Modelos de dados, serviços e pacotes
│   │   │   ├── port_checker.rs# Verificação de portas e sockets TCP
│   │   │   ├── package_manager.rs # Downloader assíncrono e extrator
│   │   │   ├── service_manager.rs # Gestão do ciclo de vida dos processos
│   │   │   └── template_engine.rs # Motor de templates portáteis
│   └── amp-gui/               # Interface Gráfica com egui / eframe
│       ├── src/
│       │   ├── app.rs         # Ciclo de vida e navegação da UI
│       │   ├── i18n.rs        # Sistema de internacionalização
│       │   ├── views/         # Visualizações modulares
│       │   │   ├── dashboard.rs
│       │   │   ├── versions.rs
│       │   │   ├── vhosts.rs
│       │   │   ├── configs.rs
│       │   │   ├── logs.rs
│       │   │   └── settings.rs
│       │   └── main.rs        # Ponto de entrada nativo
└── www/                       # Diretório raiz padrão dos sites
    ├── index.php
    └── phpinfo.php
```

---

## 🚀 Como Executar

### Pré-requisitos
- [Rust](https://rustup.rs/) (v1.80+)

### Compilar e Rodar
```bash
cd amp-control
cargo run -p amp-gui
```
