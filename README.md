# `tokval` - Tokenized Revenue Valuation Tool

`tokval` is a command-line tool for calculating the fair present value of a publisher's tokenized quarterly advertising revenue. It performs a comprehensive sensitivity analysis across dozens of scenarios and generates a detailed report.

## Core Features

  - Calculates present value using a standard Discounted Cash Flow (DCF) model.
  - Runs a sensitivity analysis across 48+ scenarios (payout timings, market volatility, investor engagement).
  - All financial assumptions are configurable via command-line arguments.
  - Models "investor lift" to quantify how token holders can drive revenue growth.
  - Generates a detailed, multi-section text report summarizing the valuation.

## Installation and Setup

To run `tokval`, you first need to install the Rust programming language toolchain and then build the project from the source code.

### Step 1: Install Rust

If you don't have Rust installed, open your terminal and run the following command. This will download and run `rustup`, the official Rust installer.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the on-screen instructions to complete the installation. This command works for macOS, Linux, and Windows Subsystem for Linux (WSL). For Windows, you can also download the installer from [rust-lang.org](https://www.rust-lang.org/tools/install).

After installation, ensure your terminal is restarted or reloaded for the changes to take effect.

### Step 2: Clone and Build `tokval`

Next, download the `tokval` source code and compile it.

```sh
# 1. Clone the repository from GitHub
git clone <repository_url>

# 2. Navigate into the project directory
cd tokval

# 3. Build the project in release mode for maximum performance
cargo build --release
```

This process will compile the code and place the final executable file in the `target/release/` directory.

## How to Run `tokval`

Once the project is built, you can run the program.

### Option A: Run Directly (From Project Directory)

You can execute the binary by specifying its full path within the project folder. All commands and flags follow the program name.

```sh
# Example of running with the required --forecast flag
./target/release/tokenclick-tokval --forecast 220000
```

> **Note:** On Windows, the executable will be `tokenclick-tokval.exe`, so the command would be `.\target\release\tokenclick-tokval.exe --forecast 220000`.

### Option B: Install for Easy Access (Recommended)

For more convenient use, you can copy the `tokenclick-tokval` executable to a directory that is in your system's `PATH`. The standard location for Cargo-installed binaries is a good choice.

```sh
# This command copies the binary to your Cargo home directory
cp target/release/tokenclick-tokval ~/.cargo/bin/

# Now you can call it from anywhere on your system
tokenclick-tokval --forecast 220000
```

## Command-Line Arguments

The tool is configured using the following arguments. Only `--forecast` is required.

| Argument | Flag(s) | Description | Default |
|---|---|---|---|
| **Forecast** | `-f`, `--forecast` | **(Required)** Publisher's raw quarterly revenue forecast. | N/A |
| **Risk-Free Rate** | `-r`, `--risk-free-rate` | Risk-free rate as a percentage (e.g., 4.5 for 4.5%). | `4.5` |
| **Platform Risk Premium** | `-p`, `--platform-risk-premium` | Platform risk premium as a percentage. | `12.0` |
| **Platform Adjustment** | `-a`, `--platform-adjustment` | Platform adjustment factor as a percentage. | `-9.1` |
| **Baseline Audience** | `--baseline-audience` | Baseline monthly audience for lift model calculations. | `1000000` |
| **RPM** | `--rpm` | Revenue per thousand impressions (RPM) for the lift model. | `15.0` |
| **Investor Count** | `--investor-count` | Estimated number of token investors to model lift. | `1000` |
| **Lift per Investor** | `--lift-per-investor` | Estimated new audience members generated per active investor. | `10` |

### Usage Examples

1.  **Basic valuation with default settings:**

    ```sh
    ./target/release/tokval --forecast 250000
    ```

2.  **Valuation with a higher risk profile:**

    ```sh
    ./target/release/tokval -f 300000 -r 5.2 -p 15.0
    ```

3.  **Full customization of lift model and financials:**
    ```sh
    ./target/release/tokval --forecast 500000 --investor-count 500 --lift-per-investor 20 --rpm 25
    ```

## Financial Model Overview

The valuation is based on three core concepts:

1.  **Adjusted Cash Flow:** The raw forecast is modified by the `platform-adjustment` factor.
2.  **Risk-Adjusted Discount Rate:** A rate calculated from the `risk-free-rate`, a fixed `platform-risk-premium`, and a variable `volatility-premium` determined by the scenario.
3.  **Investor Lift:** Additional revenue is calculated based on the number of investors, their assumed effectiveness (`lift-per-investor`), their engagement level (the scenario), and the `rpm`. This is added to the baseline cash flow.

The final present value for each scenario is calculated by discounting the total cash flow over the specified time period.

## For Developers

### Project Structure

  - `main.rs`: Application entry point and orchestration.
  - `cli.rs`: Command-line argument definitions (`clap`).
  - `model.rs`: Core data structures and enums.
  - `valuation.rs`: The financial calculation engine.
  - `report_generator.rs`: Builds the final text report.
  - `error.rs`: Custom error handling types.

### Building and Testing

  - **Build for development:** `cargo build`
  - **Run unit tests:** `cargo test`
  - **Build an optimized release version:** `cargo build --release`
  - **Check formatting and lints:** `cargo fmt -- --check` and `cargo clippy`

## License

This project is licensed under the Apache License, Version 2.0.
