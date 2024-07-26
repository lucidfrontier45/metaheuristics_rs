use std::time::Duration;

use approx::assert_abs_diff_eq;

use crate::optim::{LocalSearchOptimizer, SimulatedAnnealingOptimizer};

use super::QuadraticModel;

#[test]
fn test() {
    let model = QuadraticModel::new(3, vec![2.0, 0.0, -3.5], (-10.0, 10.0));
    let opt = SimulatedAnnealingOptimizer::new(10000, 10);
    let null_closure = None::<&fn(_)>;
    let (final_solution, final_score, _) = opt.run(
        &model,
        None,
        5000,
        Duration::from_secs(10),
        null_closure,
        (1.0, 0.1),
    );
    assert_abs_diff_eq!(2.0, final_solution[0], epsilon = 0.05);
    assert_abs_diff_eq!(0.0, final_solution[1], epsilon = 0.05);
    assert_abs_diff_eq!(-3.5, final_solution[2], epsilon = 0.05);
    assert_abs_diff_eq!(0.0, final_score.into_inner(), epsilon = 0.05);
}
