use std::env;

use python_script_analyser::{File, get_file_lines, vec_str_to_vec_line};

fn main() {
    // Get command line arguments.
    let args: Vec<String> = env::args().collect();
    
    // Make sure there is at least one commandline argument.
    if args.len() <= 1 {
        println!("Usage: python-script-analyser.exe <filename>");
        println!("Note: This program does not check for errors, use the python interpreter for that.");
        return;
    }
    
    // Assume that the first argument is the filename of the python script.
    let filename: &str = &args[1];
    
    // Read file contents.
    let lines = match get_file_lines(filename) {
        Ok(lines) => lines, 
        Err(error) => {
            eprintln!("An error occured while trying to read the file {}: {:?}", filename, error);
            return;
        }
    };
    
    // TODO: Do one pass over lines to check for indentation inconsistencies.
    
    // Handle file.
    let file: File = File::new(filename, &vec_str_to_vec_line(&lines));
    
    // Print file data.
    println!("{}", file.as_string(0));
}
