use std::sync::atomic::{AtomicBool, Ordering};

static SKIP_HISTORY_BACKFILL: AtomicBool = AtomicBool::new(false);

/// Enable or disable history backfill skipping for initial sync imports.
pub fn set_skip_history_backfill(enabled: bool) {
    SKIP_HISTORY_BACKFILL.store(enabled, Ordering::Relaxed);
}

/// Returns whether initial-sync imports should avoid creating history gaps.
pub fn skip_history_backfill() -> bool {
    SKIP_HISTORY_BACKFILL.load(Ordering::Relaxed)
}
