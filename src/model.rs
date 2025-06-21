//! Core data models and types for the valuation system.

use std::collections::HashMap;

/// Payout timing scenarios representing different payment delays
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PayoutScenario {
    /// Payment after 60 days
    Day60,
    /// Payment after 90 days
    Day90,
    /// Payment after 120 days
    Day120,
}

impl PayoutScenario {
    /// Returns all payout scenarios in order.
    pub fn all() -> &'static [Self] {
        &[Self::Day60, Self::Day90, Self::Day120]
    }

    /// Get the time period in years for DCF calculation. Assumes a 365-day year.
    pub fn years(&self) -> f64 {
        match self {
            Self::Day60 => 60.0 / 365.0,
            Self::Day90 => 90.0 / 365.0,
            Self::Day120 => 120.0 / 365.0,
        }
    }

    /// Get days as integer
    #[allow(dead_code)]
    pub fn days(&self) -> u32 {
        match self {
            Self::Day60 => 60,
            Self::Day90 => 90,
            Self::Day120 => 120,
        }
    }
}

/// Display the payout scenario as a string.
impl std::fmt::Display for PayoutScenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Day60 => write!(f, "60 Days"),
            Self::Day90 => write!(f, "90 Days"),
            Self::Day120 => write!(f, "120 Days"),
        }
    }
}

/// Market volatility scenarios affecting risk premium.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VolatilityScenario {
    /// Low market volatility
    Low,
    /// Typical market conditions
    Typical,
    /// High market volatility
    High,
    /// Extreme market conditions
    Extreme,
}

impl VolatilityScenario {
    /// Returns all volatility scenarios in order
    pub fn all() -> &'static [Self] {
        &[Self::Low, Self::Typical, Self::High, Self::Extreme]
    }

    /// Get the volatility premium for this scenario.
    pub fn premium(&self) -> f64 {
        match self {
            Self::Low => 0.05,     // 5%
            Self::Typical => 0.10, // 10%
            Self::High => 0.20,    // 20%
            Self::Extreme => 0.30, // 30%
        }
    }

    /// Get the volatility percentage for display.
    pub fn percentage(&self) -> f64 {
        self.premium() * 100.0
    }
}

/// Implement Display for VolatilityScenario.
impl std::fmt::Display for VolatilityScenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low Vol"),
            Self::Typical => write!(f, "Typical"),
            Self::High => write!(f, "High Vol"),
            Self::Extreme => write!(f, "Extreme"),
        }
    }
}

/// Investor participation lift scenarios, now representing an activation factor.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum LiftScenario {
    /// Low investor activation (e.g., 50% of potential)
    Low,
    /// Medium / Expected investor activation (e.g., 100% of potential)
    Medium,
    /// High investor activation / viral effects (e.g., 150% of potential)
    High,
}

impl LiftScenario {
    /// Returns all lift scenarios
    pub fn all() -> &'static [Self] {
        &[Self::Low, Self::Medium, Self::High]
    }

    /// Get the activation factor for this scenario.
    pub fn activation_factor(&self) -> f64 {
        match self {
            Self::Low => 0.5,    // 50%
            Self::Medium => 1.0, // 100%
            Self::High => 1.5,   // 150%
        }
    }

    /// Calculate the additional monthly audience based on investor drivers.
    pub fn additional_audience(&self, investor_count: u32, lift_per_investor: f64) -> f64 {
        (investor_count as f64) * lift_per_investor * self.activation_factor()
    }

    /// Get the quarterly lift dollar amount (audience * RPM * 3 months)
    pub fn quarterly_lift(&self, investor_count: u32, lift_per_investor: f64, rpm: f64) -> f64 {
        let audience = self.additional_audience(investor_count, lift_per_investor);
        audience * (rpm / 1000.0) * 3.0
    }
}

impl std::fmt::Display for LiftScenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low Lift"),
            Self::Medium => write!(f, "Medium Lift"),
            Self::High => write!(f, "High Lift"),
        }
    }
}

/// Input parameters for valuation calculations
#[derive(Debug, Clone)]
pub struct ValuationInputs {
    // Publisher's raw quarterly revenue forecast
    pub raw_forecast: f64,
    /// Risk-free rate (as decimal, e.g., 0.045 for 4.5%)
    pub risk_free_rate: f64,
    /// Platform risk premium (as decimal)
    pub platform_risk_premium: f64,
    /// Platform adjustment factor (as decimal, negative for reduction)
    pub platform_adjustment_factor: f64,
    /// Baseline monthly audience for lift calculations
    pub baseline_audience: f64,
    /// Revenue per thousand impressions (RPM) for lift calculations
    pub rpm: f64,
    /// Investor count for lift calculations
    pub investor_count: u32,
    /// Lift per investor for lift calculations
    pub lift_per_investor: f64,
}

/// Components used to calculate the discount rate
#[derive(Debug, Clone)]
pub struct DiscountRateComponents {
    /// Base risk-free rate
    pub risk_free_rate: f64,
    /// Premium based on market volatility
    pub volatility_premium: f64,
    /// Premium for platform and publisher risk
    pub platform_risk_premium: f64,
}

impl DiscountRateComponents {
    /// Calculate the total discount rate
    pub fn total_rate(&self) -> f64 {
        self.risk_free_rate + self.volatility_premium + self.platform_risk_premium
    }
}

/// Result of a single valuation calculation
#[derive(Debug, Clone)]
pub struct ValuationResult {
    /// Calculated present value
    pub present_value: f64,
    /// Payout timing scenario used
    pub payout_scenario: PayoutScenario,
    /// Volatility scenario used
    pub volatility_scenario: VolatilityScenario,
    /// Lift scenario used
    pub lift_scenario: Option<LiftScenario>,
}

/// Comprehensive data structure containing all report data
#[derive(Debug, Clone)]
pub struct ReportData {
    /// Original inputs
    pub inputs: ValuationInputs,
    /// A unified list of all valuation results across all scenarios.
    pub all_valuations: Vec<ValuationResult>,
    /// Discount rates for each volatility scenario
    pub discount_rates: HashMap<VolatilityScenario, DiscountRateComponents>,
    /// Summary statistics
    pub summary: SummaryStatistics,
    /// Lift model assumptions
    pub lift_assumptions: LiftAssumptions,
}

/// Summary statistics for the executive summary
#[derive(Debug, Clone)]
pub struct SummaryStatistics {
    /// Minimum valuation across all scenarios
    pub min_valuation: f64,
    /// Maximum valuation across all scenarios
    pub max_valuation: f64,
    /// Central estimate (typical volatility, 90 days, medium lift)
    pub central_estimate: f64,
    /// Best case volatility scenario
    pub best_volatility: VolatilityScenario,
    /// Worst case volatility scenario
    pub worst_volatility: VolatilityScenario,
    /// Impact of volatility change (as percentage)
    pub volatility_impact: f64,
    /// Impact of lift change from low to high (as percentage)
    pub lift_impact: f64,
    /// Impact of payout cycle extension from 60 to 120 days (as percentage)
    pub payout_impact: f64,
    /// Adjusted baseline revenue
    pub adjusted_baseline: f64,
}

/// Assumptions for the lift model
#[derive(Debug, Clone)]
pub struct LiftAssumptions {
    /// Base monthly audience
    pub baseline_audience: f64,
    /// Revenue per thousand impressions
    pub rpm: f64,
    /// Number of investors
    pub investor_count: u32,
    /// Lift per investor
    pub lift_per_investor: f64,
}
