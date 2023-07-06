use std::env;

fn main() {
    // Get command line arguments.
    let args: Vec<String> = env::args().collect();
    
    // Make sure there is at least one commandline argument.
    if args.len() <= 1 {
        println!("Usage: python-script-analyser.exe <filename>");
        return;
    }
    
    // Assume that the first argument is the filename of the python script.
    let filename: &str = &args[1];
    
    println!("{}", filename);
}
