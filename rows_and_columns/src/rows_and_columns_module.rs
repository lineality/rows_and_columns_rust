//! # Memory-Efficient CSV Inspection Tool
//! 
//! This module provides functions for inspecting large CSV files without
//! loading them entirely into memory. It supports header inspection and
//! random access to specific rows using efficient streaming techniques.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::env;
use std::process;

/// Parses command line arguments and executes the appropriate CSV inspection function.
///
/// This function handles the CLI commands, validates arguments, and calls the
/// specific inspection functions based on user input.
///
/// # Commands
///
/// * `--describe <file_path>` - Show column count, header names, and row count
/// * `--row <row_index> <file_path>` - Display a specific row with headers
///
/// # Returns
///
/// * `()` - The function exits the process on error
pub fn csv_inspection_main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    
    // If no arguments are provided (just the program name), start interactive mode
    if args.len() == 1 {
        match q_and_q_tool() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        return;
    }
    
    // Parse the command-line arguments
    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }
    
    // Process based on the command argument
    match args[1].as_str() {
        "--describe" => {
            if args.len() < 3 {
                eprintln!("Error: Missing file path");
                print_usage(&args[0]);
                process::exit(1);
            }
            
            let file_path = &args[2];
            
            // Validate file path exists
            if !Path::new(file_path).exists() {
                eprintln!("Error: File '{}' does not exist", file_path);
                process::exit(1);
            }
            
            // Describe the CSV file
            match describe_csv(file_path) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error describing CSV file: {}", e);
                    process::exit(1);
                }
            }
        },
        "--row" => {
            // Need at least 4 arguments for --row command
            if args.len() < 4 {
                eprintln!("Error: Missing row index or file path");
                print_usage(&args[0]);
                process::exit(1);
            }
            
            // Parse row index
            let row_index = match args[2].parse::<usize>() {
                Ok(idx) => idx,
                Err(_) => {
                    eprintln!("Error: Row index must be a non-negative integer");
                    process::exit(1);
                }
            };
            
            let file_path = &args[3];
            
            // Validate file path exists
            if !Path::new(file_path).exists() {
                eprintln!("Error: File '{}' does not exist", file_path);
                process::exit(1);
            }
            
            // Display the specified row
            match display_csv_row(file_path, row_index) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error displaying row: {}", e);
                    process::exit(1);
                }
            }
        },
        _ => {
            eprintln!("Error: Unknown command '{}'", args[1]);
            print_usage(&args[0]);
            process::exit(1);
        }
    }
}

// Add this function to the rows_and_columns.rs file

/// Provides an interactive question-and-answer interface for the CSV inspection tool.
/// This function is triggered when the program is run without arguments.
///
/// # Returns
///
/// * `Result<(), io::Error>` - Success or an I/O error
pub fn q_and_q_tool() -> Result<(), io::Error> {
    // Print welcome banner
    println!("=== CSV Inspection Tool ===");
    println!();
    
    // Display help information
    println!("Available operations:");
    println!("  1. describe - Show column count, headers, and row count");
    println!("  2. row      - Display a specific row with headers");
    println!();
    
    // Get file path
    print!("1. What is the path to your .csv file? ");
    io::stdout().flush()?; // Ensure prompt is displayed before reading input
    
    let mut file_path = String::new();
    io::stdin().read_line(&mut file_path)?;
    let file_path = file_path.trim();
    
    // Validate file path
    if !Path::new(file_path).exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File not found: '{}'", file_path)
        ));
    }
    
    // Get operation choice
    println!();
    println!("2. What operation do you want?");
    println!("   (by option number)");
    println!("   1. describe file");
    println!("   2. row: print a row");
    print!("Enter option number: ");
    io::stdout().flush()?;
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice = choice.trim();
    
    // Process choice
    match choice {
        "1" => {
            println!("\nExecuting: describe {}\n", file_path);
            describe_csv(file_path)
        },
        "2" => {
            print!("Enter row number to display: ");
            io::stdout().flush()?;
            
            let mut row_index_str = String::new();
            io::stdin().read_line(&mut row_index_str)?;
            
            // Parse row index
            match row_index_str.trim().parse::<usize>() {
                Ok(row_index) => {
                    println!("\nExecuting: row {} {}\n", row_index, file_path);
                    display_csv_row(file_path, row_index)
                },
                Err(_) => {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Invalid row number. Please enter a non-negative integer."
                    ))
                }
            }
        },
        _ => {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid option: '{}'. Please enter 1 or 2.", choice)
            ))
        }
    }
}

/// Prints the tool's usage instructions.
///
/// # Arguments
///
/// * `program_name` - The name of the program
fn print_usage(program_name: &str) {
    eprintln!("Usage:");
    eprintln!("  {} --describe <csv_file_path>", program_name);
    eprintln!("  {} --row <row_index> <csv_file_path>", program_name);
    eprintln!("\nExample:");
    eprintln!("  {} --describe data.csv", program_name);
    eprintln!("  {} --row 5 data.csv", program_name);
}

/// Parses a line of CSV text into separate fields.
///
/// # Arguments
///
/// * `line` - A line of CSV text to parse
///
/// # Returns
///
/// * `Vec<String>` - Vector of parsed fields
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    
    for ch in line.chars() {
        match ch {
            '"' => {
                // Toggle quoted state
                in_quotes = !in_quotes;
            },
            ',' if !in_quotes => {
                // End of field if comma outside quotes
                fields.push(current_field.trim().to_string());
                current_field = String::new();
            },
            _ => {
                // Add any other character to current field
                current_field.push(ch);
            }
        }
    }
    
    // Add the last field
    fields.push(current_field.trim().to_string());
    
    fields
}

/// Describes a CSV file's structure and outputs a summary.
/// Only reads the header row and counts the remaining rows without loading them.
///
/// # Arguments
///
/// * `file_path` - Path to the CSV file to describe
///
/// # Returns
///
/// * `Result<(), io::Error>` - Success or an I/O error
pub fn describe_csv(file_path: impl AsRef<Path>) -> Result<(), io::Error> {
    // Open the file
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    
    // Read only the first line for headers
    let mut header_line = String::new();
    let bytes_read = reader.read_line(&mut header_line)?;
    
    if bytes_read == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Empty CSV file"));
    }
    
    // Parse headers
    let headers = parse_csv_line(&header_line);
    
    // Count remaining lines without loading their content
    let mut row_count = 0;
    let mut line_buffer = String::new();
    
    while reader.read_line(&mut line_buffer)? > 0 {
        if !line_buffer.trim().is_empty() {
            row_count += 1;
        }
        // Clear buffer for next line to minimize memory usage
        line_buffer.clear();
    }
    
    // Calculate column widths for display
    let mut column_widths = Vec::new();
    for header in &headers {
        column_widths.push(header.len());
    }
    
    // Print summary information
    println!("CSV Structure Summary:");
    println!("=====================");
    println!("Number of columns: {}", headers.len());
    println!("Number of data rows: {}", row_count);
    println!();
    
    // Print headers table
    println!("Column Headers:");
    println!("==============");
    print_headers_table(&headers, &column_widths);
    
    Ok(())
}

/// Prints a table showing the headers.
///
/// # Arguments
///
/// * `headers` - Vector of header strings
/// * `column_widths` - Vector of column widths for formatting
fn print_headers_table(headers: &[String], column_widths: &[usize]) {
    // Print top border
    print_horizontal_border(column_widths);
    
    // Print header row
    print!("| ");
    for (i, header) in headers.iter().enumerate() {
        let width = column_widths[i];
        print!("{:<width$} | ", header, width = width);
    }
    println!();
    
    // Print bottom border
    print_horizontal_border(column_widths);
}

/// Displays a specific row from a CSV file with headers.
/// Only reads the header row and the requested row, not the entire file.
///
/// # Arguments
///
/// * `file_path` - Path to the CSV file
/// * `row_index` - Index of the row to display (0-based for data rows, not counting header)
///
/// # Returns
///
/// * `Result<(), io::Error>` - Success or an I/O error
pub fn display_csv_row(file_path: impl AsRef<Path>, row_index: usize) -> Result<(), io::Error> {
    // Open the file
    let file = File::open(&file_path)?;
    let mut reader = BufReader::new(file);
    
    // Read only the first line for headers
    let mut header_line = String::new();
    let bytes_read = reader.read_line(&mut header_line)?;
    
    if bytes_read == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Empty CSV file"));
    }
    
    // Parse headers
    let headers = parse_csv_line(&header_line);
    
    // Skip to the target row (for very large files, this is more efficient than reading each line)
    let mut current_row = 0;
    let mut row_data = Vec::new();
    let mut line_buffer = String::new();
    
    while current_row < row_index {
        // Read a line and check if we reached EOF
        let bytes_read = reader.read_line(&mut line_buffer)?;
        if bytes_read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Row index {} out of range (file has only {} data rows)", row_index, current_row)
            ));
        }
        
        // If line is not empty, count it as a data row
        if !line_buffer.trim().is_empty() {
            current_row += 1;
        }
        
        // Clear the buffer for next line
        line_buffer.clear();
    }
    
    // Read the target row
    reader.read_line(&mut line_buffer)?;
    if !line_buffer.trim().is_empty() {
        row_data = parse_csv_line(&line_buffer);
    } else {
        // Try to read the next non-empty line if current is empty
        while reader.read_line(&mut line_buffer)? > 0 {
            if !line_buffer.trim().is_empty() {
                row_data = parse_csv_line(&line_buffer);
                break;
            }
            line_buffer.clear();
        }
    }
    
    if row_data.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Row {} is empty or couldn't be read", row_index)
        ));
    }
    
    // Calculate column widths (max of header and value width)
    let mut column_widths = Vec::new();
    for (i, header) in headers.iter().enumerate() {
        let header_width = header.len();
        let value_width = if i < row_data.len() { row_data[i].len() } else { 0 };
        column_widths.push(header_width.max(value_width));
    }
    
    // Print table
    println!("Row {} Data:", row_index);
    println!("===========");
    
    // Print top border
    print_horizontal_border(&column_widths);
    
    // Print header row
    print!("| ");
    for (i, header) in headers.iter().enumerate() {
        let width = column_widths[i];
        print!("{:<width$} | ", header, width = width);
    }
    println!();
    
    // Print separator
    print_horizontal_border(&column_widths);
    
    // Print value row
    print!("| ");
    for (i, _) in headers.iter().enumerate() {
        let width = column_widths[i];
        let value = if i < row_data.len() { &row_data[i] } else { "" };
        print!("{:<width$} | ", value, width = width);
    }
    println!();
    
    // Print bottom border
    print_horizontal_border(&column_widths);
    
    Ok(())
}

/// Helper function to print horizontal borders for tables.
///
/// # Arguments
///
/// * `column_widths` - Vector of column widths
fn print_horizontal_border(column_widths: &[usize]) {
    print!("+");
    for width in column_widths {
        print!("-{}-+", "-".repeat(*width));
    }
    println!();
}


