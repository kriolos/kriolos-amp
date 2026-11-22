use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::Duration;
use crate::models::{ServiceError, ServiceType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DbEngine {
    MariaDb,
    Mysql,
    Postgresql,
}

impl DbEngine {
    pub fn folder_name(&self) -> &'static str {
        match self {
            DbEngine::MariaDb => "mariadb",
            DbEngine::Mysql => "mysql",
            DbEngine::Postgresql => "postgresql",
        }
    }

    pub fn to_service_type(&self) -> ServiceType {
        match self {
            DbEngine::MariaDb => ServiceType::Mariadb,
            DbEngine::Mysql => ServiceType::Mysql,
            DbEngine::Postgresql => ServiceType::Postgresql,
        }
    }

    pub fn from_service_type(service: ServiceType) -> Option<Self> {
        match service {
            ServiceType::Mariadb => Some(DbEngine::MariaDb),
            ServiceType::Mysql => Some(DbEngine::Mysql),
            ServiceType::Postgresql => Some(DbEngine::Postgresql),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DbStartupOptions<'a> {
    pub engine: DbEngine,
    pub version: &'a str,
    pub bin_path: &'a Path,
    pub port: u16,
    pub bind_address: &'a str,
    pub base_data_dir: &'a Path,
}

pub struct DbSupervisor;

impl DbSupervisor {
    /// Resolves the dedicated data directory for this database engine.
    /// By default, stores in `data/<engine>` (e.g. `data/mariadb`, `data/postgresql`).
    pub fn resolve_data_dir(engine: DbEngine, _version: &str, base_data_dir: &Path) -> PathBuf {
        base_data_dir.join(engine.folder_name())
    }

    /// Finds the base root installation directory of the database package.
    pub fn find_basedir(bin_path: &Path) -> PathBuf {
        let parent = bin_path.parent().unwrap_or_else(|| Path::new("."));
        if parent.file_name().map(|n| n == "bin").unwrap_or(false) {
            parent.parent().unwrap_or(parent).to_path_buf()
        } else {
            parent.to_path_buf()
        }
    }

    /// Checks if the data directory has been initialized on first run;
    /// if not, runs the appropriate initialization utility.
    pub fn ensure_data_initialized(
        engine: DbEngine,
        bin_path: &Path,
        data_dir: &Path,
    ) -> Result<(), ServiceError> {
        match engine {
            DbEngine::MariaDb | DbEngine::Mysql => {
                let marker = data_dir.join("mysql");
                if marker.exists() {
                    return Ok(());
                }

                std::fs::create_dir_all(data_dir).map_err(|e| ServiceError::ExecutionFailed {
                    service: engine.to_service_type(),
                    details: format!("Failed to create data dir {}: {}", data_dir.display(), e),
                })?;

                let basedir = Self::find_basedir(bin_path);

                // Try finding mariadb-install-db / mysql_install_db
                #[cfg(windows)]
                let candidates = [
                    bin_path.with_file_name("mariadb-install-db.exe"),
                    basedir.join("bin").join("mariadb-install-db.exe"),
                    basedir.join("mariadb-install-db.exe"),
                    bin_path.with_file_name("mysql_install_db.exe"),
                ];

                #[cfg(not(windows))]
                let candidates = [
                    basedir.join("scripts").join("mariadb-install-db"),
                    bin_path.with_file_name("mariadb-install-db"),
                    basedir.join("bin").join("mariadb-install-db"),
                    basedir.join("scripts").join("mysql_install_db"),
                    bin_path.with_file_name("mysql_install_db"),
                ];

                let mut ran_tool = false;
                for tool in &candidates {
                    if tool.is_file() {
                        let mut cmd = Command::new(tool);

                        #[cfg(windows)]
                        {
                            use std::os::windows::process::CommandExt;
                            const CREATE_NO_WINDOW: u32 = 0x08000000;
                            cmd.creation_flags(CREATE_NO_WINDOW);
                            cmd.arg(format!("--datadir={}", data_dir.display()))
                               .arg("--default-user");
                        }

                        #[cfg(not(windows))]
                        {
                            cmd.arg(format!("--basedir={}", basedir.display()))
                               .arg(format!("--datadir={}", data_dir.display()))
                               .arg("--auth-root-authentication-method=normal");
                        }

                        let output = cmd.output().map_err(|e| ServiceError::ExecutionFailed {
                            service: engine.to_service_type(),
                            details: format!("Failed to run DB install tool {}: {}", tool.display(), e),
                        })?;

                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            return Err(ServiceError::ExecutionFailed {
                                service: engine.to_service_type(),
                                details: format!(
                                    "DB initialization tool failed: {}\n{}",
                                    stderr.trim(),
                                    stdout.trim()
                                ),
                            });
                        }

                        ran_tool = true;
                        break;
                    }
                }

                // Fallback for Oracle MySQL 5.7+ / 8.0+ / 8.4+: mysqld --initialize-insecure
                if !ran_tool {
                    let mut cmd = Command::new(bin_path);
                    cmd.arg("--no-defaults")
                        .arg(format!("--basedir={}", basedir.display()))
                        .arg(format!("--datadir={}", data_dir.display()))
                        .arg("--initialize-insecure");

                    #[cfg(windows)]
                    {
                        use std::os::windows::process::CommandExt;
                        const CREATE_NO_WINDOW: u32 = 0x08000000;
                        cmd.creation_flags(CREATE_NO_WINDOW);
                    }

                    let output = cmd.output().map_err(|e| ServiceError::ExecutionFailed {
                        service: engine.to_service_type(),
                        details: format!("Failed to run mysqld --initialize-insecure: {}", e),
                    })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(ServiceError::ExecutionFailed {
                            service: engine.to_service_type(),
                            details: format!("MySQL --initialize-insecure failed: {}", stderr.trim()),
                        });
                    }
                }

                Ok(())
            }
            DbEngine::Postgresql => {
                let marker = data_dir.join("PG_VERSION");
                if marker.exists() {
                    return Ok(());
                }

                std::fs::create_dir_all(data_dir).map_err(|e| ServiceError::ExecutionFailed {
                    service: ServiceType::Postgresql,
                    details: format!("Failed to create data dir {}: {}", data_dir.display(), e),
                })?;

                let basedir = Self::find_basedir(bin_path);

                #[cfg(windows)]
                let candidates = [
                    bin_path.with_file_name("initdb.exe"),
                    basedir.join("bin").join("initdb.exe"),
                ];

                #[cfg(not(windows))]
                let candidates = [
                    bin_path.with_file_name("initdb"),
                    basedir.join("bin").join("initdb"),
                ];

                let mut initdb_tool = None;
                for tool in &candidates {
                    if tool.is_file() {
                        initdb_tool = Some(tool.clone());
                        break;
                    }
                }

                let initdb = initdb_tool.unwrap_or_else(|| bin_path.with_file_name("initdb"));

                let mut cmd = Command::new(&initdb);
                cmd.arg("-D")
                    .arg(data_dir)
                    .arg("-U")
                    .arg("postgres")
                    .arg("--auth=trust")
                    .arg("--encoding=UTF8")
                    .arg("--no-locale");

                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    const CREATE_NO_WINDOW: u32 = 0x08000000;
                    cmd.creation_flags(CREATE_NO_WINDOW);
                }

                let output = cmd.output().map_err(|e| ServiceError::ExecutionFailed {
                    service: ServiceType::Postgresql,
                    details: format!("Failed to run initdb at {}: {}", initdb.display(), e),
                })?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(ServiceError::ExecutionFailed {
                        service: ServiceType::Postgresql,
                        details: format!("PostgreSQL initdb failed: {}", stderr.trim()),
                    });
                }

                Ok(())
            }
        }
    }

    /// Constructs the foreground daemon Command with isolated data paths and custom bind_address.
    pub fn build_daemon_command(opts: &DbStartupOptions, data_dir: &Path) -> Command {
        let mut cmd = Command::new(opts.bin_path);
        let basedir = Self::find_basedir(opts.bin_path);

        match opts.engine {
            DbEngine::MariaDb | DbEngine::Mysql => {
                cmd.arg("--no-defaults");
                cmd.arg(format!("--basedir={}", basedir.display()));
                cmd.arg(format!("--datadir={}", data_dir.display()));

                let share_dir = basedir.join("share");
                if share_dir.exists() {
                    cmd.arg(format!("--lc-messages-dir={}", share_dir.display()));
                }

                cmd.arg(format!("--port={}", opts.port));
                cmd.arg(format!("--bind-address={}", opts.bind_address));
                cmd.arg("--console");
            }
            DbEngine::Postgresql => {
                cmd.arg("-D").arg(data_dir);
                cmd.arg("-p").arg(opts.port.to_string());
                cmd.arg("-h").arg(opts.bind_address);
                cmd.arg("-k").arg(std::env::temp_dir());
            }
        }

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        cmd
    }

    /// Safely terminates the database child process with SIGTERM on Unix, giving it time to
    /// flush dirty pages (InnoDB buffer pool, WAL) before falling back to kill.
    pub fn graceful_stop(
        service: ServiceType,
        child: &mut Child,
        timeout: Duration,
    ) -> Result<(), ServiceError> {
        #[cfg(unix)]
        {
            let pid = child.id();
            if pid > 0 {
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }

                let start = std::time::Instant::now();
                while start.elapsed() < timeout {
                    match child.try_wait() {
                        Ok(Some(_)) => return Ok(()),
                        Ok(None) => std::thread::sleep(Duration::from_millis(80)),
                        Err(e) => {
                            let _ = child.kill();
                            let _ = child.wait();
                            return Err(ServiceError::ExecutionFailed {
                                service,
                                details: format!("Error waiting for DB graceful shutdown: {}", e),
                            });
                        }
                    }
                }
            }
        }

        // Windows fallback or Unix timeout expiration
        let _ = child.kill();
        let _ = child.wait();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_db_engine_mapping() {
        assert_eq!(DbEngine::from_service_type(ServiceType::Mariadb), Some(DbEngine::MariaDb));
        assert_eq!(DbEngine::from_service_type(ServiceType::Mysql), Some(DbEngine::Mysql));
        assert_eq!(DbEngine::from_service_type(ServiceType::Postgresql), Some(DbEngine::Postgresql));
        assert_eq!(DbEngine::from_service_type(ServiceType::Apache), None);
        assert_eq!(DbEngine::from_service_type(ServiceType::Php), None);
    }

    #[test]
    fn test_find_basedir() {
        let bin = Path::new("/opt/mariadb/bin/mariadbd");
        assert_eq!(DbSupervisor::find_basedir(bin), PathBuf::from("/opt/mariadb"));

        let flat = Path::new("/opt/mariadb/mariadbd");
        assert_eq!(DbSupervisor::find_basedir(flat), PathBuf::from("/opt/mariadb"));
    }

    #[test]
    fn test_build_daemon_command_mariadb() {
        let bin_path = PathBuf::from("/test/mariadb/bin/mariadbd");
        let data_dir = PathBuf::from("/test/data/mariadb");
        let opts = DbStartupOptions {
            engine: DbEngine::MariaDb,
            version: "11.4.3",
            bin_path: &bin_path,
            port: 3307,
            bind_address: "127.0.0.1",
            base_data_dir: Path::new("/test/data"),
        };

        let cmd = DbSupervisor::build_daemon_command(&opts, &data_dir);
        let args: Vec<String> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        assert!(args.contains(&"--no-defaults".to_string()));
        assert!(args.contains(&"--port=3307".to_string()));
        assert!(args.contains(&"--bind-address=127.0.0.1".to_string()));
        assert!(args.contains(&"--console".to_string()));
        assert!(args.iter().any(|a| a.starts_with("--datadir=")));
        assert!(args.iter().any(|a| a.starts_with("--basedir=")));
    }

    #[test]
    fn test_build_daemon_command_postgresql_custom_bind() {
        let bin_path = PathBuf::from("/test/pgsql/bin/postgres");
        let data_dir = PathBuf::from("/test/data/postgresql");
        let opts = DbStartupOptions {
            engine: DbEngine::Postgresql,
            version: "17.0",
            bin_path: &bin_path,
            port: 5432,
            bind_address: "0.0.0.0",
            base_data_dir: Path::new("/test/data"),
        };

        let cmd = DbSupervisor::build_daemon_command(&opts, &data_dir);
        let args: Vec<String> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        assert_eq!(args[0], "-D");
        assert_eq!(args[1], data_dir.to_str().unwrap());
        assert_eq!(args[2], "-p");
        assert_eq!(args[3], "5432");
        assert_eq!(args[4], "-h");
        assert_eq!(args[5], "0.0.0.0");
        assert_eq!(args[6], "-k");
    }

    #[test]
    fn test_ensure_data_initialized_already_exists() {
        let temp_dir = std::env::temp_dir().join("amp_test_db_initialized");
        let _ = fs::create_dir_all(temp_dir.join("mysql"));

        let bin_path = PathBuf::from("/mock/bin/mariadbd");
        let res = DbSupervisor::ensure_data_initialized(DbEngine::MariaDb, &bin_path, &temp_dir);
        assert!(res.is_ok());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
