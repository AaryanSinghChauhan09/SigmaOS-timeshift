//! SigmaOS-timeshift — system snapshot & restore manager
//! (Linux Mint `timeshift` parity).
//!
//! Snapshot scheduling modes (RSYNC copy vs BTRFS native), retention
//! policy (keep N hourly/daily/weekly/monthly), restore selection,
//! and boot-entry tracking. Filesystem operations are delegated to
//! the storage subsystem — this is the policy/metadata layer.

/// Snapshot backend mode (timeshift --snapshot-device modes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotMode {
    /// File-level copy (works on any fs, slower).
    Rsync,
    /// BTRFS native snapshot (instant, CoW).
    Btrfs,
}

/// Retention policy like timeshift's schedule settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionPolicy {
    pub keep_hourly: u32,
    pub keep_daily: u32,
    pub keep_weekly: u32,
    pub keep_monthly: u32,
    /// Minimum interval between snapshots in hours.
    pub min_interval_hours: u32,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            keep_hourly: 6,
            keep_daily: 7,
            keep_weekly: 4,
            keep_monthly: 2,
            min_interval_hours: 6,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotKind {
    /// Scheduled snapshot.
    Scheduled,
    /// Manual `timeshift --create`.
    Manual,
    /// Pre-upgrade safety snapshot.
    PreUpgrade,
}

#[derive(Debug, Clone)]
pub struct SystemSnapshotEntry {
    pub snapshot_id: String,
    pub btrfs_subvolume: String,
    pub is_boot_snapshot: bool,
    pub mode: SnapshotMode,
    pub kind: SnapshotKind,
    pub created_unix: i64,
}

impl SystemSnapshotEntry {
    /// Parse timeshift-style id `YYYY-MM-DD_HH-MM-SS` -> unix is not
    /// derivable without a calendar; store creation time separately.
    pub fn age_hours(&self, now_unix: i64) -> i64 {
        (now_unix - self.created_unix).max(0) / 3600
    }
}

#[derive(Debug)]
pub struct TimeshiftManager {
    snapshots: Vec<SystemSnapshotEntry>,
    mode: SnapshotMode,
    retention: RetentionPolicy,
    /// Last snapshot unix time for min-interval enforcement.
    last_snapshot_unix: Option<i64>,
}

impl Default for TimeshiftManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeshiftManager {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            mode: SnapshotMode::Btrfs,
            retention: RetentionPolicy::default(),
            last_snapshot_unix: None,
        }
    }

    pub fn set_mode(&mut self, mode: SnapshotMode) {
        self.mode = mode;
    }

    pub fn set_retention(&mut self, r: RetentionPolicy) {
        self.retention = r;
    }

    /// Whether a new snapshot is allowed now (min-interval gate).
    pub fn can_snapshot_now(&self, now_unix: i64) -> bool {
        match self.last_snapshot_unix {
            None => true,
            Some(last) => {
                (now_unix - last) >= (self.retention.min_interval_hours as i64) * 3600
            }
        }
    }

    /// Record a snapshot. Returns false if interval not elapsed.
    pub fn record_snapshot_at(
        &mut self,
        id: &str,
        subvol: &str,
        boot: bool,
        kind: SnapshotKind,
        now_unix: i64,
    ) -> bool {
        if !self.can_snapshot_now(now_unix) {
            return false;
        }
        self.snapshots.push(SystemSnapshotEntry {
            snapshot_id: id.to_string(),
            btrfs_subvolume: subvol.to_string(),
            is_boot_snapshot: boot,
            mode: self.mode,
            kind,
            created_unix: now_unix,
        });
        self.last_snapshot_unix = Some(now_unix);
        true
    }

    /// Legacy API kept for compatibility.
    pub fn record_snapshot(&mut self, id: &str, subvol: &str, boot: bool) {
        let _ = self.record_snapshot_at(id, subvol, boot, SnapshotKind::Scheduled, 0);
    }

    pub fn count(&self) -> usize {
        self.snapshots.len()
    }

    pub fn snapshots(&self) -> &[SystemSnapshotEntry] {
        &self.snapshots
    }

    /// Candidates for deletion under the retention policy, oldest first.
    /// Simplified model: counts by age bucket vs keep limits.
    pub fn retention_prune_candidates(&self, now_unix: i64) -> Vec<&SystemSnapshotEntry> {
        let r = self.retention;
        let mut hourly = 0u32;
        let mut daily = 0u32;
        let mut weekly = 0u32;
        let mut monthly = 0u32;
        let mut prune = Vec::new();

        // sort oldest first so we keep the newest within each bucket
        let mut sorted: Vec<&SystemSnapshotEntry> = self.snapshots.iter().collect();
        sorted.sort_by_key(|s| s.created_unix);

        for s in sorted.iter().rev() {
            let age_h = s.age_hours(now_unix);
            let bucket_full = if age_h < 24 {
                hourly += 1;
                hourly > r.keep_hourly
            } else if age_h < 24 * 7 {
                daily += 1;
                daily > r.keep_daily
            } else if age_h < 24 * 30 {
                weekly += 1;
                weekly > r.keep_weekly
            } else {
                monthly += 1;
                monthly > r.keep_monthly
            };
            if bucket_full {
                prune.push(*s);
            }
        }
        prune.sort_by_key(|s| s.created_unix);
        prune
    }

    /// Select best restore point: newest non-boot snapshot, or newest
    /// boot snapshot when `prefer_boot`.
    pub fn select_restore(&self, prefer_boot: bool) -> Option<&SystemSnapshotEntry> {
        let mut candidates: Vec<&SystemSnapshotEntry> = self
            .snapshots
            .iter()
            .filter(|s| prefer_boot == s.is_boot_snapshot || !prefer_boot)
            .collect();
        if candidates.is_empty() {
            candidates = self.snapshots.iter().collect();
        }
        candidates.iter().max_by_key(|s| s.created_unix).copied()
    }
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

    #[test]
    fn min_interval_gates_snapshots() {
        let mut ts = TimeshiftManager::new();
        ts.set_retention(RetentionPolicy { min_interval_hours: 6, ..Default::default() });
        assert!(ts.record_snapshot_at("s1", "@", false, SnapshotKind::Manual, 1000));
        // 1 hour later -> rejected
        assert!(!ts.record_snapshot_at("s2", "@", false, SnapshotKind::Manual, 1000 + 3600));
        // 7 hours later -> accepted
        assert!(ts.record_snapshot_at("s3", "@", false, SnapshotKind::Manual, 1000 + 7 * 3600));
    }

    #[test]
    fn retention_prunes_oldest_beyond_keep() {
        let mut ts = TimeshiftManager::new();
        ts.set_retention(RetentionPolicy {
            keep_hourly: 2,
            keep_daily: 0,
            keep_weekly: 0,
            keep_monthly: 0,
            min_interval_hours: 0,
        });
        // 4 snapshots all <24h old
        for i in 0..4 {
            ts.record_snapshot_at(&format!("s{i}"), "@", false, SnapshotKind::Scheduled, i * 3600);
        }
        let prune = ts.retention_prune_candidates(4 * 3600);
        // keep newest 2, prune oldest 2
        assert_eq!(prune.len(), 2);
        assert_eq!(prune[0].snapshot_id, "s0");
    }

    #[test]
    fn restore_prefers_boot_or_latest() {
        let mut ts = TimeshiftManager::new();
        ts.set_retention(RetentionPolicy { min_interval_hours: 0, ..Default::default() });
        ts.record_snapshot_at("old-boot", "@boot", true, SnapshotKind::Scheduled, 100);
        ts.record_snapshot_at("new-normal", "@", false, SnapshotKind::Scheduled, 200);
        let normal = ts.select_restore(false).unwrap();
        assert_eq!(normal.snapshot_id, "new-normal");
        let boot = ts.select_restore(true).unwrap();
        assert_eq!(boot.snapshot_id, "old-boot");
    }
}
