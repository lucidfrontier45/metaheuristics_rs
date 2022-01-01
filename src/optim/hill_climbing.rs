use ordered_float::NotNan;
use rayon::prelude::*;

use crate::OptModel;

use super::Optimizer;

fn optimize<ModelType, StateType>(
    model: &ModelType,
    initial_state: Option<&StateType>,
    n_iter: usize,
    patience: usize,
) -> (StateType, f64)
where
    ModelType: OptModel<StateType = StateType>,
    StateType: Clone,
{
    let mut rng = rand::thread_rng();
    let mut current_state = if let Some(s) = initial_state {
        s.clone()
    } else {
        model.generate_random_state(&mut rng).unwrap()
    };
    let mut current_score = model.evaluate_state(&current_state);
    let mut counter = 0;
    for _ in 0..n_iter {
        let (trial_state, _) = model.generate_trial_state(&current_state, &mut rng);
        let trial_score = model.evaluate_state(&trial_state);
        if trial_score < current_score {
            current_state = trial_state;
            current_score = trial_score;
            counter = 0;
        } else {
            counter += 1;
            if counter == patience {
                break;
            }
        }
    }
    (current_state, current_score)
}

#[derive(Clone, Copy)]
pub struct HillClimbingOptimizer {
    patience: usize,
    n_trials: usize,
}

impl HillClimbingOptimizer {
    pub fn new(patience: usize, n_trials: usize) -> Self {
        Self { patience, n_trials }
    }
}

impl Optimizer for HillClimbingOptimizer {
    type AdditionalArgType = ();
    type AdditionalRetType = ();

    fn optimize<ModelType, StateType>(
        &self,
        model: &ModelType,
        initial_state: Option<&StateType>,
        n_iter: usize,
        _arg: &Self::AdditionalArgType,
    ) -> (StateType, f64, Self::AdditionalRetType)
    where
        ModelType: OptModel<StateType = StateType> + Sync + Send,
        StateType: Clone + Sync + Send,
    {
        let (final_state, final_score) = (0..self.n_trials)
            .into_par_iter()
            .map(|_| optimize(model, initial_state, n_iter, self.patience))
            .min_by_key(|(_, score)| NotNan::new(*score).unwrap())
            .unwrap();

        (final_state, final_score, ())
    }
}