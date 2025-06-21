//! tokval - Token Valuator
//!
//! A command-line tool for calculating the fair present value of a publisher's
//! tokenized quarterly ad revenue with comprehensive sensitivity analysis and reporting.

mod cli;
mod error;
mod model;
mod report_generator;
mod valuation;

use clap::Parser;
use cli::Args;
use error::ModelError;
use model::ValuationInputs;
use report_generator::generate_full_report;
use valuation::calculate_full_valuation;

fn main() -> Result<(), ModelError> {
    let args = Args::parse();

    let inputs = ValuationInputs {
        raw_forecast: args.forecast,
        risk_free_rate: args.risk_free_rate / 100.0,
        platform_risk_premium: args.platform_risk_premium / 100.0,
        platform_adjustment_factor: args.platform_adjustment / 100.0,
        baseline_audience: args.baseline_audience,
        rpm: args.rpm,
        investor_count: args.investor_count,
        lift_per_investor: args.lift_per_investor,
    };

    // Calculate all valuation data
    let report_data = calculate_full_valuation(&inputs)?;

    // Generate and print the full report
    let report = generate_full_report(&report_data);
    println!("{}", report);

    Ok(())
}
