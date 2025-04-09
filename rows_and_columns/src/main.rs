//! # CSV Inspection Tool
//! 
//! A command-line tool for inspecting CSV files, displaying
//! basic information about structure and contents.
//! 
//! ## Usage
//! 
//! ```
//! # To describe a CSV file:
//! cargo run -- --describe path/to/file.csv
//! 
//! # To display a specific row:
//! cargo run -- --row 5 path/to/file.csv
//! ```

mod rows_and_columns_module;

/// Main entry point for the CSV inspection tool.
fn main() {
    // Delegate all functionality to the module
    rows_and_columns_module::csv_inspection_main();
}
