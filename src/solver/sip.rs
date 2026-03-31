use std::collections::{BTreeSet, HashMap};

use pyo3::prelude::*;

type Strategy = HashMap<String, Vec<(usize, f32)>>;

/// Apply SIP to a single info state's action distribution.
fn sip_single(actions: &[(usize, f32)], epsilon: f32, action_cap: usize) -> Vec<(usize, f32)> {
    // Step 1: Filter by epsilon
    let mut filtered: Vec<(usize, f32)> = actions
        .iter()
        .filter(|(_, p)| *p > epsilon)
        .cloned()
        .collect();

    // Step 2: Fallback — if nothing passes, keep highest-prob action
    if filtered.is_empty() {
        let max_prob = actions
            .iter()
            .map(|(_, p)| *p)
            .fold(f32::NEG_INFINITY, f32::max);
        // Ties broken by lowest cell index (actions are sorted ascending by index from MCCFR)
        if let Some(&entry) = actions.iter().find(|(_, p)| (*p - max_prob).abs() < 1e-6) {
            return vec![(entry.0, 1.0)];
        }
        // Defensive: shouldn't happen if actions is non-empty
        return actions.to_vec();
    }

    // Step 3: Cap — stable sort descending by prob (equal probs keep ascending index order)
    filtered.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.cmp(&b.0))
    });
    filtered.truncate(action_cap);

    // Step 4: Normalize
    let total: f32 = filtered.iter().map(|(_, p)| p).sum();
    if total > 0.0 {
        for entry in &mut filtered {
            entry.1 /= total;
        }
    }

    // Output sorted by cell index ascending (consistent with MCCFR output)
    filtered.sort_by_key(|&(idx, _)| idx);
    filtered
}

/// Precompute all unique fractions p/q where 1 <= p < q <= frac_limit.
/// Returns a sorted Vec<f32>.
fn compute_fractions(frac_limit: usize) -> Vec<f32> {
    let mut fracs = BTreeSet::new();
    for q in 2..=frac_limit {
        for p in 1..q {
            let val = p as f32 / q as f32;
            fracs.insert(val.to_bits());
        }
    }
    fracs.into_iter().map(f32::from_bits).collect()
}

/// Try to fractionize an action distribution.
/// Returns Some(fractionized) if ALL actions map to a fraction within eta
/// AND the fractions sum to 1.0 (within tolerance). Returns None otherwise.
fn try_fractionize(
    actions: &[(usize, f32)],
    fractions: &[f32],
    eta: f32,
) -> Option<Vec<(usize, f32)>> {
    if fractions.is_empty() {
        return None;
    }
    let mut result = Vec::with_capacity(actions.len());
    for &(idx, prob) in actions {
        // Binary search for nearest fraction
        let pos = fractions.partition_point(|&f| f < prob);
        let mut best: Option<f32> = None;
        let mut best_dist = f32::MAX;
        // Check candidate at pos and pos-1
        if pos < fractions.len() {
            let dist = (fractions[pos] - prob).abs();
            if dist <= eta && dist < best_dist {
                best = Some(fractions[pos]);
                best_dist = dist;
            }
        }
        if pos > 0 {
            let dist = (fractions[pos - 1] - prob).abs();
            if dist <= eta && dist < best_dist {
                best = Some(fractions[pos - 1]);
            }
        }
        match best {
            Some(frac) => result.push((idx, frac)),
            None => return None, // This action has no matching fraction
        }
    }
    // Check sum = 1.0
    let sum: f32 = result.iter().map(|(_, p)| p).sum();
    if (sum - 1.0).abs() < 1e-6 {
        Some(result)
    } else {
        None
    }
}

#[pyfunction]
#[pyo3(signature = (strategy, epsilon, action_cap))]
pub fn simplify_policy(
    strategy: Strategy,
    epsilon: f32,
    action_cap: usize,
) -> PyResult<Strategy> {
    if action_cap < 1 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "action_cap must be >= 1",
        ));
    }
    let mut result = HashMap::with_capacity(strategy.len());
    for (key, actions) in &strategy {
        if actions.is_empty() {
            continue;
        }
        result.insert(key.clone(), sip_single(actions, epsilon, action_cap));
    }
    Ok(result)
}

#[pyfunction]
#[pyo3(signature = (strategy, epsilon, action_cap, frac_limit, eta))]
pub fn simplify_policy_plus(
    strategy: Strategy,
    epsilon: f32,
    action_cap: usize,
    frac_limit: usize,
    eta: f32,
) -> PyResult<Strategy> {
    if action_cap < 1 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "action_cap must be >= 1",
        ));
    }
    if frac_limit < 1 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "frac_limit must be >= 1",
        ));
    }
    let fractions = compute_fractions(frac_limit);
    let mut result = HashMap::with_capacity(strategy.len());
    for (key, actions) in &strategy {
        if actions.is_empty() {
            continue;
        }
        let simplified = sip_single(actions, epsilon, action_cap);
        let final_actions = if eta > 0.0 {
            try_fractionize(&simplified, &fractions, eta).unwrap_or(simplified)
        } else {
            simplified
        };
        result.insert(key.clone(), final_actions);
    }
    Ok(result)
}

#[cfg(test)]
#[path = "sip_tests.rs"]
mod tests;
