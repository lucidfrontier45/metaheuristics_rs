use anyhow::Result as AnyResult;
use auto_impl::auto_impl;

use crate::{callback::OptCallbackFn, Duration, OptModel};

/// Optimizer that implements local search algorithm
#[auto_impl(&, Box, Rc, Arc)]
pub trait LocalSearchOptimizer<M: OptModel> {
    /// Start optimization
    fn optimize<F>(
        &self,
        model: &M,
        initial_solution: M::SolutionType,
        initial_score: M::ScoreType,
        n_iter: usize,
        time_limit: Duration,
        callback: Option<&F>,
    ) -> (M::SolutionType, M::ScoreType)
    where
        M: OptModel,
        F: OptCallbackFn<M::SolutionType, M::ScoreType>;

    /// Start optimization
    fn run<F>(
        &self,
        model: &M,
        initial_solution_and_score: Option<(M::SolutionType, M::ScoreType)>,
        n_iter: usize,
        time_limit: Duration,
        callback: Option<&F>,
    ) -> AnyResult<(M::SolutionType, M::ScoreType)>
    where
        M: OptModel,
        F: OptCallbackFn<M::SolutionType, M::ScoreType>,
    {
        let (initial_solution, initial_score) = match initial_solution_and_score {
            Some((solution, score)) => (solution, score),
            None => {
                let mut rng = rand::thread_rng();
                model.generate_random_solution(&mut rng)?
            }
        };

        let (initial_solution, initial_score) =
            model.preprocess_solution(initial_solution, initial_score)?;

        let (solution, score) = self.optimize(
            model,
            initial_solution,
            initial_score,
            n_iter,
            time_limit,
            callback,
        );

        let (solution, score) = model.postprocess_solution(solution, score);
        Ok((solution, score))
    }
}

/// Transition probability function
pub trait TransitionProbabilityFn<ST: Ord + Sync + Send + Copy>: Fn(ST, ST) -> f64 {}

impl<F, ST: Ord + Sync + Send + Copy> TransitionProbabilityFn<ST> for F where F: Fn(ST, ST) -> f64 {}
