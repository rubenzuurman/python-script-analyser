use std::env;
use std::io::{BufWriter, Write};

use python_script_analyser::{File, get_file_lines, vec_str_to_vec_line, write_to_writer, flush_writer};

fn main() {
    // Initialize writer.
    let stdout_handle = std::io::stdout();
    let mut writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
    
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
    let file: File = File::new(filename, &vec_str_to_vec_line(&lines), &mut writer);
    
    // Print file data.
    let fas: String = file.as_string(0);
    let fas_bytes: &[u8] = fas.as_bytes();
    write_to_writer(&mut writer, fas_bytes);
    
    file.scan(&mut writer);
    
    let buffer_vec: Vec<u8> = writer.buffer().to_vec();
    let buffer: String = String::from_utf8(buffer_vec).unwrap();
    
    // Check occurences of "WARNING".
    let number_of_warnings: usize = buffer.matches("WARNING").count();
    write_to_writer(&mut writer, format!("Number of warnings: {}\n", number_of_warnings).as_bytes());
    
    // Flush writer.
    flush_writer(&mut writer);
}
