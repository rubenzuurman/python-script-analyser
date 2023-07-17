use std::env;

use python_script_analyser::{File, get_file_lines, vec_str_to_vec_line};

static STRING: &str = r"{indentation}class Rect:";

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
    
    let file: File = File::create(filename, &vec_str_to_vec_line(&lines));
    
    // Print file data.
    println!("Imports: {:?}", file.get_imports());
    println!("Global variables: {:?}", file.get_global_variables());
    for function in file.get_functions() {
        println!("\nFunction source '{}':", function.get_name());
        for line in function.get_source() {
            println!("{}", line);
        }
    }
    for class in file.get_classes() {
        println!("\nClass source '{}':", class.get_name());
        for line in class.get_source() {
            println!("{}", line);
        }
        println!("Class methods:");
        for method in class.get_methods() {
            for line in method.get_source() {
                println!("{}", line);
            }
        }
    }
}
