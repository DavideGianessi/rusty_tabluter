use std::sync::atomic::{AtomicU64, Ordering};

use crate::debug::debug_log;

static NODES_VISITED: AtomicU64 = AtomicU64::new(0);
static TT_HITS: AtomicU64 = AtomicU64::new(0);

pub fn inc_nodes() {
    NODES_VISITED.fetch_add(1, Ordering::Relaxed);
}

pub fn inc_tt_hits() {
    TT_HITS.fetch_add(1, Ordering::Relaxed);
}

pub fn reset_stats() {
    NODES_VISITED.store(0, Ordering::Relaxed);
    TT_HITS.store(0, Ordering::Relaxed);
}

pub fn print_stats() {
    let nodes = NODES_VISITED.load(Ordering::Relaxed);
    let hits = TT_HITS.load(Ordering::Relaxed);

    debug_log(0, &format!("Nodes visited: {}", nodes));
    debug_log(0,&format!("TT hits: {}", hits));

    if nodes > 0 {
        debug_log(0, &format!("TT hit rate: {:.2}%", (hits as f64 / nodes as f64) * 100.0));
    }
}
