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

    /// Baseline monthly audience for lift model calculations
    #[arg(long, default_value = "1000000")]
    pub baseline_audience: f64,

    /// Revenue per thousand impressions (RPM) for lift model calculations
    #[arg(long, default_value = "15.0")]
    pub rpm: f64,

    /// --- NEW ---
    /// Estimated number of token investors to model lift
    #[arg(long, default_value = "1000")]
    pub investor_count: u32,

    /// --- NEW ---
    /// Estimated new audience members generated per active investor per month
    #[arg(long, default_value = "10")]
    pub lift_per_investor: f64,
}
