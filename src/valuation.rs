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

    // Calculate baseline valuations (no lift)
    let mut baseline_valuations = Vec::new();
    for &payout in PayoutScenario::all() {
        for &volatility in VolatilityScenario::all() {
            let discount_rate = discount_rates[&volatility].total_rate();
            let present_value =
                calculate_present_value(adjusted_baseline, discount_rate, payout.years())?;

            baseline_valuations.push(ValuationResult {
                present_value,
                payout_scenario: payout,
                volatility_scenario: volatility,
                lift_scenario: None,
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
        let mut scenario_results = Vec::new();
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

                scenario_results.push(ValuationResult {
                    present_value,
                    payout_scenario: payout,
                    volatility_scenario: volatility,
                    lift_scenario: Some(lift_scenario),
                });
            }
        }

        lift_valuations.insert(lift_scenario, scenario_results);
    }

    // Calculate summary statistics
    let summary =
        calculate_summary_statistics(&baseline_valuations, &lift_valuations, adjusted_baseline)?;

    Ok(ReportData {
        inputs: inputs.clone(),
        baseline_valuations,
        lift_valuations,
        discount_rates,
        summary,
        lift_assumptions,
    })
}

/// Calculate summary statistics for the final summary
fn calculate_summary_statistics(
    baseline_valuations: &[ValuationResult],
    lift_valuations: &HashMap<LiftScenario, Vec<ValuationResult>>,
    adjusted_baseline: f64,
) -> Result<SummaryStatistics, ModelError> {
    // Find min and max across all scenarios
    let mut all_values = Vec::new();
    all_values.extend(baseline_valuations.iter().map(|v| v.present_value));
    for (_, results) in lift_valuations {
        all_values.extend(results.iter().map(|v| v.present_value));
    }

    let min_valuation = all_values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_valuation = all_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    // Central estimate: typical volatility, 90 days, medium lift
    let central_estimate = lift_valuations[&LiftScenario::Medium]
        .iter()
        .find(|v| {
            v.payout_scenario == PayoutScenario::Day90
                && v.volatility_scenario == VolatilityScenario::Typical
        })
        .ok_or_else(|| ModelError::CalculationError("Could not find central estimate".to_string()))?
        .present_value;

    // Volatility impact: compare low vs extreme at 90 days, medium lift
    let low_vol_value = lift_valuations[&LiftScenario::Medium]
        .iter()
        .find(|v| {
            v.payout_scenario == PayoutScenario::Day90
                && v.volatility_scenario == VolatilityScenario::Low
        })
        .ok_or_else(|| {
            ModelError::CalculationError("Could not find low volatility value".to_string())
        })?
        .present_value;

    let extreme_vol_value = lift_valuations[&LiftScenario::Medium]
        .iter()
        .find(|v| {
            v.payout_scenario == PayoutScenario::Day90
                && v.volatility_scenario == VolatilityScenario::Extreme
        })
        .ok_or_else(|| {
            ModelError::CalculationError("Could not find extreme volatility value".to_string())
        })?
        .present_value;

    let volatility_impact = ((low_vol_value - extreme_vol_value) / low_vol_value) * 100.0;

    // Lift impact: compare low vs high at 90 days, typical volatility
    let low_lift_value = lift_valuations[&LiftScenario::Low]
        .iter()
        .find(|v| {
            v.payout_scenario == PayoutScenario::Day90
                && v.volatility_scenario == VolatilityScenario::Typical
        })
        .ok_or_else(|| ModelError::CalculationError("Could not find low lift value".to_string()))?
        .present_value;

    let high_lift_value = lift_valuations[&LiftScenario::High]
        .iter()
        .find(|v| {
            v.payout_scenario == PayoutScenario::Day90
                && v.volatility_scenario == VolatilityScenario::Typical
        })
        .ok_or_else(|| ModelError::CalculationError("Could not find high lift value".to_string()))?
        .present_value;

    let lift_impact = ((high_lift_value - low_lift_value) / low_lift_value) * 100.0;

    // Payout impact: compare 60 vs 120 days at typical volatility, medium lift
    let day60_value = lift_valuations[&LiftScenario::Medium]
        .iter()
        .find(|v| {
            v.payout_scenario == PayoutScenario::Day60
                && v.volatility_scenario == VolatilityScenario::Typical
        })
        .ok_or_else(|| ModelError::CalculationError("Could not find 60-day value".to_string()))?
        .present_value;

    let day120_value = lift_valuations[&LiftScenario::Medium]
        .iter()
        .find(|v| {
            v.payout_scenario == PayoutScenario::Day120
                && v.volatility_scenario == VolatilityScenario::Typical
        })
        .ok_or_else(|| ModelError::CalculationError("Could not find 120-day value".to_string()))?
        .present_value;

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
