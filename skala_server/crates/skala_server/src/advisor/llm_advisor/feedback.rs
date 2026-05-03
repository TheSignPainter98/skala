use std::fmt::Display;

use crate::{FeedbackConfig, FeedbackRegime};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Feedback {
    content: String,
}

impl Display for Feedback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { content } = self;
        write!(f, "{content}")
    }
}

#[derive(Clone, Debug)]
pub(crate) struct FeedbackProvider {
    feedback_regime: FeedbackRegime,
    positive_feedback_pool: Vec<Feedback>,
    negative_feedback_pool: Vec<Feedback>,
}

impl FeedbackProvider {
    pub(crate) fn new(feedback_regime: FeedbackRegime, config: FeedbackConfig) -> Self {
        let FeedbackConfig {
            positive: positive_feedback_pool,
            negative: negative_feedback_pool,
        } = config;
        Self {
            feedback_regime,
            positive_feedback_pool,
            negative_feedback_pool,
        }
    }

    pub(crate) fn feedback(&self) -> impl Iterator<Item = &Feedback> + Send {
        let pool = match self.feedback_regime {
            FeedbackRegime::Absent => &[],
            FeedbackRegime::Positive => self.positive_feedback_pool.as_slice(),
            FeedbackRegime::Negative => self.negative_feedback_pool.as_slice(),
        };
        const SEED: u64 = 99;
        FeedbackGenerator { pool, index: SEED }
    }
}

/// A pseudo-random feedback generator.
struct FeedbackGenerator<'pool> {
    pool: &'pool [Feedback],
    index: u64,
}

impl<'pool> Iterator for FeedbackGenerator<'pool> {
    type Item = &'pool Feedback;

    fn next(&mut self) -> Option<Self::Item> {
        let Self { pool, index } = self;

        if pool.is_empty() {
            return None;
        }

        *index = index.wrapping_mul(997) % 1223;
        Some(&pool[*index as usize % pool.len()])
    }
}
