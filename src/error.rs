//! Error types for the valuation model.

use thiserror::Error;

/// Errors that can occur during valuation calculations
#[derive(Error, Debug)]
pub enum ModelError {
    /// Invalid input parameter provided
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Error during calculation
    #[error("Calculation error: {0}")]
    CalculationError(String),
}
