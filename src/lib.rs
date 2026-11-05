#[derive(Debug, Clone)]
pub struct SystemSnapshotEntry {
    pub snapshot_id: String,
    pub btrfs_subvolume: String,
    pub is_boot_snapshot: bool,
}

#[derive(Debug, Default)]
pub struct TimeshiftManager {
    snapshots: Vec<SystemSnapshotEntry>,
}

impl TimeshiftManager {
    pub fn new() -> Self { Self::default() }
    pub fn record_snapshot(&mut self, id: &str, subvol: &str, boot: bool) {
        self.snapshots.push(SystemSnapshotEntry {
            snapshot_id: id.to_string(),
            btrfs_subvolume: subvol.to_string(),
            is_boot_snapshot: boot,
        });
    }
    pub fn count(&self) -> usize { self.snapshots.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_timeshift_snapshot_recording() {
        let mut ts = TimeshiftManager::new();
        ts.record_snapshot("2026-11-05_12-00-00", "@rootfs_snap1", true);
        assert_eq!(ts.count(), 1);
    }
}
