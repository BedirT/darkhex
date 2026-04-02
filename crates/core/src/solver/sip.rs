use std::collections::{BTreeSet, HashMap};

use crate::error::CoreError;

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
        if let Some(&entry) = actions.iter().find(|(_, p)| (*p - max_prob).abs() < 1e-6) {
            return vec![(entry.0, 1.0)];
        }
        return actions.to_vec();
    }

    // Step 3: Cap — stable sort descending by prob
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

    // Output sorted by cell index ascending
    filtered.sort_by_key(|&(idx, _)| idx);
    filtered
}

/// Precompute all unique fractions p/q where 1 <= p < q <= frac_limit.
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
        let pos = fractions.partition_point(|&f| f < prob);
        let mut best: Option<f32> = None;
        let mut best_dist = f32::MAX;
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
            None => return None,
        }
    }
    let sum: f32 = result.iter().map(|(_, p)| p).sum();
    if (sum - 1.0).abs() < 1e-6 {
        Some(result)
    } else {
        None
    }
}

pub fn simplify_policy(
    strategy: Strategy,
    epsilon: f32,
    action_cap: usize,
) -> Result<Strategy, CoreError> {
    if action_cap < 1 {
        return Err(CoreError::InvalidArgument(
            "action_cap must be >= 1".to_string(),
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

pub fn simplify_policy_plus(
    strategy: Strategy,
    epsilon: f32,
    action_cap: usize,
    frac_limit: usize,
    eta: f32,
) -> Result<Strategy, CoreError> {
    if action_cap < 1 {
        return Err(CoreError::InvalidArgument(
            "action_cap must be >= 1".to_string(),
        ));
    }
    if frac_limit < 1 {
        return Err(CoreError::InvalidArgument(
            "frac_limit must be >= 1".to_string(),
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
