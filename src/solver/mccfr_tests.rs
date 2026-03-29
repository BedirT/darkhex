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
fn outcome_discovers_all_2x2_info_states() {
    let mut solver =
        MCCFRSolver::new(2, 2, Some(Sampling::Outcome), Some(0.6), Some(42)).unwrap();
    solver.solve(10000);
    let n = solver.num_info_states();
    assert!(n >= 40, "expected >=40 info states, got {n}");
    assert!(n <= 50, "expected <=50 info states, got {n}");
}

#[test]
fn outcome_default_sampling() {
    let solver = MCCFRSolver::new(2, 2, None, None, None).unwrap();
    assert_eq!(solver.sampling, Sampling::Outcome);
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
    // Outcome Sampling validates epsilon
    assert!(MCCFRSolver::new(2, 2, Some(Sampling::Outcome), Some(0.0), None).is_err());
    assert!(MCCFRSolver::new(2, 2, Some(Sampling::Outcome), Some(-0.1), None).is_err());
    assert!(MCCFRSolver::new(2, 2, Some(Sampling::Outcome), Some(1.1), None).is_err());
    assert!(MCCFRSolver::new(2, 2, Some(Sampling::Outcome), Some(0.6), None).is_ok());
    assert!(MCCFRSolver::new(2, 2, Some(Sampling::Outcome), Some(1.0), None).is_ok());
}

#[test]
fn epsilon_ignored_for_external() {
    // External Sampling ignores epsilon — any value accepted
    assert!(MCCFRSolver::new(2, 2, Some(Sampling::External), Some(0.0), None).is_ok());
    assert!(MCCFRSolver::new(2, 2, Some(Sampling::External), None, None).is_ok());
}
