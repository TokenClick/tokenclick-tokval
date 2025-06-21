//! Core valuation logic and calculations.

use crate::error::ModelError;
use crate::model::*;
use std::collections::HashMap;

/// Calculate the adjusted baseline revenue after platform adjustment
fn calculate_adjusted_baseline(raw_forecast: f64, adjustment_factor: f64) -> f64 {
    raw_forecast * (1.0 + adjustment_factor)
}

/// Calculate the discount rate components for a given volatility scenario
fn calculate_discount_rate(
    inputs: &ValuationInputs,
    volatility_scenario: VolatilityScenario,
) -> DiscountRateComponents {
    DiscountRateComponents {
        risk_free_rate: inputs.risk_free_rate,
        volatility_premium: volatility_scenario.premium(),
        platform_risk_premium: inputs.platform_risk_premium,
    }
}

/// Calculate present value using discounted cash flow formula
/// PV = CashFlow / (1 + Rate)^Time
fn calculate_present_value(
    cash_flow: f64,
    discount_rate: f64,
    time_years: f64,
) -> Result<f64, ModelError> {
    if discount_rate < -1.0 {
        return Err(ModelError::CalculationError(
            "Discount rate would result in division by zero or negative denominator".to_string(),
        ));
    }

    let denominator = (1.0 + discount_rate).powf(time_years);
    if denominator == 0.0 {
        return Err(ModelError::CalculationError(
            "Present value calculation resulted in division by zero".to_string(),
        ));
    }

    Ok(cash_flow / denominator)
}

/// Calculate valuations for all scenario combinations and return comprehensive report data
pub fn calculate_full_valuation(inputs: &ValuationInputs) -> Result<ReportData, ModelError> {
    // Validate inputs
    if inputs.raw_forecast <= 0.0 {
        return Err(ModelError::InvalidInput(
            "Raw forecast must be positive".to_string(),
        ));
    }
    if inputs.baseline_audience <= 0.0 {
        return Err(ModelError::InvalidInput(
            "Baseline audience must be positive".to_string(),
        ));
    }
    if inputs.rpm <= 0.0 {
        return Err(ModelError::InvalidInput("RPM must be positive".to_string()));
    }

    // Calculate adjusted baseline revenue
    let adjusted_baseline =
        calculate_adjusted_baseline(inputs.raw_forecast, inputs.platform_adjustment_factor);

    // Calculate discount rates for all volatility scenarios
    let mut discount_rates = HashMap::new();
    for &volatility in VolatilityScenario::all() {
        discount_rates.insert(volatility, calculate_discount_rate(inputs, volatility));
    }

    let mut all_valuations = Vec::new();

    // Calculate baseline valuations (no lift) and add to the unified vector
    for &payout in PayoutScenario::all() {
        for &volatility in VolatilityScenario::all() {
            let discount_rate = discount_rates[&volatility].total_rate();
            let present_value =
                calculate_present_value(adjusted_baseline, discount_rate, payout.years())?;

            all_valuations.push(ValuationResult {
                present_value,
                payout_scenario: payout,
                volatility_scenario: volatility,
                lift_scenario: None, // `None` for baseline
            });
        }
    }

    // Calculate valuations for each lift scenario using configurable inputs
    let mut lift_valuations = HashMap::new();
    let lift_assumptions = LiftAssumptions {
        baseline_audience: inputs.baseline_audience,
        rpm: inputs.rpm,
        investor_count: inputs.investor_count,
        lift_per_investor: inputs.lift_per_investor,
    };

    for &lift_scenario in LiftScenario::all() {
        let scenario_results: Vec<ValuationResult> = Vec::new();
        let lift_amount = lift_scenario.quarterly_lift(
            lift_assumptions.investor_count,
            lift_assumptions.lift_per_investor,
            lift_assumptions.rpm,
        );
        let lifted_revenue = adjusted_baseline + lift_amount;

        for &payout in PayoutScenario::all() {
            for &volatility in VolatilityScenario::all() {
                let discount_rate = discount_rates[&volatility].total_rate();
                let present_value =
                    calculate_present_value(lifted_revenue, discount_rate, payout.years())?;

                all_valuations.push(ValuationResult {
                    present_value,
                    payout_scenario: payout,
                    volatility_scenario: volatility,
                    lift_scenario: Some(lift_scenario), // Set the specific lift scenario
                });
            }
        }

        lift_valuations.insert(lift_scenario, scenario_results);
    }

    // Calculate summary statistics from the unified vector
    let summary = calculate_summary_statistics(&all_valuations, adjusted_baseline)?;

    Ok(ReportData {
        inputs: inputs.clone(),
        all_valuations, // Pass the single unified vector
        discount_rates,
        summary,
        lift_assumptions,
    })
}

/// Calculate summary statistics for the executive summary
fn calculate_summary_statistics(
    all_valuations: &[ValuationResult], // Takes the single unified vector
    adjusted_baseline: f64,
) -> Result<SummaryStatistics, ModelError> {
    let all_values: Vec<f64> = all_valuations.iter().map(|v| v.present_value).collect();

    let min_valuation = all_values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_valuation = all_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    // The find logic now uses the `.lift_scenario` field, fixing the dead code warning!
    let find_value =
        |payout: PayoutScenario, vol: VolatilityScenario, lift: Option<LiftScenario>| {
            all_valuations
                .iter()
                .find(|v| {
                    v.payout_scenario == payout
                        && v.volatility_scenario == vol
                        && v.lift_scenario == lift
                })
                .map(|v| v.present_value)
                .ok_or_else(|| {
                    ModelError::CalculationError(format!("Could not find value for scenario combo"))
                })
        };

    let central_estimate = find_value(
        PayoutScenario::Day90,
        VolatilityScenario::Typical,
        Some(LiftScenario::Medium),
    )?;

    let low_vol_value = find_value(
        PayoutScenario::Day90,
        VolatilityScenario::Low,
        Some(LiftScenario::Medium),
    )?;
    let extreme_vol_value = find_value(
        PayoutScenario::Day90,
        VolatilityScenario::Extreme,
        Some(LiftScenario::Medium),
    )?;
    let volatility_impact = ((low_vol_value - extreme_vol_value) / low_vol_value) * 100.0;

    let low_lift_value = find_value(
        PayoutScenario::Day90,
        VolatilityScenario::Typical,
        Some(LiftScenario::Low),
    )?;
    let high_lift_value = find_value(
        PayoutScenario::Day90,
        VolatilityScenario::Typical,
        Some(LiftScenario::High),
    )?;
    let lift_impact = ((high_lift_value - low_lift_value) / low_lift_value) * 100.0;

    let day60_value = find_value(
        PayoutScenario::Day60,
        VolatilityScenario::Typical,
        Some(LiftScenario::Medium),
    )?;
    let day120_value = find_value(
        PayoutScenario::Day120,
        VolatilityScenario::Typical,
        Some(LiftScenario::Medium),
    )?;
    let payout_impact = ((day60_value - day120_value) / day60_value) * 100.0;

    Ok(SummaryStatistics {
        min_valuation,
        max_valuation,
        central_estimate,
        best_volatility: VolatilityScenario::Low,
        worst_volatility: VolatilityScenario::Extreme,
        volatility_impact,
        lift_impact,
        payout_impact,
        adjusted_baseline,
    })
}
