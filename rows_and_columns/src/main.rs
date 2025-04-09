//! # Memory-Efficient CSV Inspection Tool
//! 
//! A command-line tool for inspecting CSV files without loading them entirely
//! into memory. Ideal for working with massive datasets that would crash
//! general-purpose software.
//! 
//! ## Usage
//! 
//! ```
//! # To describe a CSV file (headers and row count):
//! cargo run -- --describe path/to/large_file.csv
//! 
//! # To display a specific row:
//! cargo run -- --row 5 path/to/large_file.csv
//! ```

mod rows_and_columns_module;

/// Main entry point for the CSV inspection tool.
fn main() {
    // Delegate all functionality to the module
    rows_and_columns_module::csv_inspection_main();
}
