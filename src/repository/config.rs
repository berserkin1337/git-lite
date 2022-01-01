use configparser::ini::Ini;
use std::fmt::Debug;
use std::path::Path;

#[derive(Debug)]
pub struct GitConfig {
    pub conf: Ini,
}

impl GitConfig {
    pub fn new(conf: Ini) -> GitConfig {
        GitConfig { conf }
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        self.conf.write(path).map_err(|e| e.to_string())
    }
}

impl Default for GitConfig {
    fn default() -> Self {
        let mut conf = Ini::new();
        conf.setstr("core", "repositoryformatversion", Some("0"));
        conf.setstr("core", "filemode", Some("false"));
        conf.setstr("core", "bare", Some("false"));

        GitConfig { conf }
    }
}

