//! Command-line interface definitions for tokval.
use clap::Parser;

/// Token Valuator - Calculate fair present value of tokenized quarterly ad revenue
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Publisher's raw quarterly revenue forecast
    #[arg(short, long)]
    pub forecast: f64,

    /// Risk-free rate (as percentage, e.g., 4.5 for 4.5%)
    #[arg(short, long, default_value = "4.5")]
    pub risk_free_rate: f64,

    /// Platform risk premium (as percentage)
    #[arg(short = 'p', long, default_value = "12.0")]
    pub platform_risk_premium: f64,

    /// Platform adjustment factor (as percentage, negative for reduction)
    #[arg(short = 'a', long, default_value = "-9.1")]
    pub platform_adjustment: f64,
}
