//! Reusable view builders (top-K, windows, joins, graphs).

use timely::dataflow::Scope;

pub struct TopKConfig {
    pub k: usize,
}

impl Default for TopKConfig {
    fn default() -> Self {
        Self { k: 10 }
    }
}

/// Placeholder for a top-K builder; concrete implementations will live here.
pub fn top_k_placeholder<G: Scope>(_cfg: TopKConfig, _scope: &mut G) {
    // Implementation to be added in MVP Phase 1.
}

