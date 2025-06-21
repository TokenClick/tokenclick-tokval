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

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    /// Creates a default, valid set of ValuationInputs for use in tests.
    /// Includes the new investor-centric lift model parameters.
    fn get_default_inputs() -> ValuationInputs {
        ValuationInputs {
            raw_forecast: 220_000.0,
            risk_free_rate: 0.045,
            platform_risk_premium: 0.12,
            platform_adjustment_factor: -0.091,
            baseline_audience: 1_000_000.0,
            rpm: 15.0,
            investor_count: 1000,
            lift_per_investor: 10.0,
        }
    }

    #[test]
    fn test_adjusted_baseline_calculation() {
        let baseline = calculate_adjusted_baseline(100_000.0, -0.091);
        assert_relative_eq!(baseline, 90_900.0, epsilon = 0.01);
    }

    #[test]
    fn test_present_value_calculation() {
        // Test with known values for a full year
        let pv_full_year = calculate_present_value(100_000.0, 0.10, 1.0).unwrap();
        assert_relative_eq!(pv_full_year, 90_909.09, epsilon = 0.01);

        // Test with 90 days, ensuring the expected value matches the precise calculation.
        let time_period = 90.0 / 365.0; // approx 0.246575
        let pv_90_days = calculate_present_value(100_000.0, 0.10, time_period).unwrap();

        // The expected value is updated to the correct result of the calculation.
        assert_relative_eq!(pv_90_days, 97_677.29, epsilon = 0.01);
    }

    #[test]
    /// Tests the calculation of additional audience based on the new driver-based model.
    fn test_additional_audience_calculation() {
        let investor_count = 1000;
        let lift_per_investor = 10.0;

        // Medium scenario has an activation factor of 1.0
        let medium_lift_audience =
            LiftScenario::Medium.additional_audience(investor_count, lift_per_investor);
        // Expected: 1000 investors * 10 lift/investor * 1.0 factor = 10,000
        assert_relative_eq!(medium_lift_audience, 10_000.0);

        // High scenario has an activation factor of 1.5
        let high_lift_audience =
            LiftScenario::High.additional_audience(investor_count, lift_per_investor);
        // Expected: 1000 investors * 10 lift/investor * 1.5 factor = 15,000
        assert_relative_eq!(high_lift_audience, 15_000.0);
    }

    #[test]
    /// Tests the quarterly lift revenue calculation using the new model.
    fn test_quarterly_lift_revenue_calculation() {
        let investor_count = 1000;
        let lift_per_investor = 10.0;
        let rpm = 20.0; // Use a custom RPM for the test

        // Test Medium Lift (activation factor 1.0)
        let medium_lift_revenue =
            LiftScenario::Medium.quarterly_lift(investor_count, lift_per_investor, rpm);
        // Expected Audience: 1000 * 10 * 1.0 = 10,000
        // Expected Revenue: (10,000 / 1000) * $20 RPM * 3 months = $600
        assert_relative_eq!(medium_lift_revenue, 600.0);

        // Test Low Lift (activation factor 0.5)
        let low_lift_revenue =
            LiftScenario::Low.quarterly_lift(investor_count, lift_per_investor, rpm);
        // Expected Audience: 1000 * 10 * 0.5 = 5,000
        // Expected Revenue: (5,000 / 1000) * $20 RPM * 3 months = $300
        assert_relative_eq!(low_lift_revenue, 300.0);
    }

    #[test]
    /// An integration test to verify the end-to-end calculation with custom inputs
    /// for the new, more sophisticated lift model.
    fn test_full_valuation_with_custom_lift_drivers() {
        let mut inputs = get_default_inputs();
        // Override default inputs with custom test values
        inputs.rpm = 25.0;
        inputs.investor_count = 2000;
        inputs.lift_per_investor = 15.0;

        let report_data = calculate_full_valuation(&inputs).unwrap();

        // 1. Check that the report's assumptions reflect the custom inputs.
        assert_relative_eq!(report_data.lift_assumptions.rpm, 25.0);
        assert_eq!(report_data.lift_assumptions.investor_count, 2000);
        assert_relative_eq!(report_data.lift_assumptions.lift_per_investor, 15.0);

        // 2. Manually calculate the expected result for a single, known scenario (the central estimate).
        // Scenario: Medium Lift, Typical Volatility, 90 Day Payout

        // Calculate expected lift revenue using the new model
        let expected_lift = LiftScenario::Medium.quarterly_lift(
            inputs.investor_count,
            inputs.lift_per_investor,
            inputs.rpm,
        );
        // Expected Audience: 2000 * 15 * 1.0 = 30,000
        // Expected Revenue: (30,000 / 1000) * $25 RPM * 3 months = $2,250
        assert_relative_eq!(expected_lift, 2_250.0);

        // Calculate total cash flow for this scenario
        let adjusted_baseline =
            calculate_adjusted_baseline(inputs.raw_forecast, inputs.platform_adjustment_factor);
        let expected_lifted_revenue = adjusted_baseline + expected_lift;

        // Calculate the discount rate for this scenario
        let components = calculate_discount_rate(&inputs, VolatilityScenario::Typical);
        let discount_rate = components.total_rate();
        let time_years = PayoutScenario::Day90.years();

        // Calculate the final expected Present Value
        let expected_pv =
            calculate_present_value(expected_lifted_revenue, discount_rate, time_years).unwrap();

        // 3. Assert that the central estimate calculated by the main function matches our manual calculation.
        let central_estimate = report_data.summary.central_estimate;
        assert_relative_eq!(central_estimate, expected_pv, epsilon = 0.01);
    }
}
