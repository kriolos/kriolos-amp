use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct TemplateEngine {
    variables: HashMap<String, String>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn set_variable<K: Into<String>, V: Into<String>>(&mut self, key: K, val: V) -> &mut Self {
        self.variables.insert(key.into(), val.into());
        self
    }

    pub fn render(&self, content: &str) -> String {
        let mut result = content.to_string();
        for (key, val) in &self.variables {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, val);
        }
        result
    }

    pub fn process_file(
        &self,
        template_path: &Path,
        output_path: &Path,
    ) -> Result<(), std::io::Error> {
        let content = fs::read_to_string(template_path)?;
        let rendered = self.render(&content);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(output_path, rendered)?;
        Ok(())
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}
