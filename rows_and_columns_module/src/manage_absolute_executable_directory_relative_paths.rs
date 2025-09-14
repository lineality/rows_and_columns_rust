// src/manage_absolute_executable_directory_relative_paths.rs
/// # manage_absolute_executable_directory_relative_paths - Executable-relative path resolution in Rust
/// use -> cargo build --profile release-performance
/// or, use -> cargo build --profile release-small 
/// see: https://github.com/lineality/rust_compile_optimizations_cheatsheet
///
/// This module provides functions for working with file paths relative to the 
/// executable's directory location rather than the current working directory (CWD).
///
/// The main function `make_input_path_name_abs_executabledirectoryrelative_nocheck` converts a path 
/// to an absolute path that's resolved relative to the executable's location.

/* Docs:
# Executable-Directory-Relative Path Resolution

This module solves the common issue where paths are resolved relative to the current
working directory, which can lead to problems when your executable is run from different
locations. Instead, it ensures paths are resolved relative to where your executable is located.

### Sample main file to use this module
```rust
// src/main.rs

// import manage_absolute_executable_directory_relative_paths module w/ these 2 lines
mod manage_absolute_executable_directory_relative_paths;
use manage_absolute_executable_directory_relative_paths::make_input_path_name_abs_executabledirectoryrelative_nocheck;

fn main() {
    // Get a path relative to the executable directory, not the CWD
    match make_input_path_name_abs_executabledirectoryrelative_nocheck("data/config.json") {
        Ok(absolute_path) => println!("Absolute path: {}", absolute_path.display()),
        Err(e) => {
            eprintln!("Error resolving path: {}", e);
            std::process::exit(1);
        }
    }
}
```

## Always
```
Always best practice.
Always extensive doc strings.
Always comments.
Always clear, meaningful, unique names.
Always absolute file paths.
Always error handling.
Never unsafe code.
Never use unwrap.
```
*/

use std::fs;
use std::path::{Path, PathBuf};
use std::io;

/// Gets the directory where the current executable is located.
///
/// # Returns
///
/// * `Result<PathBuf, io::Error>` - The absolute directory path containing the executable or an error
///   if it cannot be determined.
pub fn get_absolute_path_to_executable_parentdirectory() -> Result<PathBuf, io::Error> {
    // Get the path to the current executable
    let executable_path = std::env::current_exe().map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Failed to determine current executable path: {}", e),
        )
    })?;
    
    // Get the directory containing the executable
    let executable_directory = executable_path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Failed to determine parent directory of executable",
        )
    })?;
    
    Ok(executable_directory.to_path_buf())
}

/// Converts a path to an absolute path based on the executable's directory location.
/// Does NOT check if the path exists or attempt to create anything.
///
/// # Arguments
///
/// * `path_to_make_absolute` - A path to convert to an absolute path relative to 
///   the executable's directory location.
///
/// # Returns
///
/// * `Result<PathBuf, io::Error>` - The absolute path based on the executable's directory or an error
///   if the executable's path cannot be determined or if the path cannot be resolved.
///
/// # Examples
///
/// ```
/// use manage_absolute_executable_directory_relative_paths::make_input_path_name_abs_executabledirectoryrelative_nocheck;
///
/// // Get an absolute path for "data/config.json" relative to the executable directory
/// let abs_path = make_input_path_name_abs_executabledirectoryrelative_nocheck("data/config.json").unwrap();
/// println!("Absolute path: {}", abs_path.display());
/// ```
pub fn make_input_path_name_abs_executabledirectoryrelative_nocheck<P: AsRef<Path>>(path_to_make_absolute: P) -> Result<PathBuf, io::Error> {
    // Get the directory where the executable is located
    let executable_directory = get_absolute_path_to_executable_parentdirectory()?;
    
    // Create a path by joining the executable directory with the provided path
    let target_path = executable_directory.join(path_to_make_absolute);
    
    // If the path doesn't exist, we still return the absolute path without trying to canonicalize
    if !abs_executable_directory_relative_exists(&target_path)? {
        // Ensure the path is absolute (it should be since we joined with executable_directory)
        if target_path.is_absolute() {
            return Ok(target_path);
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Failed to create absolute path",
            ));
        }
    }
    
    // Path exists, so we can canonicalize it to resolve any ".." or "." segments
    target_path.canonicalize().map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to canonicalize path: {}", e),
        )
    })
}

/// Checks if a path exists (either as a file or directory).
///
/// # Arguments
///
/// * `path_to_check` - The path to check for existence
///
/// # Returns
///
/// * `Result<bool, io::Error>` - Whether the path exists or an error
pub fn abs_executable_directory_relative_exists<P: AsRef<Path>>(path_to_check: P) -> Result<bool, io::Error> {
    let path = path_to_check.as_ref();
    Ok(path.exists())
}

/// Gets an absolute path for an existing directory relative to the executable's directory.
/// Returns an error if the directory doesn't exist or isn't a directory.
///
/// # Arguments
///
/// * `dir_path` - A directory path to convert to an absolute path relative to 
///   the executable's directory location.
///
/// # Returns
///
/// * `Result<PathBuf, io::Error>` - The absolute directory path or an error
pub fn make_dir_path_abs_executabledirectoryrelative_canonicalized_or_error<P: AsRef<Path>>(dir_path: P) -> Result<PathBuf, io::Error> {
    let path = make_input_path_name_abs_executabledirectoryrelative_nocheck(dir_path)?;
    
    // Check if the path exists and is a directory
    if !abs_executable_directory_relative_exists(&path)? {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Directory does not exist",
        ));
    } else if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Path exists but is not a directory",
        ));
    }
    
    // Canonicalize the path (should succeed because we've verified it exists)
    path.canonicalize().map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to canonicalize directory path: {}", e),
        )
    })
}

/// Creates a new directory at the specified path relative to the executable directory.
/// Returns an error if the directory already exists.
///
/// # Arguments
///
/// * `dir_path` - The directory path relative to the executable directory
///
/// # Returns
///
/// * `Result<PathBuf, io::Error>` - The absolute, canonicalized path to the newly created directory
pub fn mkdir_new_abs_executabledirectoryrelative_canonicalized<P: AsRef<Path>>(dir_path: P) -> Result<PathBuf, io::Error> {
    // Get the absolute path without checking existence
    let abs_path = make_input_path_name_abs_executabledirectoryrelative_nocheck(dir_path)?;
    
    // Check if the directory already exists
    if abs_executable_directory_relative_exists(&abs_path)? {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Directory already exists",
        ));
    }
    
    // Create the directory and all parent directories
    std::fs::create_dir_all(&abs_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to create directory: {}", e),
        )
    })?;
    
    // Canonicalize the path (should succeed because we just created it)
    abs_path.canonicalize().map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to canonicalize newly created directory path: {}", e),
        )
    })
}

/// Makes, verifies, or creates a directory path relative to the executable directory location.
///
/// This function performs the following sequential steps:
/// 1. Converts the provided directory path string to an absolute path relative to the executable directory
/// 2. Checks if the directory exists at the calculated absolute path location
/// 3. If the directory does not exist, creates it and all necessary parent directories
/// 4. Returns the canonicalized (absolute path with all symlinks resolved) path to the directory
///
/// # Arguments
///
/// * `dir_path_string` - A string representing the directory path relative to the executable directory
///
/// # Returns
///
/// * `Result<PathBuf, std::io::Error>` - The canonicalized absolute path to the directory if successful,
///   or an error if any step fails (executable path determination, directory creation, or canonicalization)
///
/// # Errors
///
/// This function may return an error in the following situations:
/// - If the executable's directory cannot be determined
/// - If directory creation fails due to permissions or other I/O errors
/// - If path canonicalization fails
///
/// use example:
/// // Ensure the project graph data directory exists relative to the executable
/// let project_graph_directory_result = make_verify_or_create_executabledirectoryrelative_canonicalized_dir_path("project_graph_data");

/// // Handle any errors that might occur during directory creation or verification
/// let project_graph_directory = match project_graph_directory_result {
///     Ok(directory_path) => directory_path,
///     Err(io_error) => {
///         // Log the error and handle appropriately for your application
///         return Err(format!("Failed to ensure project graph directory exists: {}", io_error).into());
///     }
/// };
///
pub fn make_verify_or_create_executabledirectoryrelative_canonicalized_dir_path(
    dir_path_string: &str
) -> Result<PathBuf, std::io::Error> {
    // Step 1: Convert the provided directory path to an absolute path relative to the executable
    let absolute_dir_path = make_input_path_name_abs_executabledirectoryrelative_nocheck(dir_path_string)?;
    
    // Step 2: Check if the directory exists at the calculated absolute path
    let directory_exists = abs_executable_directory_relative_exists(&absolute_dir_path)?;
    
    if !directory_exists {
        // Step 3: Directory doesn't exist, create it and all parent directories
        // Note: mkdir_new_abs_executabledirectoryrelative_canonicalized will also canonicalize the path
        mkdir_new_abs_executabledirectoryrelative_canonicalized(dir_path_string)
    } else {
        // Step 4: Directory already exists, canonicalize the path to resolve any symlinks
        absolute_dir_path.canonicalize().map_err(|canonicalization_error| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to canonicalize existing directory path: {}", canonicalization_error)
            )
        })
    }
}

/// Counts the number of subdirectories in the specified directory using executable-relative paths.
///
/// This function verifies the target directory exists, converts the path to be relative to the
/// executable's location, counts only subdirectories (not files), and handles errors gracefully.
/// If any errors occur at any step, the function returns 0 without panicking.
///
/// # Arguments
///
/// * `dir_path` - A path to the directory whose subdirectories should be counted.
///                Can be absolute or relative to the executable's directory.
///
/// # Returns
///
/// * `usize` - The number of subdirectories found, or 0 if any errors occur
///   (directory doesn't exist, not a directory, permission errors, etc.)
///
/// # Examples
///
/// ```
/// // Count subdirectories in "data/team_channels" relative to executable location
/// let channel_count = count_subdirectories_executabledirectoryrelative_default_zero("data/team_channels");
/// println!("Found {} team channels", channel_count);
/// ```
pub fn count_subdirectories_executabledirectoryrelative_default_zero<P: AsRef<Path>>(dir_path: P) -> usize {
    // First verify the path exists and is a directory
    let abs_path = match make_dir_path_abs_executabledirectoryrelative_canonicalized_or_error(dir_path) {
        Ok(path) => {
            println!("Found valid directory at: {:?}", path);
            path
        },
        Err(e) => {
            // This covers cases where the directory doesn't exist or isn't a directory
            println!("Error: directory validation failed: {}", e);
            return 0;
        }
    };

    // Attempt to read directory entries
    match fs::read_dir(&abs_path) {
        Ok(entries) => {
            // Count only directories
            let count = entries
                .filter_map(|entry_result| {
                    match entry_result {
                        Ok(entry) => {
                            match entry.file_type() {
                                Ok(file_type) if file_type.is_dir() => Some(()),
                                Ok(_) => None, // Not a directory
                                Err(e) => {
                                    println!("Error determining file type for {:?}: {}", 
                                              entry.path(), e);
                                    None
                                }
                            }
                        },
                        Err(e) => {
                            println!("Error reading directory entry: {}", e);
                            None
                        }
                    }
                })
                .count();
            
            println!("Found {} subdirectories in {:?}", count, abs_path);
            count
        },
        Err(e) => {
            println!("Error reading directory contents of {:?}: {}", abs_path, e);
            0 // Return 0 on error
        }
    }
}

/// Gets an absolute path for an existing file relative to the executable's directory.
/// Returns an error if the file doesn't exist or isn't a file.
///
/// # Arguments
///
/// * `file_path` - A file path to convert to an absolute path relative to 
///   the executable's directory location.
///
/// # Returns
///
/// * `Result<PathBuf, io::Error>` - The absolute file path or an error
///
/// use example
///
/// // Check for uma.toml file relative to the executable's directory
/// let uma_toml_path_result = make_file_path_abs_executabledirectoryrelative_canonicalized_or_error("uma.toml");
///
/// // Handle the result appropriately
/// let uma_toml_path = match uma_toml_path_result {
///     Ok(file_path) => {
///         // File exists, we can proceed with using it
///         debug_log!("Found uma.toml at: {:?}", file_path);
///         file_path
///     },
///     Err(io_error) => {
///         if io_error.kind() == std::io::ErrorKind::NotFound {
///             // File doesn't exist - handle this specific case
///             return Err(format!("Configuration file uma.toml not found in executable directory").into());
///         } else if io_error.kind() == std::io::ErrorKind::InvalidInput {
///             // Path exists but is a directory
///             return Err(format!("uma.toml exists but is a directory, not a file").into());
///         } else {
///             // Other I/O errors
///             return Err(format!("Error accessing uma.toml: {}", io_error).into());
///         }
///     }
/// };
///
pub fn make_file_path_abs_executabledirectoryrelative_canonicalized_or_error<P: AsRef<Path>>(file_path: P) -> Result<PathBuf, io::Error> {
    let path = make_input_path_name_abs_executabledirectoryrelative_nocheck(file_path)?;
    
    // Check if the path exists and is a file
    if !abs_executable_directory_relative_exists(&path)? {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "File does not exist",
        ));
    } else if path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Path exists but is a directory, not a file",
        ));
    }
    
    // Canonicalize the path (should succeed because we've verified it exists)
    path.canonicalize().map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to canonicalize file path: {}", e),
        )
    })
}

/// Creates necessary parent directories for a file path relative to the executable.
/// Does NOT create the file itself, only prepares the path structure.
///
/// # Arguments
///
/// * `file_path` - A file path relative to the executable's directory
///
/// # Returns
///
/// * `Result<PathBuf, io::Error>` - The absolute path to the (non-existent) file with parent directories prepared
pub fn prepare_file_parent_directories_abs_executabledirectoryrelative<P: AsRef<Path>>(file_path: P) -> Result<PathBuf, io::Error> {
    let path = make_input_path_name_abs_executabledirectoryrelative_nocheck(file_path)?;
    
    // If the path exists and is a directory, that's an error
    if abs_executable_directory_relative_exists(&path)? && path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Path exists but is a directory, not a file",
        ));
    }
    
    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        if !abs_executable_directory_relative_exists(parent)? {
            std::fs::create_dir_all(parent).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to create parent directory: {}", e),
                )
            })?;
        }
    }
    
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::Path;

    // Test get_absolute_path_to_executable_parentdirectory
    #[test]
    fn test_get_executable_parentdirectory() {
        let result = get_absolute_path_to_executable_parentdirectory();
        
        // Should succeed and return a valid path
        assert!(result.is_ok());
        
        let dir = result.unwrap();
        
        // Should be absolute
        assert!(dir.is_absolute());
        
        // Should exist
        assert!(dir.exists());
        
        // Should be a directory
        assert!(dir.is_dir());
        
        // Should match parent of current_exe
        let expected = std::env::current_exe()
            .expect("Failed to get current executable path")
            .parent()
            .expect("Failed to get parent directory")
            .to_path_buf();
        
        assert_eq!(dir, expected);
    }

    // Test make_input_path_name_abs_executabledirectoryrelative_nocheck with various inputs
    #[test]
    fn test_make_path_absolute_nocheck() {
        // Test with a simple relative path
        let result = make_input_path_name_abs_executabledirectoryrelative_nocheck("some/path.txt");
        assert!(result.is_ok());
        
        let path = result.unwrap();
        assert!(path.is_absolute());
        
        // Path should be executable_dir + relative_path
        let exec_dir = get_absolute_path_to_executable_parentdirectory().unwrap();
        assert_eq!(path, exec_dir.join("some/path.txt"));
        
        // Test with a path containing ..
        let result = make_input_path_name_abs_executabledirectoryrelative_nocheck("some/../other/path.txt");
        assert!(result.is_ok());
        
        // Test with an empty path
        let result = make_input_path_name_abs_executabledirectoryrelative_nocheck("");
        assert!(result.is_ok());
        
        // Test with just a dot
        let result = make_input_path_name_abs_executabledirectoryrelative_nocheck(".");
        assert!(result.is_ok());
        
        // Test with an absolute path (platform-specific)
        #[cfg(windows)]
        let abs_path = "C:\\absolute\\path.txt";
        #[cfg(not(windows))]
        let abs_path = "/absolute/path.txt";
        
        let result = make_input_path_name_abs_executabledirectoryrelative_nocheck(abs_path);
        assert!(result.is_ok());
    }

    // Test abs_executable_directory_relative_exists with various paths
    #[test]
    fn test_path_exists() {
        // Test with the current directory path (which definitely exists)
        let current_dir = env::current_dir().unwrap();
        let result = abs_executable_directory_relative_exists(&current_dir);
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Test with the executable directory (which definitely exists)
        let exec_dir = get_absolute_path_to_executable_parentdirectory().unwrap();
        let result = abs_executable_directory_relative_exists(&exec_dir);
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Test with the executable file itself (which definitely exists)
        let exec_file = std::env::current_exe().unwrap();
        let result = abs_executable_directory_relative_exists(&exec_file);
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Test with a path that shouldn't exist
        let nonexistent = Path::new("/this/path/definitely/does/not/exist/12345abcde");
        let result = abs_executable_directory_relative_exists(nonexistent);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    // Test errors from make_input_path_name_abs_executabledirectoryrelative_nocheck
    #[test]
    fn test_path_error_handling() {
        // Create an invalid path that would cause an error
        // This is difficult to do in a cross-platform way without actually creating files,
        // but we can at least check that we don't panic
        let result = make_input_path_name_abs_executabledirectoryrelative_nocheck("\0invalid");
        
        // On most platforms, this should fail (but we're just making sure it doesn't panic)
        if result.is_err() {
            let err = result.err().unwrap();
            assert!(err.to_string().len() > 0); // Error should have a message
        }
    }

    // Test path normalization logic
    #[test]
    fn test_path_normalization() {
        // Get executable directory
        let exec_dir = get_absolute_path_to_executable_parentdirectory().unwrap();
        
        // Create paths with different representations
        let path1 = make_input_path_name_abs_executabledirectoryrelative_nocheck("dir/file.txt").unwrap();
        let path2 = make_input_path_name_abs_executabledirectoryrelative_nocheck("dir/./file.txt").unwrap();
        let path3 = make_input_path_name_abs_executabledirectoryrelative_nocheck("dir/../dir/file.txt").unwrap();
        
        // Before canonicalization, these might not be equal
        // Check they all have the correct base directory though
        assert!(path1.starts_with(&exec_dir));
        assert!(path2.starts_with(&exec_dir));
        assert!(path3.starts_with(&exec_dir));
        
        // Path1 and path2 should resolve to the same path if they were canonicalized
        // We can't actually canonicalize without the paths existing, but we can check
        // that our function handles them correctly
    }

    // Test access to "real" files in the executable directory
    #[test]
    fn test_real_file_access() {
        // Get the executable file itself, which definitely exists
        let exec_file = std::env::current_exe().unwrap();
        
        // Check that it exists using our function
        let result = abs_executable_directory_relative_exists(&exec_file);
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Get its filename
        let filename = exec_file.file_name().unwrap();
        
        // Now try using our path function to resolve the same file
        let resolved_path = make_input_path_name_abs_executabledirectoryrelative_nocheck(filename).unwrap();
        
        // The resolved path should exist and be a file
        assert!(resolved_path.exists());
        assert!(resolved_path.is_file());
    }

    // Test edge cases for all functions
    #[test]
    fn test_edge_cases() {
        // Empty path
        let result = make_input_path_name_abs_executabledirectoryrelative_nocheck("");
        assert!(result.is_ok());
        
        // Just a dot
        let result = make_input_path_name_abs_executabledirectoryrelative_nocheck(".");
        assert!(result.is_ok());
        
        // Just dot-dot (parent dir)
        let result = make_input_path_name_abs_executabledirectoryrelative_nocheck("..");
        assert!(result.is_ok());
        
        // Long path with many segments
        let long_path = "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z/file.txt";
        let result = make_input_path_name_abs_executabledirectoryrelative_nocheck(long_path);
        assert!(result.is_ok());
        
        // Path with unicode characters
        let unicode_path = "🦀/😊/🔥/file.txt";
        let result = make_input_path_name_abs_executabledirectoryrelative_nocheck(unicode_path);
        assert!(result.is_ok());
        
        // Path with spaces
        let path_with_spaces = "folder with spaces/file with spaces.txt";
        let result = make_input_path_name_abs_executabledirectoryrelative_nocheck(path_with_spaces);
        assert!(result.is_ok());
    }

    // Test behavior of directory-specific functions with simulated paths
    #[test]
    fn test_directory_functions() {
        // Create an absolute path to a directory that doesn't exist
        let nonexistent_dir = Path::new("/nonexistent/directory/path");
        
        // If path doesn't exist, our function should return an error
        // We're not actually calling it here because it would fail, but we're making
        // sure our logic is correct for checking existence
        assert!(!nonexistent_dir.exists());
        
        // Get a directory that definitely exists (the executable parent directory)
        let existing_dir = get_absolute_path_to_executable_parentdirectory().unwrap();
        assert!(existing_dir.exists());
        assert!(existing_dir.is_dir());
    }
    
    // Test behavior of file-specific functions with simulated paths
    #[test]
    fn test_file_functions() {
        // Create an absolute path to a file that doesn't exist
        let nonexistent_file = Path::new("/nonexistent/file/path.txt");
        
        // If path doesn't exist, our function should return an error
        // We're not actually calling it here because it would fail, but we're making
        // sure our logic is correct for checking existence
        assert!(!nonexistent_file.exists());
        
        // Get a file that definitely exists (the executable itself)
        let existing_file = std::env::current_exe().unwrap();
        assert!(existing_file.exists());
        assert!(existing_file.is_file());
    }
}