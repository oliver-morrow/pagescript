use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Resolver {
    base_path: Option<PathBuf>,
    embedded: HashMap<String, &'static str>,
}

impl Resolver {
    pub fn new(base_path: Option<PathBuf>) -> Self {
        let mut embedded = HashMap::new();
        embedded.insert(
            "stdlib/product.page".to_string(),
            include_str!("../../../stdlib/product.page"),
        );
        embedded.insert(
            "stdlib/data.page".to_string(),
            include_str!("../../../stdlib/data.page"),
        );
        embedded.insert(
            "stdlib/docs.page".to_string(),
            include_str!("../../../stdlib/docs.page"),
        );
        embedded.insert(
            "stdlib/layout.page".to_string(),
            include_str!("../../../stdlib/layout.page"),
        );

        Self {
            base_path,
            embedded,
        }
    }

    pub fn resolve(&self, path_str: &str) -> Result<String, String> {
        // 1. Try embedded first if it starts with stdlib/
        if let Some(content) = self.embedded.get(path_str) {
            return Ok(content.to_string());
        }

        // 2. Try filesystem relative to base_path
        if let Some(base) = &self.base_path {
            let path = base.join(path_str);
            if let Ok(content) = fs::read_to_string(&path) {
                return Ok(content);
            }
        }

        // 3. Try filesystem relative to current dir
        if let Ok(content) = fs::read_to_string(path_str) {
            return Ok(content);
        }

        Err(format!("Could not resolve path: {path_str}"))
    }

    pub fn base_path(&self) -> Option<&Path> {
        self.base_path.as_deref()
    }
}
