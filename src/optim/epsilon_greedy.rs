use std::{cell::RefCell, rc::Rc};

use rand::Rng;
use rayon::prelude::*;

use crate::callback::{OptCallbackFn, OptProgress};
use crate::OptModel;

/// Optimizer that implements epsilon-greedy algorithm.
/// Unlike a total greedy algorithm such as hill climbing,
/// it allows transitions that worsens the score with a fixed probability
#[derive(Clone, Copy)]
pub struct EpsilonGreedyOptimizer {
    patience: usize,
    n_trials: usize,
    epsilon: f64,
}

impl EpsilonGreedyOptimizer {
    /// Constructor of EpsilonGreedyOptimizer
    ///
    /// - `patience` : the optimizer will give up
    ///   if there is no improvement of the score after this number of iterations
    /// - `n_trials` : number of trial states to generate and evaluate at each iteration
    /// - `epsilon` : probability to accept a transition that worsens the score. Must be in [0, 1].
    pub fn new(patience: usize, n_trials: usize, epsilon: f64) -> Self {
        Self {
            patience,
            n_trials,
            epsilon,
        }
    }

    /// Start optimization
    ///
    /// - `model` : the model to optimize
    /// - `initial_state` : the initial state to start optimization. If None, a random state will be generated.
    /// - `n_iter`: maximum iterations
    /// - `callback` : callback function that will be invoked at the end of each iteration
    pub fn optimize<M, F>(
        &self,
        model: &M,
        initial_state: Option<M::StateType>,
        n_iter: usize,
        callback: Option<&F>,
    ) -> (M::StateType, M::ScoreType)
    where
        M: OptModel + Sync + Send,
        F: OptCallbackFn<M::StateType, M::ScoreType>,
    {
        let mut rng = rand::thread_rng();
        let mut current_state = if let Some(s) = initial_state {
            s
        } else {
            model.generate_random_state(&mut rng).unwrap()
        };
        let mut current_score = model.evaluate_state(&current_state);
        let best_state = Rc::new(RefCell::new(current_state.clone()));
        let mut best_score = current_score;
        let mut counter = 0;
        let mut accepted_counter = 0;
        for it in 0..n_iter {
            let (trial_state, trial_score) = (0..self.n_trials)
                .into_par_iter()
                .map(|_| {
                    let mut rng = rand::thread_rng();
                    let (state, _, score) =
                        model.generate_trial_state(&current_state, &mut rng, Some(current_score));
                    (state, score)
                })
                .min_by_key(|(_, score)| *score)
                .unwrap();

            let r: f64 = rng.gen();

            if trial_score < current_score || self.epsilon > r {
                current_state = trial_state;
                current_score = trial_score;
                accepted_counter += 1;
            }

            if current_score < best_score {
                best_state.replace(current_state.clone());
                best_score = current_score;
                counter = 0;
            }

            counter += 1;
            if counter == self.patience {
                break;
            }

            if let Some(f) = callback {
                let progress =
                    OptProgress::new(it, accepted_counter, best_state.clone(), best_score);
                f(progress);
            }
        }

        let best_state = (*best_state.borrow()).clone();
        (best_state, best_score)
    }
}