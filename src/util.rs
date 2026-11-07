//! timeshift model — system snapshot/restore tool.

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub id: String,
    pub name: String,
    pub date: String,
    pub size: u64,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub snapshots: Vec<Snapshot>,
    pub next_id: u64,
}

impl Default for Model {
    fn default() -> Self { Self { snapshots: vec![], next_id: 1 } }
}

impl Model {
    pub fn new() -> Self { Self::default() }

    pub fn create(&mut self, name: &str, date: &str) -> String {
        let id = format!("{:x}", self.next_id);
        self.next_id += 1;
        self.snapshots.push(Snapshot { id: id.clone(), name: name.into(), date: date.into(), size: 0, description: String::new() });
        id
    }

    pub fn delete(&mut self, id: &str) -> bool {
        if let Some(pos) = self.snapshots.iter().position(|s| s.id == id) {
            self.snapshots.remove(pos);
            true
        } else { false }
    }

    pub fn list(&self) -> &[Snapshot] { &self.snapshots }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshot_create_delete() {
        let mut t = Model::new();
        let id = t.create("before-update", "2024-01-01");
        assert_eq!(t.list().len(), 1);
        assert!(t.delete(&id));
    }
}
