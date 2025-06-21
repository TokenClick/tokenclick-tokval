//! Report generation module for creating comprehensive financial analysis reports.

use crate::model::*;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::{Table, presets::UTF8_FULL};
use num_format::{Locale, ToFormattedString};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate the complete financial report
pub fn generate_full_report(data: &ReportData) -> String {
    let mut report = String::new();

    // Header
    report.push_str(&format!("{}\n", "=".repeat(82)));
    report
        .push_str("  Valuation and Sensitivity Analysis of Tokenized Future Advertising Revenue\n");
    report.push_str(&format!("{}\n\n", "=".repeat(82)));

    // Executive Summary
    report.push_str(&generate_executive_summary(data));

    // Section 1: Methodology Overview
    report.push_str(&generate_methodology_section());

    // Section 2: Model Assumptions and Inputs
    report.push_str(&generate_assumptions_section(data));

    // Section 3: Risk-Adjusted Discount Rates
    report.push_str(&generate_discount_rates_section(data));

    // Section 4: Baseline Valuation Analysis
    report.push_str(&generate_baseline_valuation_section(data));

    // Section 5: Investor Lift Model and Analysis
    report.push_str(&generate_lift_model_section(data));

    // Section 6: Full Valuation Analysis
    report.push_str(&generate_full_valuation_section(data));

    // Section 7: Key Insights and Risk Factors
    report.push_str(&generate_insights_section(data));

    // Section 8: Conclusion
    report.push_str(&generate_conclusion_section(data));

    report
}

fn generate_executive_summary(data: &ReportData) -> String {
    format!(
        r#"Executive Summary & Strategic Recommendations
--------------------------------------------
This report presents a comprehensive financial model for the valuation of a novel digital asset: tokenized quarterly advertising revenue. The analysis employs discounted cash flow (DCF) methodology with multi-scenario sensitivity analysis to establish fair market value ranges under varying market conditions and investor participation levels.

* Absolute Valuation Range: The fair market value for the total token pool lies between a low of {} and a high of {}.
* Central Estimate ("Most Likely" Valuation): The most probable fair market value is estimated to be {}.
* Analysis of Key Value Drivers and Sensitivities:
    * Market Volatility: Moving from {:.0}% to {:.0}% volatility decreases the asset's valuation by {:.1}%.
    * Investor Lift: Moving from a Low to a High Lift scenario increases the valuation by {:.1}%.
    * Payout Cycle: Extending the payout cycle from 60 to 120 days reduces the valuation by {:.1}%.

"#,
        format_currency(data.summary.min_valuation),
        format_currency(data.summary.max_valuation),
        format_currency(data.summary.central_estimate),
        data.summary.best_volatility.percentage(),
        data.summary.worst_volatility.percentage(),
        data.summary.volatility_impact,
        data.summary.lift_impact,
        data.summary.payout_impact,
    )
}

fn generate_methodology_section() -> &'static str {
    r#"Section 1: Methodology Overview
================================

This valuation employs industry-standard discounted cash flow (DCF) analysis adapted for tokenized revenue streams. The methodology incorporates:

1. Risk-Adjusted Discount Rates: Composed of risk-free rate, volatility premium, and platform/publisher risk premium.
2. Platform Adjustment Factor: Applied to raw revenue forecasts to account for operational realities.
3. Investor Lift Modeling: Quantifies the potential revenue enhancement from investor community participation.
4. Multi-Scenario Analysis: Evaluates outcomes across various distinct combinations of market conditions.

"#
}

fn generate_assumptions_section(data: &ReportData) -> String {
    format!(
        r#"Section 2: Model Assumptions and Inputs
========================================

Core Financial Inputs:
* Publisher's Raw Quarterly Revenue Forecast: {}
* Platform Adjustment Factor: {:.1}%
* Adjusted Baseline Revenue: {}
* Risk-Free Rate: {:.1}%
* Platform/Publisher Risk Premium: {:.1}%

Scenario Parameters:
* Volatility Scenarios: Low (2%), Typical (4%), High (6%), Extreme (10%)
* Payout Timing: 60 days, 90 days, 120 days
* Investor Lift Scenarios: Low, Medium, High

"#,
        format_currency(data.inputs.raw_forecast),
        data.inputs.platform_adjustment_factor * 100.0,
        format_currency(data.summary.adjusted_baseline),
        data.inputs.risk_free_rate * 100.0,
        data.inputs.platform_risk_premium * 100.0,
    )
}

fn generate_discount_rates_section(data: &ReportData) -> String {
    let mut section = String::from(
        r#"Section 3: Risk-Adjusted Discount Rates
========================================

The discount rate calculation follows standard financial theory, incorporating three components:
1. Risk-Free Rate (baseline return for risk-free investments)
2. Volatility Premium (compensation for market uncertainty)
3. Platform/Publisher Risk Premium (specific operational and credit risks)

---
"#,
    );

    section.push_str(&build_discount_rate_table(data));
    section.push_str("\n---\n\n");
    section
}

fn generate_baseline_valuation_section(data: &ReportData) -> String {
    let mut section = String::from(
        r#"Section 4: Baseline Valuation Analysis (No Investor Lift)
==========================================================

The baseline valuation represents the present value of the adjusted revenue stream without considering any potential investor lift effects. This establishes the floor value for the tokenized asset.

---
"#,
    );

    section.push_str(&build_baseline_valuation_table(data));
    section.push_str("\n---\n\n");
    section.push_str("Key Observations:\n");
    section
        .push_str("* Valuations decrease as payout timing extends due to time value of money.\n");
    section.push_str(
        "* Higher volatility scenarios require higher discount rates, reducing present values.\n",
    );
    section.push_str(
        "* The impact of timing becomes more pronounced in high-volatility environments.\n\n",
    );

    section
}

fn generate_lift_model_section(data: &ReportData) -> String {
    let assumptions = &data.lift_assumptions;
    let mut section = format!(
        r#"Section 5: Investor Lift Model and Analysis
============================================

The investor lift model quantifies potential revenue enhancement driven by the token holder community. Rather than assuming a fixed lift, this model is based on tangible drivers, providing a more robust framework for analysis.

Core Lift Model Assumptions:
* Total Investor Count: {}
* Audience Lift per Investor: {:.1} new monthly visitors
* Baseline Monthly Audience: {} unique visitors
* Revenue per Thousand Impressions (RPM): ${:.2}

The model calculates the total potential monthly audience lift and then applies an "Activation Factor" to simulate different levels of community engagement and effectiveness.

---
"#,
        assumptions.investor_count.to_formatted_string(&Locale::en),
        assumptions.lift_per_investor,
        (assumptions.baseline_audience.round() as i64).to_formatted_string(&Locale::en),
        assumptions.rpm
    );

    // Table 3: Lift Scenarios
    section.push_str(&build_lift_scenarios_table(data));
    section.push_str("\n---\n\n");

    // Table 4: Audience Growth
    section.push_str(&build_audience_growth_table(data));
    section.push_str("\n---\n\n");

    // Table 5: Revenue Impact
    section.push_str(&build_revenue_impact_table(data));
    section.push_str("\n---\n\n");

    section
}

fn generate_full_valuation_section(data: &ReportData) -> String {
    let mut section = String::from(
        r#"Section 6: Full Valuation Analysis with Investor Lift
======================================================

The following matrices present the complete valuation analysis incorporating investor lift effects. Each table represents a different lift scenario, showing how tokenized revenue values vary across volatility and payout timing conditions.

"#,
    );

    for lift_scenario in LiftScenario::all() {
        section.push_str(&format!(
            "\n### Valuation Matrix: {} Scenario\n\n",
            lift_scenario
        ));
        section.push_str("---\n");
        section.push_str(&build_valuation_table(data, *lift_scenario));
        section.push_str("\n---\n\n");
    }

    section
}

fn generate_insights_section(data: &ReportData) -> String {
    format!(
        r#"Section 7: Key Insights and Risk Factors
=========================================

Valuation Sensitivities:
1. Time Value Impact: Each 30-day delay in payout reduces valuation by approximately {:.1}%.
2. Volatility Premium: Moving from low to extreme volatility reduces value by {:.1}%.
3. Investor Lift Potential: Active investor participation can enhance value by up to {:.1}%.

Risk Considerations:
* Platform Risk: Operational challenges could impact revenue realization.
* Market Risk: Volatility in digital advertising markets affects cash flows.
* Timing Risk: Delays in payment cycles directly reduce present values.
* Participation Risk: Investor lift depends on community engagement.

Investment Implications:
* The wide valuation range ({} to {}) reflects the nascent nature of tokenized revenue assets.
* The central estimate of {} assumes moderate market conditions and medium investor participation.
* Investors should consider their risk tolerance and market outlook when evaluating entry points.

"#,
        data.summary.payout_impact / 2.0,
        data.summary.volatility_impact,
        data.summary.lift_impact,
        format_currency(data.summary.min_valuation),
        format_currency(data.summary.max_valuation),
        format_currency(data.summary.central_estimate),
    )
}

fn generate_conclusion_section(data: &ReportData) -> String {
    // Basic timestamp generation
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    format!(
        r#"Section 8: Conclusion
=====================

This comprehensive analysis establishes a valuation framework for tokenized advertising revenue, a novel asset class at the intersection of digital media and decentralized finance. The model demonstrates that:

1. Fair market value is highly sensitive to market conditions and structural parameters.
2. The central valuation estimate of {} represents a balanced scenario.
3. Investor participation through the lift mechanism provides significant upside potential.
4. Risk-adjusted returns must account for volatility, timing, and platform-specific factors.

The tokenization of revenue streams represents an innovative approach to media financing, offering publishers immediate liquidity while providing investors exposure to digital advertising growth. As this market matures, we expect valuation methodologies to evolve and risk premiums to compress, potentially enhancing asset values over time.

---
Report Generated Timestamp: {}
Model Version: 0.2.1
"#,
        format_currency(data.summary.central_estimate),
        timestamp,
    )
}

// Helper functions to build tables

fn build_discount_rate_table(data: &ReportData) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Volatility Scenario",
            "Risk-Free Rate",
            "Volatility Premium",
            "Platform Premium",
            "Total Discount Rate",
        ]);

    for volatility in VolatilityScenario::all() {
        let components = &data.discount_rates[volatility];
        table.add_row(vec![
            volatility.to_string(),
            format!("{:.1}%", components.risk_free_rate * 100.0),
            format!("{:.1}%", components.volatility_premium * 100.0),
            format!("{:.1}%", components.platform_risk_premium * 100.0),
            format!("{:.1}%", components.total_rate() * 100.0),
        ]);
    }

    format!(
        "Table 1: Risk-Adjusted Discount Rates by Volatility Scenario\n\n{}",
        table
    )
}

fn build_baseline_valuation_table(data: &ReportData) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Payout Timing",
            "Low Vol (2%)",
            "Typical (4%)",
            "High Vol (6%)",
            "Extreme (10%)",
        ]);

    for payout in PayoutScenario::all() {
        let mut row = vec![payout.to_string()];

        for volatility in VolatilityScenario::all() {
            // Find logic now searches the unified vector for baseline (lift_scenario: None)
            let value = data
                .all_valuations
                .iter()
                .find(|v| {
                    v.payout_scenario == *payout
                        && v.volatility_scenario == *volatility
                        && v.lift_scenario.is_none() // Check for baseline
                })
                .map(|v| format_currency(v.present_value))
                .unwrap_or_else(|| "N/A".to_string());
            row.push(value);
        }

        table.add_row(row);
    }

    format!(
        "Table 2: Baseline Valuation Matrix (No Investor Lift)\n\n{}",
        table
    )
}

fn build_lift_scenarios_table(data: &ReportData) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Lift Scenario",
            "Investor Activation Factor",
            "Resulting Monthly Audience Lift",
        ]);

    let assumptions = &data.lift_assumptions;
    for lift in LiftScenario::all() {
        let audience_lift =
            lift.additional_audience(assumptions.investor_count, assumptions.lift_per_investor);
        table.add_row(vec![
            lift.to_string(),
            format!("{:.0}%", lift.activation_factor() * 100.0),
            (audience_lift.round() as i64).to_formatted_string(&Locale::en),
        ]);
    }

    format!("Table 3: Investor Lift Activation Scenarios\n\n{}", table)
}

fn build_audience_growth_table(data: &ReportData) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Metric",
            "Baseline",
            "Low Lift",
            "Medium Lift",
            "High Lift",
        ]);

    let assumptions = &data.lift_assumptions;

    // Monthly Audience row
    let mut audience_row = vec!["Total Monthly Audience".to_string()];
    audience_row
        .push((assumptions.baseline_audience.round() as i64).to_formatted_string(&Locale::en));
    for lift in LiftScenario::all() {
        let additional_audience =
            lift.additional_audience(assumptions.investor_count, assumptions.lift_per_investor);
        let total = assumptions.baseline_audience + additional_audience;
        audience_row.push((total.round() as i64).to_formatted_string(&Locale::en));
    }
    table.add_row(audience_row);

    // Growth % row
    let mut growth_row = vec!["Growth vs Baseline".to_string()];
    growth_row.push("0.0%".to_string());
    for lift in LiftScenario::all() {
        let additional_audience =
            lift.additional_audience(assumptions.investor_count, assumptions.lift_per_investor);
        let growth_pct = (additional_audience / assumptions.baseline_audience) * 100.0;
        growth_row.push(format!("{:.1}%", growth_pct));
    }
    table.add_row(growth_row);

    format!("Table 4: Audience Growth Under Lift Scenarios\n\n{}", table)
}

fn build_revenue_impact_table(data: &ReportData) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Revenue Component",
            "Low Lift",
            "Medium Lift",
            "High Lift",
        ]);

    let assumptions = &data.lift_assumptions;

    // Baseline Quarterly Revenue
    table.add_row(vec![
        "Baseline Quarterly Revenue".to_string(),
        format_currency(data.summary.adjusted_baseline),
        format_currency(data.summary.adjusted_baseline),
        format_currency(data.summary.adjusted_baseline),
    ]);

    // Additional Quarterly Revenue
    let mut lift_row = vec!["Additional Quarterly Revenue from Lift".to_string()];
    for lift in LiftScenario::all() {
        let quarterly_lift = lift.quarterly_lift(
            assumptions.investor_count,
            assumptions.lift_per_investor,
            assumptions.rpm,
        );
        lift_row.push(format_currency(quarterly_lift));
    }
    table.add_row(lift_row);

    // Total Quarterly Revenue
    let mut total_row = vec!["Total Lifted Quarterly Revenue".to_string()];
    for lift in LiftScenario::all() {
        let quarterly_lift = lift.quarterly_lift(
            assumptions.investor_count,
            assumptions.lift_per_investor,
            assumptions.rpm,
        );
        let total = data.summary.adjusted_baseline + quarterly_lift;
        total_row.push(format_currency(total));
    }
    table.add_row(total_row);

    format!("Table 5: Revenue Impact of Investor Lift\n\n{}", table)
}

fn build_valuation_table(data: &ReportData, lift_scenario: LiftScenario) -> String {
    let table_num = match lift_scenario {
        LiftScenario::Low => 6,
        LiftScenario::Medium => 7,
        LiftScenario::High => 8,
    };

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Payout Timing",
            "Low Vol (2%)",
            "Typical (4%)",
            "High Vol (6%)",
            "Extreme (10%)",
        ]);

    for payout in PayoutScenario::all() {
        let mut row = vec![payout.to_string()];

        for volatility in VolatilityScenario::all() {
            let value = data
                .all_valuations
                .iter()
                .find(|v| {
                    v.payout_scenario == *payout
                        && v.volatility_scenario == *volatility
                        && v.lift_scenario == Some(lift_scenario)
                })
                .map(|v| format_currency(v.present_value))
                .unwrap_or_else(|| "N/A".to_string());
            row.push(value);
        }

        table.add_row(row);
    }

    format!(
        "Table {}: Final Valuation Matrix - {} Scenario\n\n{}",
        table_num, lift_scenario, table
    )
}

/// Formats a f64 value as a currency string, e.g., "$1,234,567"
fn format_currency(value: f64) -> String {
    let rounded_value = value.round() as i64;
    format!("${}", rounded_value.to_formatted_string(&Locale::en))
}
