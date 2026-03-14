use std::sync::atomic::{AtomicU64, Ordering};

const MAX_DEPTH: usize = 20;

static NODES_PER_DEPTH: [AtomicU64; MAX_DEPTH + 1] = 
    [const { AtomicU64::new(0) }; MAX_DEPTH + 1];
static HITS_PER_DEPTH: [AtomicU64; MAX_DEPTH + 1] = 
    [const { AtomicU64::new(0) }; MAX_DEPTH + 1];

pub fn inc_nodes(depth: usize) {
    if depth <= MAX_DEPTH {
        NODES_PER_DEPTH[depth].fetch_add(1, Ordering::Relaxed);
    }
}

pub fn inc_tt_hits(depth: usize) {
    if depth <= MAX_DEPTH {
        HITS_PER_DEPTH[depth].fetch_add(1, Ordering::Relaxed);
    }
}

pub fn reset_stats() {
    for i in 0..=MAX_DEPTH {
        NODES_PER_DEPTH[i].store(0, Ordering::Relaxed);
        HITS_PER_DEPTH[i].store(0, Ordering::Relaxed);
    }
}

pub fn print_stats_string() -> String {
    let mut lines = Vec::new();
    let mut total_nodes = 0;
    let mut total_hits = 0;

    lines.push("--- Depth Stats ---".to_string());

    for d in 0..=MAX_DEPTH {
        let nodes = NODES_PER_DEPTH[d].load(Ordering::Relaxed);
        let hits = HITS_PER_DEPTH[d].load(Ordering::Relaxed);

        if nodes > 0 {
            total_nodes += nodes;
            total_hits += hits;
            
            let hit_rate = (hits as f64 / nodes as f64) * 100.0;
            lines.push(format!(
                "Depth {:2}: Nodes {:8} | TT Hits {:8} | Hit Rate {:6.2}%",
                d, nodes, hits, hit_rate
            ));
        }
    }

    lines.push("--- Total ---".to_string());
    lines.push(format!("Total Nodes: {}", total_nodes));
    lines.push(format!("Total Hits:  {}", total_hits));
    
    if total_nodes > 0 {
        lines.push(format!(
            "Total Hit Rate: {:.2}%",
            (total_hits as f64 / total_nodes as f64) * 100.0
        ));
    }

    lines.join("\n")
}
