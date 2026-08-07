use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub struct Resolver {
    base_path: Option<PathBuf>,
    embedded: HashMap<String, &'static str>,
}

impl Resolver {
    pub fn new(base_path: Option<PathBuf>) -> Self {
        let mut embedded = HashMap::new();
        embedded.insert(
            "stdlib/product.page".to_string(),
            include_str!("../stdlib/product.page"),
        );
        embedded.insert(
            "stdlib/data.page".to_string(),
            include_str!("../stdlib/data.page"),
        );
        embedded.insert(
            "stdlib/docs.page".to_string(),
            include_str!("../stdlib/docs.page"),
        );
        embedded.insert(
            "stdlib/layout.page".to_string(),
            include_str!("../stdlib/layout.page"),
        );

        Self {
            base_path: base_path.and_then(|path| fs::canonicalize(path).ok()),
            embedded,
        }
    }

    pub fn resolve(&self, path_str: &str) -> Result<String, String> {
        if !is_valid_import_path(path_str) {
            return Err(format!("Invalid import path: {path_str}"));
        }

        // 1. Try embedded first if it starts with stdlib/
        if let Some(content) = self.embedded.get(path_str) {
            return Ok(content.to_string());
        }

        let Some(base) = &self.base_path else {
            return Err(format!(
                "Could not resolve path without an explicit import root: {path_str}"
            ));
        };
        let candidate = base.join(path_str);
        let path = fs::canonicalize(&candidate)
            .map_err(|_| format!("Could not resolve path: {path_str}"))?;
        if !path.starts_with(base) {
            return Err(format!(
                "Import path escapes the configured root: {path_str}"
            ));
        }
        fs::read_to_string(&path).map_err(|_| format!("Could not resolve path: {path_str}"))
    }

    pub fn base_path(&self) -> Option<&Path> {
        self.base_path.as_deref()
    }
}

pub fn is_valid_import_path(path_str: &str) -> bool {
    !path_str.contains("..")
        && !Path::new(path_str).is_absolute()
        && !Path::new(path_str)
            .components()
            .any(|component| matches!(component, Component::ParentDir))
}
