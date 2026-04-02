use super::*;

#[test]
fn regret_matching_uniform_on_zero() {
    let regrets = vec![0.0, 0.0, 0.0];
    let mut sigma = Vec::new();
    regret_matching(&regrets, &mut sigma);
    for &p in &sigma {
        assert!((p - 1.0 / 3.0).abs() < 1e-6);
    }
}

#[test]
fn regret_matching_positive_only() {
    let regrets = vec![2.0, -1.0, 1.0];
    let mut sigma = Vec::new();
    regret_matching(&regrets, &mut sigma);
    assert!((sigma[0] - 2.0 / 3.0).abs() < 1e-6);
    assert!((sigma[1] - 0.0).abs() < 1e-6);
    assert!((sigma[2] - 1.0 / 3.0).abs() < 1e-6);
}

#[test]
fn external_runs() {
    let mut solver = MCCFRSolver::new(2, 2, Some(Sampling::External), None, Some(42)).unwrap();
    solver.solve(100);
    assert_eq!(solver.iterations(), 100);
    assert!(solver.num_info_states() > 0);
}

#[test]
fn outcome_runs() {
    let mut solver = MCCFRSolver::new(2, 2, Some(Sampling::Outcome), None, Some(42)).unwrap();
    solver.solve(100);
    assert_eq!(solver.iterations(), 100);
    assert!(solver.num_info_states() > 0);
}

#[test]
fn outcome_discovers_all_2x2_canonical_info_states() {
    let mut solver = MCCFRSolver::new(2, 2, Some(Sampling::Outcome), Some(0.6), Some(42)).unwrap();
    solver.solve(10000);
    let n = solver.num_info_states();
    assert!(n >= 18, "expected >=18 canonical info states, got {n}");
    assert!(n <= 25, "expected <=25 canonical info states, got {n}");
}

#[test]
fn outcome_default_sampling() {
    let solver = MCCFRSolver::new(2, 2, None, None, None).unwrap();
    assert_eq!(solver.sampling(), Sampling::Outcome);
}

#[test]
fn outcome_strategy_valid() {
    let mut solver = MCCFRSolver::new(2, 2, None, None, Some(42)).unwrap();
    solver.solve(5000);
    let strategy = solver.get_average_strategy();
    assert!(!strategy.is_empty());
    for (key, probs) in &strategy {
        let sum: f32 = probs.iter().map(|(_, p)| p).sum();
        assert!((sum - 1.0).abs() < 0.05, "at {key}: sum={sum}");
    }
}

#[test]
fn outcome_3x3_runs() {
    let mut solver = MCCFRSolver::new(3, 3, None, None, Some(42)).unwrap();
    solver.solve(100);
    assert_eq!(solver.iterations(), 100);
    assert!(solver.num_info_states() > 100);
}

#[test]
fn epsilon_validation_outcome() {
    assert!(MCCFRSolver::new(2, 2, Some(Sampling::Outcome), Some(0.0), None).is_err());
    assert!(MCCFRSolver::new(2, 2, Some(Sampling::Outcome), Some(-0.1), None).is_err());
    assert!(MCCFRSolver::new(2, 2, Some(Sampling::Outcome), Some(1.1), None).is_err());
    assert!(MCCFRSolver::new(2, 2, Some(Sampling::Outcome), Some(0.6), None).is_ok());
    assert!(MCCFRSolver::new(2, 2, Some(Sampling::Outcome), Some(1.0), None).is_ok());
}

#[test]
fn epsilon_ignored_for_external() {
    assert!(MCCFRSolver::new(2, 2, Some(Sampling::External), Some(0.0), None).is_ok());
    assert!(MCCFRSolver::new(2, 2, Some(Sampling::External), None, None).is_ok());
}

#[test]
fn strategy_returns_cell_indices_not_sequential() {
    let mut solver = MCCFRSolver::new(2, 2, Some(Sampling::External), None, Some(42)).unwrap();
    solver.solve(5000);
    let strategy = solver.get_average_strategy();
    let board_size = 4usize;
    for (key, probs) in &strategy {
        for &(action, _) in probs {
            assert!(
                action < board_size,
                "action {action} out of bounds for 2x2 board in key {key}"
            );
        }
        let grid: String = key.chars().skip(3).filter(|&c| c != '\n').collect();
        for &(action, _) in probs {
            assert_eq!(
                grid.as_bytes()[action],
                b'.',
                "action {action} should be an empty cell in info state {key}"
            );
        }
    }
}
