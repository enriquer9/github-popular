------------------------------
## 🚀 GitHub Popular CLI Tool
A Rust command-line application that finds the most popular public repositories owned by a github account

* Clap CLI Parsing: Implements declarative CLI argument parsing using clap's derive feature for robust input validation.
* Error Handling:   Uses anyhow and thiserror for idiomatic error propagation across async task boundaries.
* Tokio Runtime:    Leverages a multi-threaded asynchronous runtime to handle concurrent worker tasks efficiently without blocking OS threads.

------------------------------
## Getting Started

* Rust
* Cargo

## Installation & Compilation
Clone the repository and build the release binary:

cargo build --release

## Running the Application
Run the CLI using Cargo:

cargo run \<ACCOUNT\>

To see all available CLI options and help text:

cargo run -- --help

------------------------------
## 🛑 Time-Constrained Trade-offs & Future Improvements
With more time, I would implement:

* clap + verbosity + tracing:  Easy setting of verbosity levels corresponding to tracing log levels
