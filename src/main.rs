use std::env;

use regex::Regex;

use python_script_analyser::{Function, Line, Class, get_file_lines};

/*

Struct containing a function after "(no indentation)def *(*):".
Struct containing a class after "(no indentation)class *(*):".
    Struct containing class method in class.

*/

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
    
    // Set up regex strings.
    let re_import = Regex::new(r"^import (?<import>[\w+ ,]+) *$").unwrap();
    let re_from_import = Regex::new(r"^from (?<module>\w+) import (?<objects>[\w ,]+) *$").unwrap();
    let re_def = Regex::new(r"^def (?<name>\w+)\((?<params>.*)\): *$").unwrap();
    let re_class = Regex::new(r"^class (?<class>\w+)(\(\w+\))?: *$").unwrap();
    // TODO: Implement detection of global variables.
    // TODO: Implement detection of program root.
    
    // Set up regex strings for further investigation.
    let re_import_check_as = Regex::new(r"^\w+ as \w+$").unwrap();
    let re_import_replace_space = Regex::new(r" +").unwrap();
    let re_get_indentation = Regex::new(r"^(?<indentation> *).*$").unwrap();
    
    // Set up vector to hold global imports.
    let mut imported_modules: Vec<String> = Vec::new();
    let mut imported_objects: Vec<String> = Vec::new();
    let mut functions: Vec<Function> = Vec::new();
    let mut classes: Vec<Class> = Vec::new();
    
    // Initialize variables to keep track of functions and classes.
    let mut in_function: bool = false;
    let mut in_class: bool = false;
    
    let mut current_function_indentation_length: usize = 0;
    let mut current_function_indentation_set: bool = false;
    let mut current_function_source: Vec<Line> = Vec::new();
    
    let mut current_class_indentation_length: usize = 0;
    let mut current_class_indentation_set: bool = false;
    let mut current_class_source: Vec<Line> = Vec::new();
    
    // Loop over all lines and create objects or add to import vectors.
    for (index, line) in lines.iter().enumerate() {
        // Skip empty lines.
        if line.trim() == "" {
            continue;
        }
        
        // This regex execution will match any non-empty line.
        let get_indentation_captures = re_get_indentation.captures(&line).unwrap();
        let indentation_length: usize = String::from(&get_indentation_captures["indentation"]).len();
        
        if in_function && in_class {
            println!("This is kinda bad, in function and in class.");
        }
        
        if in_function {
            if !current_function_indentation_set {
                // Set indentation length for function.
                current_function_indentation_length = indentation_length;
                current_function_indentation_set = true;
                
                current_function_source.push(Line::create(index + 1, line));
            } else {
                // Check if the indentation is less than the current function indentation.
                if indentation_length >= current_function_indentation_length {
                    // Line in function.
                    current_function_source.push(Line::create(index + 1, line));
                } else {
                    // End of function.
                    // Create function object and add to functions vector.
                    let function: Function = Function::create(&current_function_source);
                    println!("Adding function with name '{}' to functions.", function.get_name());
                    functions.push(function);
                    
                    // Reset function tracking variables.
                    in_function = false;
                    current_function_indentation_length = 0;
                    current_function_indentation_set = false;
                    current_function_source = Vec::new();
                }
            }
        }
        if in_class {
            // Check if the indentation is less than the current function indentation.
            if !current_class_indentation_set {
                // Set indentation length for class.
                current_class_indentation_length = indentation_length;
                current_class_indentation_set = true;
                
                current_class_source.push(Line::create(index + 1, line));
            } else {
                if indentation_length >= current_class_indentation_length {
                    // Line in class.
                    current_class_source.push(Line::create(index + 1, line));
                } else {
                    // End of class.
                    // Create class object and add to classes vector.
                    let class: Class = Class::create(&current_class_source);
                    println!("Adding class with name '{}' to classes.", class.get_name());
                    classes.push(class);
                    
                    // Reset class tracking variables.
                    in_class = false;
                    current_class_indentation_length = 0;
                    current_class_indentation_set = false;
                    current_class_source = Vec::new();
                }
            }
        }
        let import_captures = re_import.captures(&line);
        let from_import_captures = re_from_import.captures(&line);
        let def_captures = re_def.captures(&line);
        let class_captures = re_class.captures(&line);
        match import_captures {
            Some(a) => {
                println!("Line {}: Matching import definition: '{}' '{}'", index + 1, line, &a["import"]);
                for import in a["import"].split(",") {
                    let import_trim: &str = &re_import_replace_space.replace_all(&import.trim(), " ");
                    //println!("Import trim: '{}'", import_trim);
                    let check_as_captures = re_import_check_as.captures(&import_trim);
                    match check_as_captures {
                        Some(_b) => {
                            // Import module as some other name.
                            let import_split: Vec<&str> = import_trim.split(" as ").collect();
                            imported_modules.push(String::from(import_split[1]));
                        }, 
                        None => {
                            if import_trim.contains(" ") {
                                // Import module, but does contain a space (e.g. 'import g    h').
                                eprintln!("Line {}: Import cannot contain spaces '{}' (specifically '{}').", index + 1, line, import_trim);
                            } else {
                                // Import module.
                                imported_modules.push(String::from(import_trim));
                            }
                        }
                    }
                }
            }, 
            None => match from_import_captures {
                Some(b) => {
                    println!("Line {}: Matching from import definition: '{}'", index + 1, line);
                    for object in b["objects"].split(",") {
                        let object_trim: &str = &re_import_replace_space.replace_all(&object.trim(), " ");
                        //println!("Import from trim: '{}'", object_trim);
                        let check_as_captures = re_import_check_as.captures(&object_trim);
                        match check_as_captures {
                            Some(_b) => {
                                // Import object as some other name.
                                let object_split: Vec<&str> = object_trim.split(" as ").collect();
                                imported_objects.push(String::from(object_split[1]));
                            }, 
                            None => {
                                if object_trim.contains(" ") {
                                    // Import object, but does contain a space (e.g. 'from a import b as c').
                                    eprintln!("Line {}: Import cannot contain spaces '{}' (specifically '{}').", index + 1, line, object_trim);
                                } else {
                                    // Import object.
                                    imported_objects.push(String::from(object_trim));
                                }
                            }
                        }
                    }
                }, 
                None => match def_captures {
                    Some(_c) => {
                        println!("Line {}: Matching function definition: '{}'", index + 1, line);
                        
                        in_function = true;
                        
                        current_function_source.push(Line::create(index + 1, line));
                        
                    }, 
                    None => match class_captures {
                        Some(_d) => {
                            println!("Line {}: Matching class definition: '{}'", index + 1, line);
                            
                            in_class = true;
                            
                            current_class_source.push(Line::create(index + 1, line));
                        }, 
                        None => {
                            continue;
                        }
                    }
                }
            }
        }
    }
    
    // Check if a function or class was getting collected but the source ended.
    if in_function {
        // Create function object and add to classmethods vector.
        let function: Function = Function::create(&current_function_source);
        println!("Adding function with name '{}' to functions.", function.get_name());
        functions.push(function);
    }
    if in_class {
        // Create class object and add to classes vector.
        let class: Class = Class::create(&current_class_source);
        println!("Adding class with name '{}' to classes.", class.get_name());
        classes.push(class);
    }
    
    println!("\nImported modules: '{:?}'", imported_modules);
    println!("Imported objects: '{:?}'", imported_objects);
    for function in functions {
        println!("\nFunction source '{}':", function.get_name());
        for line in function.get_source() {
            println!("{}", line);
        }
    }
    for class in classes {
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
    
    // Maybe create struct to handle the file. Create new substruct for function definitions for example.
}
