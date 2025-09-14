### rows_and_columns

# ...under construction...


rows and columns, csv db, with TUI-Dash rust 2025 v1

A minimal modular rust project
compatible with or similar to:
- Uma
- FF
- Lines
- rust toml files

User-Features:
- loads a .csv
- shows analysis:
-- e.g. items from python-pands series.describe()
- TUI-Dashboard, e.g.
-- bar-chart/histogram
-- scatter-plot
-- box and whiskers plot

### TUI dashboard system is suitable for:
-- terminal and headless posix systems
-- display as WEB-based TUI, such as a GET-TUI system that displays simple characters (e.g. not even as html) with the interface being commands entered as get-requests in the URL
- if possible both an ASCII mode and (maybe default) Unicode mode for the dashboard-TUI. 
- for a simple histogram, a single | pipe is fine.
- a box and whiskers plot could be three characters across, made of pipes and dashes.
e.g.
```ascii
-|-   (max)
 |
| |   (third quartile
| |
---   (median)
| |
| |
| |   (first quartile)
 |
 |
-|-   (min)
```

Guidelines:
- get (what is) needed when (it is) needed: not pre-loading data that is not needed
- do one thing well: not a swiss-army knife
- simple minimal TUI (see UMA, FF, Lines as examples/illustrations)
- vanilla Rust: no third party crates
- modular: called as module like FF/Lines
- uses binary-executible-relative file paths (see existing module)
- may use clearsigned data-file verification (see existing module)
- clarity is a goal, brevity is NOT a goal
- clear communication is a goal
- no unwrap
- always error handling
- always very very extensive doc strings
- always very clear comments in code
- always extremely clear variable names and never colliding names: search for any variable name must only return that variable.
- plan first: measure 10-times, cut once
- move step by step and test each step before moving on




Workflow: walk through
- data are in a .csv
- make a .csv_metadata_toml file that specifies at least the rust datatypes for the columns (other metadata might be needed)
- the .csv data will be organized by column
- all columns will be (preserving the original file) reformatted in a directory-file structure, where each columns is a directory, and each 'cell' is a directory, and each cell/value is (at least) one .txt file containing that value (the 'value' could also be a more elaborate data structure in a set of directories and files with more meta-data, but for now starting with simple columns: boolean, int, float, short-string, 
- while you can input and output a .csv, the internal format of the data (not a temporary form) is directories and files. Ideally this should also be 'human-read-able' so that someone could find the column and row and look at the value.


# No-Load Data Processing:
- iterate over data in a no-load fashion, calculating on-the-fly, not pre-loading anything. Yes, is different from standard short-sighted software that does not scale: this software must scale.

## feature 1: analysis
- such as 'descriptive statistics
- MVP start with pandas descriptive statistics, there are more that can be added (K. Nakamura's book on statistics with Rust is full of great examples!)


## feature 2: Dashboards via TUI
- generate TUI such as histogram comparing two fields

### notes:
- probably using the ff module as a way for users to select files etc.
- ideally, ff would be one module in the set of modules for rows_and_columns (if for some reason someone needs to browse or see a directory), or vice-versa, when rows_and_columns is working, ff can recommend rows_and_columns for 'opening' looking at a .csv file. FF should not be blended/mixed into the rows_and_columns module.


maybe future features: (or maybe not)
- pre-compiled lookup tables: the value-to-row lookup for each column can be made into a mini-compiled lookup dict, a kind of mini Rust (or or zig) binary for cases where fast-lookup or search are desired. e.g. make a simple program that is a hash-table to look up the row for each value. in this way all columns can have a lookup-'index' not just a primary-index column. 
- generate svg/bitmap of data visualization

