//! timeshift model — backup tool.

#[derive(Debug, Clone)]
pub struct Backup {
    pub name: String,
    pub source: String,
    pub dest: String,
    pub size_bytes: u64,
    pub created: u64,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub backups: Vec<Backup>,
}

impl Default for Model {
    fn default() -> Self { Self { backups: vec![] } }
}

impl Model {
    pub fn new() -> Self { Self::default() }

    pub fn create(&mut self, name: &str, source: &str, dest: &str, size_bytes: u64) -> usize {
        let id = self.backups.len();
        self.backups.push(Backup { name: name.into(), source: source.into(), dest: dest.into(), size_bytes, created: 0 });
        id
    }

    pub fn restore(&self, name: &str) -> Option<&Backup> {
        self.backups.iter().find(|b| b.name == name)
    }

    pub fn delete(&mut self, name: &str) -> bool {
        if let Some(i) = self.backups.iter().position(|b| b.name == name) {
            self.backups.remove(i);
            return true;
        }
        false
    }

    pub fn list(&self) -> &[Backup] { &self.backups }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn backup_create_restore_delete() {
        let mut m = Model::new();
        m.create("home", "/home/user", "/mnt/backup", 1024);
        assert!(m.restore("home").is_some());
        assert!(m.delete("home"));
    }
}
