use super::*;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-5
}

fn sum_probs(actions: &[(usize, f32)]) -> f32 {
    actions.iter().map(|(_, p)| p).sum()
}

// --- sip_single tests ---

#[test]
fn test_sip_single_action() {
    let actions = vec![(0, 1.0)];
    let result = sip_single(&actions, 0.1, 2);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], (0, 1.0));
}

#[test]
fn test_sip_filter_and_normalize() {
    // Actions: 0 at 0.6, 1 at 0.3, 2 at 0.05, 3 at 0.05
    let actions = vec![(0, 0.6), (1, 0.3), (2, 0.05), (3, 0.05)];
    let result = sip_single(&actions, 0.1, 4);
    // Only 0 and 1 survive (0.6 and 0.3 > 0.1)
    assert_eq!(result.len(), 2);
    assert!(approx_eq(sum_probs(&result), 1.0));
    // 0.6/0.9 = 0.667, 0.3/0.9 = 0.333
    assert!(approx_eq(result[0].1, 2.0 / 3.0));
    assert!(approx_eq(result[1].1, 1.0 / 3.0));
}

#[test]
fn test_sip_action_cap() {
    let actions = vec![(0, 0.4), (1, 0.3), (2, 0.2), (3, 0.1)];
    // All above epsilon=0.05, but cap=2
    let result = sip_single(&actions, 0.05, 2);
    assert_eq!(result.len(), 2);
    // Top 2 by prob: action 0 (0.4) and action 1 (0.3)
    assert_eq!(result[0].0, 0);
    assert_eq!(result[1].0, 1);
    assert!(approx_eq(sum_probs(&result), 1.0));
}

#[test]
fn test_sip_fallback_all_below_epsilon() {
    let actions = vec![(0, 0.05), (1, 0.03), (2, 0.02)];
    let result = sip_single(&actions, 0.1, 4);
    // All below epsilon, fallback to highest (action 0)
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], (0, 1.0));
}

#[test]
fn test_sip_fallback_tiebreaking() {
    // Two actions tied for highest, lowest index should win
    let actions = vec![(2, 0.05), (5, 0.05), (8, 0.03)];
    let result = sip_single(&actions, 0.1, 4);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, 2); // lowest index among tied
}

#[test]
fn test_sip_stable_sort_tiebreaking() {
    // Equal probs: lower index should be kept when capping
    let actions = vec![(0, 0.25), (1, 0.25), (2, 0.25), (3, 0.25)];
    let result = sip_single(&actions, 0.0, 2);
    assert_eq!(result.len(), 2);
    // Stable sort descending by prob, equal probs -> ascending index
    // So actions 0 and 1 should be kept
    assert_eq!(result[0].0, 0);
    assert_eq!(result[1].0, 1);
    assert!(approx_eq(sum_probs(&result), 1.0));
}

#[test]
fn test_sip_output_sorted_by_index() {
    // High prob on action 3, low on action 0
    let actions = vec![(0, 0.2), (3, 0.8)];
    let result = sip_single(&actions, 0.1, 4);
    // Output should be sorted by cell index ascending
    assert_eq!(result[0].0, 0);
    assert_eq!(result[1].0, 3);
}

// --- compute_fractions tests ---

#[test]
fn test_fractions_limit_2() {
    let fracs = compute_fractions(2);
    assert_eq!(fracs.len(), 1);
    assert!(approx_eq(fracs[0], 0.5));
}

#[test]
fn test_fractions_limit_3() {
    let fracs = compute_fractions(3);
    // 1/2, 1/3, 2/3
    assert_eq!(fracs.len(), 3);
    assert!(approx_eq(fracs[0], 1.0 / 3.0));
    assert!(approx_eq(fracs[1], 0.5));
    assert!(approx_eq(fracs[2], 2.0 / 3.0));
}

#[test]
fn test_fractions_deduplication() {
    let fracs = compute_fractions(4);
    // 1/2 = 2/4, should not be duplicated
    let half_count = fracs.iter().filter(|&&f| approx_eq(f, 0.5)).count();
    assert_eq!(half_count, 1);
}

#[test]
fn test_fractions_sorted() {
    let fracs = compute_fractions(20);
    for w in fracs.windows(2) {
        assert!(w[0] <= w[1], "fractions not sorted: {} > {}", w[0], w[1]);
    }
}

// --- try_fractionize tests ---

#[test]
fn test_fractionize_success() {
    let fracs = compute_fractions(3);
    // 0.334 ~ 1/3, 0.666 ~ 2/3
    let actions = vec![(0, 0.334), (1, 0.666)];
    let result = try_fractionize(&actions, &fracs, 0.01);
    assert!(result.is_some());
    let result = result.unwrap();
    assert!(approx_eq(result[0].1, 1.0 / 3.0));
    assert!(approx_eq(result[1].1, 2.0 / 3.0));
}

#[test]
fn test_fractionize_half_half() {
    let fracs = compute_fractions(2);
    let actions = vec![(0, 0.499), (1, 0.501)];
    let result = try_fractionize(&actions, &fracs, 0.01);
    assert!(result.is_some());
    let result = result.unwrap();
    assert!(approx_eq(result[0].1, 0.5));
    assert!(approx_eq(result[1].1, 0.5));
}

#[test]
fn test_fractionize_rejected_no_match() {
    let fracs = compute_fractions(3);
    // 0.4 is not within 0.01 of 1/3 or 1/2
    let actions = vec![(0, 0.4), (1, 0.6)];
    let result = try_fractionize(&actions, &fracs, 0.01);
    assert!(result.is_none());
}

#[test]
fn test_fractionize_rejected_sum_not_one() {
    let fracs = compute_fractions(4);
    // Both near 1/3, but 1/3 + 1/3 != 1.0
    let actions = vec![(0, 0.334), (1, 0.334)];
    let result = try_fractionize(&actions, &fracs, 0.01);
    // Should be None because 1/3 + 1/3 != 1.0
    assert!(result.is_none());
}

// --- simplify_policy tests ---

#[test]
fn test_simplify_empty_strategy() {
    let strategy = HashMap::new();
    let result = simplify_policy(strategy, 0.1, 2).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_simplify_multiple_info_states() {
    let mut strategy = HashMap::new();
    strategy.insert(
        "P0\n..\n..".to_string(),
        vec![(0, 0.6), (1, 0.3), (2, 0.05), (3, 0.05)],
    );
    strategy.insert("P1\n..\n..".to_string(), vec![(0, 0.9), (1, 0.1)]);
    let result = simplify_policy(strategy, 0.1, 2).unwrap();
    assert_eq!(result.len(), 2);
    // Each should be independently simplified
    for (_, actions) in &result {
        assert!(actions.len() <= 2);
        assert!(approx_eq(sum_probs(actions), 1.0));
    }
}

#[test]
fn test_simplify_validates_action_cap() {
    let strategy = HashMap::new();
    assert!(simplify_policy(strategy, 0.1, 0).is_err());
}

// --- simplify_policy_plus tests ---

#[test]
fn test_simplify_plus_fractionizes() {
    let mut strategy = HashMap::new();
    // After SIP with epsilon=0.1, cap=2: should get ~(0.667, 0.333)
    // Which should fractionize to (2/3, 1/3) with frac_limit=3
    strategy.insert(
        "P0\n..\n..".to_string(),
        vec![(0, 0.6), (1, 0.3), (2, 0.05), (3, 0.05)],
    );
    let result = simplify_policy_plus(strategy, 0.1, 2, 3, 0.01).unwrap();
    let actions = &result["P0\n..\n.."];
    assert_eq!(actions.len(), 2);
    // Check fractionized values
    assert!(approx_eq(actions[0].1, 2.0 / 3.0) || approx_eq(actions[0].1, 1.0 / 3.0));
}

#[test]
fn test_simplify_plus_validates_frac_limit() {
    let strategy = HashMap::new();
    assert!(simplify_policy_plus(strategy, 0.1, 2, 0, 0.005).is_err());
}
