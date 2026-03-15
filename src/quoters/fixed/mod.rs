use serde::Deserialize;

#[derive(Debug, PartialEq)]
pub struct FixedTracker {
    pub token_in: String,
    pub token_out: String,
    pub fixed_rate: f64,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct FixedTrackerConfig {
    pub token_in: String,
    pub token_out: String,
    pub fixed_rate: f64,
}
