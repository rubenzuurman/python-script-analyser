use std::env;
use std::fs;
use std::io;

use regex::Regex;

use python_script_analyser::Function;

/*

Struct containing a function after "(no indentation)def *(*):".
Struct containing a class after "(no indentation)class *(*):".
    Struct containing class method in class.

*/

fn get_file_lines(filename: &str) -> Result<Vec<String>, io::Error> {
    let mut result: Vec<String> = Vec::new();
    let contents = fs::read_to_string(filename)?;
    for line in contents.lines() {
        result.push(line.to_string());
    }
    return Ok(result);
}

fn main() {
    let func: Function = Function::create(vec![String::from("def func_name(param1, param2):"), String::from("Appel")]);
    println!("{:?}", func);
    
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
    
    // Set up regex strings.
    let re_import = Regex::new(r"^import (?<import>[\w+ ,]+) *$").unwrap();
    let re_from_import = Regex::new(r"^from (?<module>\w+) import (?<objects>[\w ,]+) *$").unwrap();
    let re_def = Regex::new(r"^def (?<def>\w+)\((?<params>[\w ,]*)\): *$").unwrap();
    let re_class = Regex::new(r"^class (?<class>\w+): *$").unwrap();
    
    // Set up regex strings for further investigation.
    let re_import_check_as = Regex::new(r"^\w+ as \w+$").unwrap();
    let re_import_replace_space = Regex::new(r" +").unwrap();
    
    // Set up vector to hold global imports.
    let mut imported_modules: Vec<String> = Vec::new();
    let mut imported_objects: Vec<String> = Vec::new();
    
    // Loop over all lines and create objects or add to import vectors.
    for (index, line) in lines.iter().enumerate() {
        let import_captures = re_import.captures(&line);
        let from_import_captures = re_from_import.captures(&line);
        let def_captures = re_def.captures(&line);
        let class_captures = re_class.captures(&line);
        match import_captures {
            Some(a) => {
                println!("Line {}: Matching import definition: '{}' '{}'", index, line, &a["import"]);
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
                                eprintln!("Line {}: Import cannot contain spaces '{}' (specifically '{}').", index, line, import_trim);
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
                    println!("Line {}: Matching from import definition: '{}'", index, line);
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
                                    eprintln!("Line {}: Import cannot contain spaces '{}' (specifically '{}').", index, line, object_trim);
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
                        println!("Line {}: Matching function definition: '{}'", index, line);
                    }, 
                    None => match class_captures {
                        Some(_d) => {
                            println!("Line {}: Matching class definition: '{}'", index, line);
                        }, 
                        None => {
                            continue;
                        }
                    }
                }
            }
        }
    }
    println!("Imported modules: '{:?}'", imported_modules);
    println!("Imported objects: '{:?}'", imported_objects);
    
    // Maybe create struct to handle the file. Create new substruct for function definitions for example.
}
