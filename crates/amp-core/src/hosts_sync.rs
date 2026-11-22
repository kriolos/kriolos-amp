use std::fs;
use std::path::{Path, PathBuf};

pub const HOSTS_MARKER_BEGIN: &str = "# --- BEGIN AMP CONTROL SITES ---";
pub const HOSTS_MARKER_END: &str = "# --- END AMP CONTROL SITES ---";

pub struct HostsSync;

impl HostsSync {
    /// Returns the system default path to the hosts file.
    pub fn default_hosts_path() -> PathBuf {
        #[cfg(windows)]
        {
            let sys_root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_string());
            PathBuf::from(sys_root).join(r"System32\drivers\etc\hosts")
        }
        #[cfg(not(windows))]
        {
            PathBuf::from("/etc/hosts")
        }
    }

    /// Scans the `www` directory and returns a list of domain names based on the folder names.
    /// E.g. folder `blog` with tld `test` -> `blog.test`.
    pub fn scan_www_domains(www_dir: &Path, tld: &str) -> Vec<String> {
        let mut domains = Vec::new();
        if let Ok(entries) = fs::read_dir(www_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    if let Ok(name) = entry.file_name().into_string() {
                        if !name.starts_with('.') {
                            let clean_name = name.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
                            if !clean_name.is_empty() {
                                domains.push(format!("{}.{}", clean_name, tld));
                            }
                        }
                    }
                }
            }
        }
        domains.sort();
        domains.dedup();
        domains
    }

    /// Renders the managed block content for the hosts file.
    pub fn render_hosts_block(domains: &[String], target_ipv4: &str, target_ipv6: &str) -> String {
        if domains.is_empty() {
            return String::new();
        }

        let mut out = String::new();
        out.push_str(HOSTS_MARKER_BEGIN);
        out.push('\n');
        for d in domains {
            out.push_str(&format!("{:<15} {}\n", target_ipv4, d));
            out.push_str(&format!("{:<15} {}\n", target_ipv6, d));
        }
        out.push_str(HOSTS_MARKER_END);
        out.push('\n');
        out
    }

    /// Strips any previously existing AMP Control block from the given hosts file text.
    pub fn strip_amp_block(content: &str) -> String {
        if let Some(start) = content.find(HOSTS_MARKER_BEGIN) {
            if let Some(end) = content[start..].find(HOSTS_MARKER_END) {
                let full_end = start + end + HOSTS_MARKER_END.len();
                // Also consume following newline if present
                let slice_after = &content[full_end..];
                let trim_len = if slice_after.starts_with("\r\n") {
                    2
                } else if slice_after.starts_with('\n') {
                    1
                } else {
                    0
                };
                let mut stripped = String::new();
                stripped.push_str(&content[..start]);
                stripped.push_str(&content[full_end + trim_len..]);
                return stripped;
            }
        }
        content.to_string()
    }

    /// Updates the hosts file safely by preserving all original user content
    /// and updating only the delimited AMP Control block.
    pub fn update_hosts_file(
        hosts_path: &Path,
        domains: &[String],
        target_ipv4: &str,
        target_ipv6: &str,
    ) -> Result<(), std::io::Error> {
        let original_content = if hosts_path.exists() {
            fs::read_to_string(hosts_path)?
        } else {
            String::new()
        };

        let stripped = Self::strip_amp_block(&original_content);
        let block = Self::render_hosts_block(domains, target_ipv4, target_ipv6);

        let mut new_content = stripped.trim_end().to_string();
        if !new_content.is_empty() {
            new_content.push('\n');
            new_content.push('\n');
        }
        new_content.push_str(&block);

        fs::write(hosts_path, new_content)?;
        Ok(())
    }

    /// Cleans/removes the AMP Control block from the hosts file completely.
    pub fn clean_hosts_file(hosts_path: &Path) -> Result<(), std::io::Error> {
        if !hosts_path.exists() {
            return Ok(());
        }
        let original = fs::read_to_string(hosts_path)?;
        let stripped = Self::strip_amp_block(&original);
        if stripped != original {
            fs::write(hosts_path, stripped.trim_end().to_string() + "\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_and_render_hosts_block() {
        let initial = "127.0.0.1 localhost\n::1 localhost\n";
        let domains = vec!["site1.test".to_string(), "site2.test".to_string()];
        let block = HostsSync::render_hosts_block(&domains, "127.0.0.1", "::1");

        let combined = format!("{}\n{}", initial, block);
        assert!(combined.contains(HOSTS_MARKER_BEGIN));
        assert!(combined.contains("site1.test"));

        let stripped = HostsSync::strip_amp_block(&combined);
        assert_eq!(stripped.trim(), initial.trim());
        assert!(!stripped.contains("site1.test"));
    }

    #[test]
    fn test_scan_www_domains() {
        let temp_dir = std::env::temp_dir().join("amp_test_www_scan");
        let _ = fs::create_dir_all(temp_dir.join("meu-app"));
        let _ = fs::create_dir_all(temp_dir.join("loja"));
        let _ = fs::create_dir_all(temp_dir.join(".git")); // Should be ignored

        let domains = HostsSync::scan_www_domains(&temp_dir, "test");
        assert_eq!(domains, vec!["loja.test".to_string(), "meu-app.test".to_string()]);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
