use std::fs;
use std::io;
use std::path::Path;
use std::ffi::OsStr;

use regex::Regex;

static PATTERN_INDENTATION: &str = r"^(?P<indentation>[\t ]*).*$";
static PATTERN_IMPORT: &str = r"^[\t ]*import[\t ]+(?P<modules>[\w, \t\.]+)$";
static PATTERN_FROM_IMPORT: &str = r"^[\t ]*from[\t ]+(?P<module>[\w\.]+)[\t ]+import[\t ]+(?P<objects>[\w ,\t]+)$";
static PATTERN_GLOBAL_VARIABLE: &str = r"^[\t ]*(?P<varname>\w+)[\t ]*(:.*)?[\t ]*=[\t ]*.*$";
static PATTERN_FUNCTION_START: &str = r"^(?P<indentation>[\t ]*)def[\t ]+(?P<name>\w+)[\t ]*\((?P<params>.*)\)[\t ]*(->[\t ]*[\w, \t\[\]]+[\t ]*)?:[\t ]*$";
static PATTERN_CLASS_START: &str = r"^(?P<indentation>[\t ]*)class[\t ]+(?P<name>\w+)[\t ]*(\((?P<parent>[\w \t\[\]\.,]*)\))?[\t ]*:[\t ]*$";
static PATTERN_CLASS_VARIABLE: &str = r"^[\t ]{INDENTATION}(?P<varname>\w+)[\t ]*(:.*)?[\t ]*=[\t ]*(?P<varvalue>.+)[\t ]*$"; // INDENTATION will be replaced with the current class indentation when this pattern is used.

#[derive(Clone, Debug)]
pub struct Line {
    number: usize, 
    text: String, 
}

impl Line {
    
    pub fn new(number: usize, text: &str) -> Self {
        return Line {
            number: number, 
            text: text.to_string()
        };
    }
    
    pub fn get_number(&self) -> usize {
        return self.number;
    }
    
    pub fn get_text(&self) -> &String {
        return &self.text;
    }
    
    pub fn is_assignment(&self) -> Option<usize> {
        /*
        A line is an assignment if it contains exactly one equal sign (not preceded by a less than sign, greater than sign, exclamation mark, plus sign, or minus sign) which is not enclosed by any of the following:
            Single quotations
            Double quotations
            Normal brackets
            Square brackets
            Curly brackets
        */
        let mut in_quotations: bool = false;
        let mut in_double_quotations: bool = false;
        let mut in_brackets_depth: u32 = 0;
        let mut in_square_brackets_depth: u32 = 0;
        let mut in_curly_brackets_depth: u32 = 0;
        
        let mut first_half: bool = true;
        let mut equals_index: usize = 0;
        for (index, c) in self.get_text().chars().enumerate() {
            match c {
                '\'' => in_quotations = !in_quotations,
                '\"' => in_double_quotations = !in_double_quotations,
                '(' => in_brackets_depth += 1,
                ')' => in_brackets_depth -= 1,
                '[' => in_square_brackets_depth += 1,
                ']' => in_square_brackets_depth -= 1,
                '{' => in_curly_brackets_depth += 1,
                '}' => in_curly_brackets_depth -= 1,
                '=' => {
                    // Check if this is the first character, in which case this is not an assignment.
                    if index == 0 {
                        return None;
                    }
                    
                    // Check if the previous character was not '>', '<', '!', '+', or '-'.
                    let prev_char: char = self.get_text().chars().nth(index - 1).unwrap();
                    if prev_char == '>' || prev_char == '<' || prev_char == '!' || prev_char == '+' || prev_char == '-' {
                        continue;
                    }
                    
                    // Check if not in quotations or brackets.
                    if !(in_quotations || in_double_quotations || in_brackets_depth > 0 || in_square_brackets_depth > 0 || in_curly_brackets_depth > 0) {
                        if first_half {
                            // First equals sign, could be an assignment.
                            first_half = false;
                            equals_index = index;
                        } else {
                            // Second equals sign, is definitely not an assignment.
                            return None;
                        }
                    }
                }, 
                _ => ()
            }
        }
        match first_half {
            true =>  return None, 
            false => return Some(equals_index), 
        }
    }
    
}

impl std::fmt::Display for Line {
    
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let line_space: &str = match self.number {
            n if n >= 1000 => " ", 
            n if n >= 100  => "  ", 
            n if n >= 10   => "   ", 
            n if n >= 1    => "    ", 
            _              => "    "
        };
        
        print!("Line{}{}: {}", line_space, self.number, self.text);
        return Ok(());
    }
    
}

impl PartialEq for Line {
    
    fn eq(&self, other: &Self) -> bool {
        return self.number == other.number && self.text == other.text;
    }
    
}

#[derive(Debug)]
pub struct Assignment {
    
    name: String, 
    value: String, 
    source: Line
    
}

impl Assignment {
    
    pub fn new(line: &Line) -> Option<Self> {
        // This function checks if the line contains an assignment. If it does, it results Some(Assignment), else it returns None. This Option<T> can then be matched by the caller.
        match line.is_assignment() {
            // Return none if the line does not contain an assignment.
            None => return None, 
            // Return some if the line does contain an assignment.
            Some(equals_index) => {
                // Split line text at index.
                let var: &str = &line.get_text().as_str()[..equals_index];
                let val: &str = &line.get_text().as_str()[equals_index+1..];
                
                // Check if the variable name contains a type hint.
                if var.contains(":") {
                    // Get index of the first ':'.
                    let mut index: usize = 0;
                    let colon_index = loop {
                        if var.chars().nth(index).unwrap() == ':' {
                            break index;
                        }
                        index += 1;
                    };
                    
                    // Extract variable name from variable name with type hint.
                    let name_type: (&str, &str) = var.split_at(colon_index);
                    let name: &str = name_type.0;
                    
                    return Some(Assignment {
                        name: name.trim().to_string(), 
                        value: val.trim().to_string(), 
                        source: line.clone()
                    });
                } else {
                    return Some(Assignment {
                        name: var.trim().to_string(), 
                        value: val.trim().to_string(), 
                        source: line.clone()
                    });
                }
            }
        }
    }
    
    pub fn get_name(&self) -> &String {
        return &self.name;
    }
    
    pub fn get_value(&self) -> &String {
        return &self.value;
    }
    
    pub fn get_source(&self) -> &Line {
        return &self.source;
    }
    
}

impl PartialEq for Assignment {
    
    fn eq(&self, other: &Self) -> bool {
        return self.get_name() == other.get_name() 
            && self.get_value() == other.get_value() 
            && self.get_source() == other.get_source();
    }
    
}

pub struct StructureTracker {
    active: bool, 
    indentation_length: usize, 
    indentation_set: bool, 
    source: Vec<Line>, 
}

impl StructureTracker {
    
    pub fn new() -> Self {
        return StructureTracker {
            active: false, 
            indentation_length: 0, 
            indentation_set: false, 
            source: Vec::new()
        };
    }
    
    pub fn start(&mut self) {
        self.active = true;
    }
    
    pub fn reset(&mut self) {
        self.active = false;
        self.indentation_length = 0;
        self.indentation_set = false;
        self.source = Vec::new();
    }
    
    pub fn is_active(&self) -> bool {
        return self.active;
    }
    
    pub fn get_indentation_length(&self) -> usize {
        return self.indentation_length;
    }
    
    pub fn set_indentation_length(&mut self, indentation_length: usize) {
        self.indentation_length = indentation_length;
        self.indentation_set = true;
    }
    
    pub fn indentation_set(&self) -> bool {
        return self.indentation_set;
    }
    
    pub fn get_source(&self) -> &Vec<Line> {
        return &self.source;
    }
    
    pub fn add_line(&mut self, line: &Line) {
        self.source.push(line.clone());
    }
    
}

#[derive(Debug, PartialEq)]
pub struct File {
    name: String, 
    imports: Vec<String>, 
    global_variables: Vec<String>, 
    functions: Vec<Function>, 
    classes: Vec<Class>, 
}

impl File {
    
    pub fn new(filename: &str, source: &Vec<Line>) -> Self {
        // Get filename from path.
        let path = Path::new(filename);
        let name: &OsStr = path.file_stem().unwrap();
        
        // Print warning if the extension is not 'py'.
        let extension: &OsStr = path.extension().unwrap();
        if extension != OsStr::new("py") {
            eprintln!("WARNING: The input file might not be a python file (extension='{}', not 'py').", extension.to_str().unwrap());
        }
        
        // Initialize structure tracker (used for tracking functions and classes).
        let mut function_tracker: StructureTracker = StructureTracker::new();
        let mut class_tracker: StructureTracker = StructureTracker::new();
        
        // Iterate over lines and detect things.
        let mut imports: Vec<String> = Vec::new();
        let mut global_vars: Vec<String> = Vec::new();
        let mut functions: Vec<Function> = Vec::new();
        let mut classes: Vec<Class> = Vec::new();
        for line in source.iter() {
            // Skip empty lines.
            if line.get_text().trim().is_empty() {
                continue;
            }
            
            // Check if currently in a function or a class.
            let indentation_length = File::get_indentation_length(line);
            if function_tracker.is_active() {
                if !function_tracker.indentation_set() {
                    // Indentation length not set, set indentation length and add line.
                    function_tracker.set_indentation_length(indentation_length);
                    function_tracker.add_line(&line);
                } else {
                    // Indentation length set.
                    if indentation_length >= function_tracker.get_indentation_length() {
                        // Not end of function, add line.
                        function_tracker.add_line(&line);
                    } else {
                        // End of function, create and push function.
                        let function: Function = Function::new(function_tracker.get_source());
                        functions.push(function);
                        // Reset tracker.
                        function_tracker.reset();
                    }
                }
            }
            if class_tracker.is_active() {
                if !class_tracker.indentation_set() {
                    // Indentation length not set, set indentation and add line.
                    class_tracker.set_indentation_length(indentation_length);
                    class_tracker.add_line(&line);
                } else {
                    // Indentation length set.
                    if indentation_length >= class_tracker.get_indentation_length() {
                        // Not end of class, add line.
                        class_tracker.add_line(&line);
                    } else {
                        // End of class, create and push class.
                        let class: Class = Class::new(class_tracker.get_source());
                        classes.push(class);
                        // Reset tracker.
                        class_tracker.reset();
                    }
                }
            }
            
            if function_tracker.is_active() || class_tracker.is_active() {
                continue;
            }
            
            // Detect imports.
            match File::line_is_import(&line) {
                Some(a) => {
                    for module in a.iter() {
                        imports.push(module.clone());
                    }
                }, 
                None => ()
            }
            
            // Detect global variables.
            match File::line_is_global_var(&line) {
                Some(a) => {
                    global_vars.push(a);
                }, 
                None => ()
            }
            
            // Detect functions.
            match File::line_is_function_start(&line) {
                Some(_) => {
                    // Set in function variables.
                    function_tracker.start();
                    function_tracker.add_line(&line);
                }, 
                None => ()
            }
            
            // Detect classes.
            match File::line_is_class_start(&line) {
                Some(_) => {
                    // Set in class variables.
                    class_tracker.start();
                    class_tracker.add_line(&line);
                }, 
                None => ()
            }
        }
        
        // Check if the function tracker or class tracker is still active.
        if function_tracker.is_active() {
            // End of function, create and push function.
            let function: Function = Function::new(function_tracker.get_source());
            functions.push(function);
        }
        if class_tracker.is_active() {
            // End of class, create and push function.
            let class: Class = Class::new(class_tracker.get_source());
            classes.push(class);
        }
        
        // Return file.
        return File {
            name: name.to_str().unwrap().to_string(), 
            imports: imports, 
            global_variables: global_vars, 
            functions: functions, 
            classes: classes
        };
    }
    
    fn get_indentation_length(line: &Line) -> usize {
        // Initialize regex and capture.
        let re_indentation = Regex::new(PATTERN_INDENTATION).unwrap();
        let indentation_capt = re_indentation.captures(line.get_text());
        
        // Return indentation length.
        return indentation_capt.unwrap()["indentation"].len();
    }
    
    fn line_is_import(line: &Line) -> Option<Vec<String>> {
        // Initialize regex.
        let re_import = Regex::new(PATTERN_IMPORT).unwrap();
        let re_from_import = Regex::new(PATTERN_FROM_IMPORT).unwrap();
        
        // Check if the line matches any of the regexes.
        let line_text: &String = line.get_text();
        let import_capt = re_import.captures(line_text);
        let from_import_capt = re_from_import.captures(line_text);
        
        match import_capt {
            Some(c) => {
                // Collect imports in a vector.
                let mut modules_vec: Vec<String> = Vec::new();
                let modules: String = String::from(&c["modules"]);
                for module in modules.split(",") {
                    // Split by " as ", if the module does not contain it, the split vector will have length 1, else it will have length 2. Regardless we want the last item in the vector.
                    let module_split: Vec<&str> = module.split(" as ").collect();
                    modules_vec.push(module_split.get(module_split.len() - 1).unwrap().trim().to_string());
                }
                
                // Remove imports containing spaces, print warning in case they do.
                let mut indices_to_remove: Vec<usize> = Vec::new();
                for (index, module) in modules_vec.iter().enumerate() {
                    if module.contains(char::is_whitespace) {
                        eprintln!("WARNING: Line {}: Import cannot contain spaces '{}' (specifically '{}').", line.get_number(), line.get_text(), module);
                        indices_to_remove.push(index);
                    }
                }
                for index in indices_to_remove.iter().rev() {
                    modules_vec.remove(*index);
                }
                
                return Some(modules_vec);
            }, 
            None => {
                match from_import_capt {
                    Some(c) => {
                        // Collect imports in a vector.
                        let mut objects_vec: Vec<String> = Vec::new();
                        let objects: String = String::from(&c["objects"]);
                        for object in objects.split(",") {
                            // Split by " as " (same as with modules).
                            let object_split: Vec<&str> = object.split(" as ").collect();
                            objects_vec.push(object_split.get(object_split.len() - 1).unwrap().trim().to_string());
                        }
                        
                        // Remove imports containing spaces, print warning in case they do.
                        let mut indices_to_remove: Vec<usize> = Vec::new();
                        for (index, object) in objects_vec.iter().enumerate() {
                            if object.contains(char::is_whitespace) {
                                eprintln!("WARNING: Line {}: Import cannot contain spaces '{}' (specifically '{}').", line.get_number(), line.get_text(), object);
                                indices_to_remove.push(index);
                            }
                        }
                        for index in indices_to_remove.iter().rev() {
                            objects_vec.remove(*index);
                        }
                        
                        return Some(objects_vec);
                    }, 
                    None => return None
                }
            }
        }
    }
    
    fn line_is_global_var(line: &Line) -> Option<String> {
        // Initialize and match regex.
        let re_global_var = Regex::new(PATTERN_GLOBAL_VARIABLE).unwrap();
        let global_var_capt = re_global_var.captures(line.get_text());
        
        match global_var_capt {
            Some(c) => return Some(c["varname"].to_string()), 
            None => return None
        }
    }
    
    fn line_is_function_start(line: &Line) -> Option<bool> {
        // Initialize and match regex.
        let re_function_definition = Regex::new(PATTERN_FUNCTION_START).unwrap();
        let function_definition_capt = re_function_definition.captures(line.get_text());
        
        match function_definition_capt {
            Some(_) => return Some(true), 
            None => return None
        }
    }
    
    fn line_is_class_start(line: &Line) -> Option<bool> {
        // Initialize and match regex.
        let re_class_definition = Regex::new(PATTERN_CLASS_START).unwrap();
        let class_definition_capt = re_class_definition.captures(line.get_text());
        
        match class_definition_capt {
            Some(_) => return Some(true), 
            None => return None
        }
    }
    
    pub fn get_name(&self) -> &String {
        return &self.name;
    }
    
    pub fn get_imports(&self) -> &Vec<String> {
        return &self.imports;
    }
    
    pub fn get_global_variables(&self) -> &Vec<String> {
        return &self.global_variables;
    }
    
    pub fn get_functions(&self) -> &Vec<Function> {
        return &self.functions;
    }
    
    pub fn get_classes(&self) -> &Vec<Class> {
        return &self.classes;
    }
    
}

#[derive(Debug, PartialEq)]
pub struct Function {
    name: String, 
    parameters: Vec<String>, 
    source: Vec<Line>, 
}

impl Function {
    
    pub fn new(source: &Vec<Line>) -> Self {
        // Get first line of the source.
        let first_line: &str = source.get(0).unwrap().get_text();
        
        // Initialize regex for getting the function name and the parameters from the function definition.
        let re_function_start = Regex::new(PATTERN_FUNCTION_START).unwrap();
        let re_replace_space = Regex::new(" +").unwrap();
        
        // Match regex and initialize function properties.
        let function_start_capt = re_function_start.captures(first_line);
        let mut name: String = "".to_string();
        let mut parameters: Vec<String> = Vec::new();
        
        // Match regex captures and set function properties.
        match function_start_capt {
            Some(a) => {
                name = String::from(&a["name"]);
                for param in a["params"].split(",") {
                    //println!("Param: '{}', nospace: '{}'", param, String::from(re_replace_space.replace_all(param.trim(), " ")));
                    // TODO: This is where you need to fix a bug where '  p5=3  ' is changed to 'p5=3', but '   p5   =   3   ' is changed to 'p5 = 3'.
                    let param_trim_replace = String::from(re_replace_space.replace_all(param.trim(), " "));
                    if !param_trim_replace.is_empty() {
                        parameters.push(param_trim_replace);
                    }
                }
            }, 
            None => {
                eprintln!("Invalid function definition on the first line of the source '{}'.", first_line);
                return Function {
                    name: "name".to_string(), 
                    parameters: Vec::new(), 
                    source: Vec::new()
                };
            }
        }
        
        // Return function object.
        return Function {
            name: name, 
            parameters: parameters, 
            source: source.to_vec()
        };
    }
    
    pub fn get_name(&self) -> &String {
        return &self.name;
    }
    
    pub fn get_parameters(&self) -> &Vec<String> {
        return &self.parameters;
    }
    
    pub fn get_source(&self) -> &Vec<Line> {
        return &self.source;
    }
    
}

#[derive(Debug)]
pub struct Class {
    name: String, 
    parent: String, 
    variables: Vec<Assignment>, 
    methods: Vec<ClassMethod>, // Methods in the class.
    classes: Vec<Class>, 
}

impl Class {
    
    pub fn new(source: &Vec<Line>) -> Self {
        // Get clone of source.
        let mut source: Vec<Line> = source.clone();
        
        // Remove any empty lines.
        let mut lines_to_remove: Vec<usize> = Vec::new();
        for (index, line) in source.iter().enumerate() {
            if line.get_text().trim().is_empty() {
                lines_to_remove.push(index);
            }
        }
        for index in lines_to_remove.iter().rev() {
            source.remove(*index);
        }
        
        // Get first line of the source.
        let first_line: &str = source.get(0).unwrap().get_text();
        
        // Initialize regex for getting the class name when no parent class/a parent class is present.
        let re_class_start = Regex::new(PATTERN_CLASS_START).unwrap();
        
        // Initialize class properties.
        let mut name: String = "".to_string();
        let mut parent: String = "".to_string();
        
        // Check if this class has a parent class and get name and parent.
        let class_start_capt = re_class_start.captures(&first_line);
        match class_start_capt {
            Some(a) => {
                name = a.name("name").unwrap().as_str().to_string();
                parent = a.name("parent").map(|m| m.as_str()).unwrap_or("").to_string();
            }, 
            None => ()
        }
        
        // Scan source for static variables.
        // Get indentation length from second line (empty lines are not present). The indentation pattern will always match.
        let second_line: &Line = source.get(1).unwrap();
        let num_spaces: usize = File::get_indentation_length(second_line);
        
        // Initialize regex and scan source.
        let re_class_var = Regex::new(PATTERN_CLASS_VARIABLE.replace("INDENTATION", format!("{}", num_spaces).as_str()).as_str()).unwrap();
        let mut variables: Vec<Assignment> = Vec::new();
        for line in source.iter() {
            let class_var_captures = re_class_var.captures(line.get_text());
            match class_var_captures {
                Some(_) => {
                    let assignment: Option<Assignment> = Assignment::new(line);
                    match assignment {
                        Some(b) => variables.push(b), 
                        None => println!("'{}' should have been an assignment, but wasn't. This is not supposed to happen. (Around lib.rs:661 btw)", line), 
                    }
                }
                None => continue
            }
        }
        
        // Initialize structure tracker (used for tracking methods).
        let mut method_tracker: StructureTracker = StructureTracker::new();
        let mut class_tracker: StructureTracker = StructureTracker::new();
        
        // Initialize methods vector.
        let mut methods: Vec<ClassMethod> = Vec::new();
        let mut classes: Vec<Class> = Vec::new();
        
        // Initialize regex objects for methods and classes.
        let re_function_start = Regex::new(PATTERN_FUNCTION_START).unwrap();
        let re_class_start = Regex::new(PATTERN_CLASS_START).unwrap();
        
        // Scan source for class methods.
        for (index, line) in source.iter().enumerate() {
            // Skip empty lines.
            if line.get_text().trim().is_empty() {
                continue;
            }
            
            // Get indentation length.
            let indentation_length: usize = File::get_indentation_length(line);
            if method_tracker.is_active() {
                if !method_tracker.indentation_set() {
                    // Indentation length not set, set indentation length and add line.
                    method_tracker.set_indentation_length(indentation_length);
                    method_tracker.add_line(&line);
                } else {
                    // Indentation length set.
                    if indentation_length >= method_tracker.get_indentation_length() {
                        // Not end of method, add line.
                        method_tracker.add_line(&line);
                    } else {
                        // End of method, create and push method.
                        let method: ClassMethod = ClassMethod::new(method_tracker.get_source());
                        println!("Adding classmethod with name '{}' to class '{}'.", &method.get_name(), name);
                        methods.push(method);
                        
                        // Reset tracker.
                        method_tracker.reset();
                    }
                }
            }
            if class_tracker.is_active() {
                if !class_tracker.indentation_set() {
                    // Indentation length not set, set indentation length and add line.
                    class_tracker.set_indentation_length(indentation_length);
                    class_tracker.add_line(&line);
                } else {
                    // Indentation length set.
                    if indentation_length >= class_tracker.get_indentation_length() {
                        // Not end of class, add line.
                        class_tracker.add_line(&line);
                    } else {
                        // End of class, create and push class.
                        let class: Class = Class::new(class_tracker.get_source());
                        println!("Adding class with name '{}' to class '{}'.", &class.get_name(), name);
                        classes.push(class);
                        
                        // Reset tracker.
                        class_tracker.reset();
                    }
                }
            }
            
            if method_tracker.is_active() || class_tracker.is_active() {
                continue;
            }
            
            // Check for method start.
            let function_start_capt = re_function_start.captures(line.get_text());
            match function_start_capt {
                Some(_) => {
                    method_tracker.start();
                    method_tracker.add_line(&line);
                }, 
                None => {
                    // Check if this is the first line of the class.
                    if index == 0 {
                        continue;
                    }
                    
                    // Check for class start.
                    let class_start_capt = re_class_start.captures(line.get_text());
                    match class_start_capt {
                        Some(_) => {
                            class_tracker.start();
                            class_tracker.add_line(&line);
                        }, 
                        None => continue
                    }
                }
            }
        }
        
        // Check if a method or class was getting collected when the source ended.
        if method_tracker.is_active() {
            // Create classmethod object and add to methods vector.
            let method: ClassMethod = ClassMethod::new(method_tracker.get_source());
            println!("Adding classmethod with name '{}' to class '{}'.", &method.get_name(), name);
            methods.push(method);
        }
        if class_tracker.is_active() {
            // Create class object and add to classes vector.
            let class: Class = Class::new(class_tracker.get_source());
            println!("Adding class with name '{}' to class '{}'.", &class.get_name(), name);
            classes.push(class);
        }
        
        return Class {
            name: name, 
            parent: parent, 
            variables: variables, 
            methods: methods, 
            classes: classes
        };
    }
    
    pub fn get_name(&self) -> &String {
        return &self.name;
    }
    
    pub fn get_parent(&self) -> &String {
        return &self.parent;
    }
    
    pub fn get_variables(&self) -> &Vec<Assignment> {
        return &self.variables;
    }
    
    pub fn get_methods(&self) -> &Vec<ClassMethod> {
        return &self.methods;
    }
    
    pub fn get_classes(&self) -> &Vec<Class> {
        return &self.classes;
    }
    
    pub fn get_source(&self) -> Vec<Line> {
        let mut lines: Vec<Line> = Vec::new();
        
        // Append source from all methods.
        for method in self.get_methods() {
            for line in method.get_source() {
                lines.push(line.clone());
            }
        }
        
        // Append source from all classes.
        for class in self.get_classes() {
            for line in class.get_source() {
                lines.push(line.clone());
            }
        }
        
        // Append source from all assignments (aka class variables).
        for assignment in self.get_variables() {
            lines.push(assignment.get_source().clone());
        }
        
        // Sort lines by line number.
        lines.sort_by_key(|line| line.get_number());
        
        // Get indentation from first line.
        let indentation: usize = File::get_indentation_length(lines.get(0).unwrap()) - 4;
        let mut indentation_str: String = "".to_string();
        for _ in 0..indentation {
            indentation_str.push_str(" ");
        }
        
        // Add dummy line to the start of the vector representing the class definition.
        let class_definition: Line = Line::new(lines.get(0).unwrap().get_number() - 1, format!("{}class {}({}): [FABICATED LINE]", indentation_str, self.get_name(), self.get_parent()).as_str());
        lines.reverse();
        lines.push(class_definition);
        lines.reverse();
        
        return lines;
    }
    
}

impl PartialEq for Class {
    
    fn eq(&self, other: &Self) -> bool {
        return self.get_name() == other.get_name() 
            && self.get_parent() == other.get_parent() 
            && self.get_variables() == other.get_variables() 
            && self.get_methods() == other.get_methods() 
            && self.get_classes() == other.get_classes();
    }
    
}

#[derive(Debug)]
pub struct ClassMethod {
    name: String, 
    parameters: Vec<String>, 
    source: Vec<Line>, // Lines in the method.
}

impl ClassMethod {
    
    fn new(source: &Vec<Line>) -> Self {
        // Get first line.
        let first_line: &str = source.get(0).unwrap().get_text();
        
        // Initialize regex for getting the function name and parameters.
        let re_function_start = Regex::new(PATTERN_FUNCTION_START).unwrap();
        let function_start_capt = re_function_start.captures(&first_line);
        
        // Get name and params from first line.
        let mut name: String = "".to_string();
        let mut params: String = "".to_string();
        match function_start_capt {
            Some(a) => {
                name = a["name"].to_string();
                params = a["params"].to_string();
            }, 
            None => {
                eprintln!("This state should not be reached(?), method definition does not match. '{}'", first_line);
                return ClassMethod {
                    name: "name".to_string(), 
                    parameters: vec![], 
                    source: vec![]
                };
            }
        }
        
        // Get parameters from params string.
        let mut in_square_brackets: bool = false;
        let mut parameters: Vec<String> = Vec::new();
        parameters.push(String::from(""));
        for c in params.chars() {
            if c == ',' {
                if in_square_brackets {
                    let last_parameter = parameters.last_mut().unwrap();
                    last_parameter.push(c);
                } else {
                    parameters.push(String::from(""));
                }
            } else {
                let last_parameter = parameters.last_mut().unwrap();
                last_parameter.push(c);
                if c == '[' {
                    in_square_brackets = true;
                } else if c == ']' {
                    in_square_brackets = false;
                }
            }
        }
        
        // Trim away leading and trailing spaces.
        for parameter in &mut parameters {
            *parameter = String::from(parameter.trim());
        }
        
        return ClassMethod {
            name: name, 
            parameters: parameters, 
            source: remove_empty_lines(source.to_vec())
        };
    }
    
    pub fn get_name(&self) -> &String {
        return &self.name;
    }
    
    pub fn get_parameters(&self) -> &Vec<String> {
        return &self.parameters;
    }
    
    pub fn get_source(&self) -> &Vec<Line> {
        return &self.source;
    }
    
}

impl PartialEq for ClassMethod {
    
    fn eq(&self, other: &Self) -> bool {
        return self.get_name() == other.get_name() 
            && self.get_parameters() == other.get_parameters() 
            && self.get_source() == other.get_source();
    }
    
}

pub fn get_file_lines(filename: &str) -> Result<Vec<String>, io::Error> {
    let mut result: Vec<String> = Vec::new();
    let contents = fs::read_to_string(filename)?;
    for line in contents.lines() {
        result.push(line.to_string());
    }
    return Ok(result);
}

pub fn get_lines_for_test(filename: &str) -> Vec<String> {
    return get_file_lines(filename).unwrap();
}

pub fn vec_str_to_vec_line(source: &Vec<String>) -> Vec<Line> {
    let mut lines: Vec<Line> = Vec::new();
    for (index, text) in source.iter().enumerate() {
        lines.push(Line::new(index + 1, text));
    }
    return lines;
}

pub fn remove_empty_lines(mut source: Vec<Line>) -> Vec<Line> {
    // Get indices to remove.
    let mut indices_to_remove: Vec<usize> = Vec::new();
    for (index, line) in source.iter().enumerate() {
        if line.get_text().trim().is_empty() {
            indices_to_remove.push(index);
        }
    }
    
    // Remove indices.
    for index in indices_to_remove.iter().rev() {
        source.remove(*index);
    }
    
    // Return new source.
    return source;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use std::collections::HashMap;
    
    #[test]
    fn test_regex_pattern_indentation() {
        // Test PATTERN_INDENTATION (should match anything).
        // Construct hashmap containing strings to match.
        let mut test_strings: HashMap<u32, &str> = HashMap::new();
        test_strings.insert(0, "     Hello world!  \\v   \t\t\t 文閩音");
        test_strings.insert(1, "        self.start_time = time.time()");
        test_strings.insert(2, "        result[\"gc_uncollectable\"] = self.gc_uncollectable  ");
        test_strings.insert(3, "class GcLogger:");
        test_strings.insert(4, "            if i % 3 == 0:");
        test_strings.insert(5, "    print(\"hihi\")");
        test_strings.insert(6, "    ");
        test_strings.insert(7, "    pub fn create(number: usize, text: &str) -> Self {");
        test_strings.insert(8, "                                                                    \t\t\t\t\t\t\t\t    print(\"Banaan\")");
        
        // Construct hashmap containing hashmaps containing values of named groups.
        let mut test_matches: HashMap<u32, HashMap<&str, &str>> = HashMap::new();
        test_matches.insert(0, HashMap::from([("indentation", "     ")]));
        test_matches.insert(1, HashMap::from([("indentation", "        ")]));
        test_matches.insert(2, HashMap::from([("indentation", "        ")]));
        test_matches.insert(3, HashMap::from([("indentation", "")]));
        test_matches.insert(4, HashMap::from([("indentation", "            ")]));
        test_matches.insert(5, HashMap::from([("indentation", "    ")]));
        test_matches.insert(6, HashMap::from([("indentation", "    ")]));
        test_matches.insert(7, HashMap::from([("indentation", "    ")]));
        test_matches.insert(8, HashMap::from([("indentation", "                                                                    \t\t\t\t\t\t\t\t    ")]));
        
        // Run tests.
        let re = Regex::new(PATTERN_INDENTATION).unwrap();
        for (key_str, value_str) in test_strings.iter() {
            let capt = re.captures(value_str);
            let map = test_matches.get(&key_str).unwrap();
            match capt {
                Some(a) => {
                    for (key, value) in map.iter() {
                        assert_eq!(&&a[*key], value);
                    }
                }, 
                None => panic!("ERROR: String '{}' should have matched 'PATTERN_INDENTATION', but didn't.", value_str)
            }
        }
    }
    
    #[test]
    fn test_regex_pattern_import() {
        // Test PATTERN_IMPORT.
        // Construct hashmap containing strings to match.
        let mut test_strings: HashMap<u32, &str> = HashMap::new();
        test_strings.insert(0, "import math");
        test_strings.insert(1, "   import     sys     \t,    \t re \t  , \t\tdatetime\t   ,  \t   zoneinfo  \t ");
        test_strings.insert(2, "  \t  import a  \t  ,   b   \t\t\t   ");
        test_strings.insert(3, "        \t\timport  \t time  ");
        test_strings.insert(4, "import mypy.errorcodes as codes");
        test_strings.insert(5, "    import mypy.checkexpr");
        test_strings.insert(6, "import glob as fileglob");
        test_strings.insert(7, "    import tomllib");
        test_strings.insert(8, "         \t\t\t\t   import       banaaaan     as     \t\t\t    appel     \t\t\t      ");
        
        // Construct hashmap containing hashmaps containing values of named groups.
        let mut test_matches: HashMap<u32, HashMap<&str, &str>> = HashMap::new();
        test_matches.insert(0, HashMap::from([("modules", "math")]));
        test_matches.insert(1, HashMap::from([("modules", "sys     \t,    \t re \t  , \t\tdatetime\t   ,  \t   zoneinfo  \t ")]));
        test_matches.insert(2, HashMap::from([("modules", "a  \t  ,   b   \t\t\t   ")]));
        test_matches.insert(3, HashMap::from([("modules", "time  ")]));
        test_matches.insert(4, HashMap::from([("modules", "mypy.errorcodes as codes")]));
        test_matches.insert(5, HashMap::from([("modules", "mypy.checkexpr")]));
        test_matches.insert(6, HashMap::from([("modules", "glob as fileglob")]));
        test_matches.insert(7, HashMap::from([("modules", "tomllib")]));
        test_matches.insert(8, HashMap::from([("modules", "banaaaan     as     \t\t\t    appel     \t\t\t      ")]));
        
        // Run tests.
        let re = Regex::new(PATTERN_IMPORT).unwrap();
        for (key_str, value_str) in test_strings.iter() {
            let capt = re.captures(value_str);
            let map = test_matches.get(&key_str).unwrap();
            match capt {
                Some(a) => {
                    for (key, value) in map.iter() {
                        assert_eq!(&&a[*key], value);
                    }
                }, 
                None => panic!("ERROR: String '{}' should have matched 'PATTERN_IMPORT', but didn't.", value_str)
            }
        }
    }
    #[test]
    fn test_regex_pattern_from_import() {
        // Test PATTERN_FROM_IMPORT.
        // Construct hashmap containing strings to match.
        let mut test_strings: HashMap<u32, &str> = HashMap::new();
        test_strings.insert(0, "from a import b as c");
        test_strings.insert(1, "   \t\t\t    from     \t d\timport     e    as   f   ,   g   ,   h   \t\t\t   as i  \t ");
        test_strings.insert(2, "from j import k aas, baas as p oop, f ish as dog, clo se as you       tube");
        test_strings.insert(3, "from mypy.options import PER_MODULE_OPTIONS, Options");
        test_strings.insert(4, "from     numpy.core.multiarray     import    \t\t _flagdict    \t,  \t flagsobj  \t     \t\t\t");
        test_strings.insert(5, "from mypy.infer import ArgumentInferContext, infer_function_type_arguments, infer_type_arguments");
        test_strings.insert(6, "from mypy import applytype, erasetype, join, message_registry, nodes, operators, types");
        test_strings.insert(7, "   \t\t\t from    \t\t        \t\t\t\t\t\t\t   mypy.semanal_enum        import         \t\t\t\tENUM_BASES");
        test_strings.insert(8, "    from . import _distributor_init");
        test_strings.insert(9, "        from numpy.__config__ import show as show_config");
        
        // Construct hashmap containing hashmaps containing values of named groups.
        let mut test_matches: HashMap<u32, HashMap<&str, &str>> = HashMap::new();
        test_matches.insert(0, HashMap::from([("module", "a"), ("objects", "b as c")]));
        test_matches.insert(1, HashMap::from([("module", "d"), ("objects", "e    as   f   ,   g   ,   h   \t\t\t   as i  \t ")]));
        test_matches.insert(2, HashMap::from([("module", "j"), ("objects", "k aas, baas as p oop, f ish as dog, clo se as you       tube")]));
        test_matches.insert(3, HashMap::from([("module", "mypy.options"), ("objects", "PER_MODULE_OPTIONS, Options")]));
        test_matches.insert(4, HashMap::from([("module", "numpy.core.multiarray"), ("objects", "_flagdict    \t,  \t flagsobj  \t     \t\t\t")]));
        test_matches.insert(5, HashMap::from([("module", "mypy.infer"), ("objects", "ArgumentInferContext, infer_function_type_arguments, infer_type_arguments")]));
        test_matches.insert(6, HashMap::from([("module", "mypy"), ("objects", "applytype, erasetype, join, message_registry, nodes, operators, types")]));
        test_matches.insert(7, HashMap::from([("module", "mypy.semanal_enum"), ("objects", "ENUM_BASES")]));
        test_matches.insert(8, HashMap::from([("module", "."), ("objects", "_distributor_init")]));
        test_matches.insert(9, HashMap::from([("module", "numpy.__config__"), ("objects", "show as show_config")]));
        
        
        // Run tests.
        let re = Regex::new(PATTERN_FROM_IMPORT).unwrap();
        for (key_str, value_str) in test_strings.iter() {
            let capt = re.captures(value_str);
            let map = test_matches.get(&key_str).unwrap();
            match capt {
                Some(a) => {
                    for (key, value) in map.iter() {
                        assert_eq!(&&a[*key], value);
                    }
                }, 
                None => panic!("ERROR: String '{}' should have matched 'PATTERN_FROM_IMPORT', but didn't.", value_str)
            }
        }
    }
    
    #[test]
    fn test_regex_pattern_global_variable() {
        // Test PATTERN_GLOBAL_VARIABLE.
        // Construct hashmap containing strings to match.
        let mut test_strings: HashMap<u32, &str> = HashMap::new();
        test_strings.insert(0, "_flagnames    =    ['C_CONTIGUOUS', 'F_CONTIGUOUS'    , 'ALIGNED'   ,   'WRITEABLE', 'OWNDATA', 'WRITEBACKIFCOPY']");
        test_strings.insert(1, "_pointer_type_cache = {}");
        test_strings.insert(2, "    __NUMPY_SETUP__ = False");
        test_strings.insert(3, "    __all__ = ['exceptions', 'ModuleDeprecationWarning', 'VisibleDeprecationWarning', 'ComplexWarning', 'TooHardError', 'AxisError']");
        test_strings.insert(4, "GLOB1 = 1");
        test_strings.insert(5, "    GLOB_PARAMETER = 100 ** 2");
        test_strings.insert(6, "GLOB_NAME = \"Bananas are pretty good\"");
        test_strings.insert(7, "GLOB_OBJ: int = time.time()");
        test_strings.insert(8, "       GLOBAL_MAP: List[Tuple[np.uint16, List[str, int]], str]     \t\t\t    =     []   \t\t\t \t   \t");
        
        // Construct hashmap containing hashmaps containing values of named groups.
        let mut test_matches: HashMap<u32, HashMap<&str, &str>> = HashMap::new();
        test_matches.insert(0, HashMap::from([("varname", "_flagnames")]));
        test_matches.insert(1, HashMap::from([("varname", "_pointer_type_cache")]));
        test_matches.insert(2, HashMap::from([("varname", "__NUMPY_SETUP__")]));
        test_matches.insert(3, HashMap::from([("varname", "__all__")]));
        test_matches.insert(4, HashMap::from([("varname", "GLOB1")]));
        test_matches.insert(5, HashMap::from([("varname", "GLOB_PARAMETER")]));
        test_matches.insert(6, HashMap::from([("varname", "GLOB_NAME")]));
        test_matches.insert(7, HashMap::from([("varname", "GLOB_OBJ")]));
        test_matches.insert(8, HashMap::from([("varname", "GLOBAL_MAP")]));
        
        // Run tests.
        let re = Regex::new(PATTERN_GLOBAL_VARIABLE).unwrap();
        for (key_str, value_str) in test_strings.iter() {
            let capt = re.captures(value_str);
            let map = test_matches.get(&key_str).unwrap();
            match capt {
                Some(a) => {
                    for (key, value) in map.iter() {
                        assert_eq!(&&a[*key], value);
                    }
                }, 
                None => panic!("ERROR: String '{}' should have matched 'PATTERN_GLOBAL_VARIABLE', but didn't.", value_str)
            }
        }
    }
    
    #[test]
    fn test_regex_pattern_function_start() {
        // Test PATTERN_FUNCTION_START.
        // Construct hashmap containing strings to match.
        let mut test_strings: HashMap<u32, &str> = HashMap::new();
        test_strings.insert(0, "def zeros(shape, dtype=None, order='C'):");
        test_strings.insert(1, "def eye(n,M=None, k=0, dtype=float, order='C'):");
        test_strings.insert(2, "    def __array_finalize__(self, obj):");
        test_strings.insert(3, "    def __mul__(self, other):  ");
        test_strings.insert(4, "    def sum(self, axis=None, dtype=None, out=None):");
        test_strings.insert(5, "    def prod(self, axis=None, dtype=None, out=None):");
        test_strings.insert(6, "    def run_case(self, testcase: DataDrivenTestCase) -> None:");
        test_strings.insert(7, "def columns(self, *cols: ColumnClause[Any], **types: Union[TypeEngine[Any], Type[TypeEngine[Any]]]) -> TextAsFrom: ");
        test_strings.insert(8, "    def self_group(self: _CL, against: Optional[Any] = ...) -> Union[_CL, Grouping[Any]]:");
        test_strings.insert(9, "         \t\t\tdef    func   (self, a=[5, 6, \"a\"], b, c, d: List[Tuple[str]]=(5, 6, 7, banaan), _str: bool=False)    ->     List[Tuple[str, int], str]  :   \t\t \t\t    ");
        
        // Construct hashmap containing hashmaps containing values of named groups.
        let mut test_matches: HashMap<u32, HashMap<&str, &str>> = HashMap::new();
        test_matches.insert(0, HashMap::from([("indentation", ""), ("name", "zeros"), ("params", "shape, dtype=None, order='C'")]));
        test_matches.insert(1, HashMap::from([("indentation", ""), ("name", "eye"), ("params", "n,M=None, k=0, dtype=float, order='C'")]));
        test_matches.insert(2, HashMap::from([("indentation", "    "), ("name", "__array_finalize__"), ("params", "self, obj")]));
        test_matches.insert(3, HashMap::from([("indentation", "    "), ("name", "__mul__"), ("params", "self, other")]));
        test_matches.insert(4, HashMap::from([("indentation", "    "), ("name", "sum"), ("params", "self, axis=None, dtype=None, out=None")]));
        test_matches.insert(5, HashMap::from([("indentation", "    "), ("name", "prod"), ("params", "self, axis=None, dtype=None, out=None")]));
        test_matches.insert(6, HashMap::from([("indentation", "    "), ("name", "run_case"), ("params", "self, testcase: DataDrivenTestCase")]));
        test_matches.insert(7, HashMap::from([("indentation", ""), ("name", "columns"), ("params", "self, *cols: ColumnClause[Any], **types: Union[TypeEngine[Any], Type[TypeEngine[Any]]]")]));
        test_matches.insert(8, HashMap::from([("indentation", "    "), ("name", "self_group"), ("params", "self: _CL, against: Optional[Any] = ...")]));
        test_matches.insert(9, HashMap::from([("indentation", "         \t\t\t"), ("name", "func"), ("params", "self, a=[5, 6, \"a\"], b, c, d: List[Tuple[str]]=(5, 6, 7, banaan), _str: bool=False")]));
        
        // Run tests.
        let re = Regex::new(PATTERN_FUNCTION_START).unwrap();
        for (key_str, value_str) in test_strings.iter() {
            let capt = re.captures(value_str);
            let map = test_matches.get(&key_str).unwrap();
            match capt {
                Some(a) => {
                    for (key, value) in map.iter() {
                        assert_eq!(&&a[*key], value);
                    }
                }, 
                None => panic!("ERROR: String '{}' should have matched 'PATTERN_FUNCTION_START', but didn't.", value_str)
            }
        }
    }
    
    #[test]
    fn test_regex_pattern_class_start() {
        // Test PATTERN_CLASS_START.
        // Construct hashmap containing strings to match.
        let mut test_strings: HashMap<u32, &str> = HashMap::new();
        test_strings.insert(0, "class BindParameter(ColumnElement[_T]):");
        test_strings.insert(1, "class Triangle:");
        test_strings.insert(2, "    class Rect(Shape):");
        test_strings.insert(3, "class ModuleWrapper(nn.Module):");
        test_strings.insert(4, "class UntypedStorage(torch._C.StorageBase, _StorageBase):");
        test_strings.insert(5, "                  \t\t\tclass Library:    \t\t  \t\t");
        test_strings.insert(6, "class SourceChangeWarning(Warning):");
        test_strings.insert(7, "     \t\t\t\t\t\t            class ETKernelIndex:   ");
        
        // Construct hashmap containing hashmaps containing values of named groups.
        let mut test_matches: HashMap<u32, HashMap<&str, &str>> = HashMap::new();
        test_matches.insert(0, HashMap::from([("indentation", ""), ("name", "BindParameter"), ("parent", "ColumnElement[_T]")]));
        test_matches.insert(1, HashMap::from([("indentation", ""), ("name", "Triangle"), ("parent", "")]));
        test_matches.insert(2, HashMap::from([("indentation", "    "), ("name", "Rect"), ("parent", "Shape")]));
        test_matches.insert(3, HashMap::from([("indentation", ""), ("name", "ModuleWrapper"), ("parent", "nn.Module")]));
        test_matches.insert(4, HashMap::from([("indentation", ""), ("name", "UntypedStorage"), ("parent", "torch._C.StorageBase, _StorageBase")]));
        test_matches.insert(5, HashMap::from([("indentation", "                  \t\t\t"), ("name", "Library"), ("parent", "")]));
        test_matches.insert(6, HashMap::from([("indentation", ""), ("name", "SourceChangeWarning"), ("parent", "Warning")]));
        test_matches.insert(7, HashMap::from([("indentation", "     \t\t\t\t\t\t            "), ("name", "ETKernelIndex"), ("parent", "")]));
        
        // Run tests.
        let re = Regex::new(PATTERN_CLASS_START).unwrap();
        for (key_str, value_str) in test_strings.iter() {
            let capt = re.captures(value_str);
            let map = test_matches.get(&key_str).unwrap();
            match capt {
                Some(a) => {
                    for (key, value) in map.iter() {
                        if key == &"parent" {
                            assert_eq!(&a.name("parent").map(|m| m.as_str()).unwrap_or(""), value);
                        } else {
                            assert_eq!(&&a[*key], value);
                        }
                    }
                }, 
                None => panic!("ERROR: String '{}' should have matched 'PATTERN_CLASS_START', but didn't.", value_str)
            }
        }
    }
    
    #[test]
    fn test_regex_pattern_class_variable() {
        // Test PATTERN_CLASS_VARIABLE.
        // Construct hashmap containing strings to match.
        let mut test_strings: HashMap<u32, &str> = HashMap::new();
        test_strings.insert(0, "    arg_meta: Tuple[ETKernelKeyOpArgMeta, ...] = ()");
        test_strings.insert(1, "    default: bool = False");
        test_strings.insert(2, "    version: int = KERNEL_KEY_VERSION");
        test_strings.insert(3, "        CLASS_VAR   =     5");
        test_strings.insert(4, "    instructions = 1");
        test_strings.insert(5, "    MAXDIM = 21201");
        test_strings.insert(6, "        CLASS_STR   = \t\t\t\t  \"Bananas are very                  spacyyyyyyyyy\"    ");
        test_strings.insert(7, "    deserialized_objects = {}");
        
        // Construct hashmap containing the indentations for each string to replace in the regex.
        let mut test_string_indentations: HashMap<u32, u32> = HashMap::new();
        test_string_indentations.insert(0, 4);
        test_string_indentations.insert(1, 4);
        test_string_indentations.insert(2, 4);
        test_string_indentations.insert(3, 8);
        test_string_indentations.insert(4, 4);
        test_string_indentations.insert(5, 4);
        test_string_indentations.insert(6, 8);
        test_string_indentations.insert(7, 4);
        
        // Construct hashmap containing hashmaps containing values of named groups.
        let mut test_matches: HashMap<u32, HashMap<&str, &str>> = HashMap::new();
        test_matches.insert(0, HashMap::from([("varname", "arg_meta"), ("varvalue", "()")]));
        test_matches.insert(1, HashMap::from([("varname", "default"), ("varvalue", "False")]));
        test_matches.insert(2, HashMap::from([("varname", "version"), ("varvalue", "KERNEL_KEY_VERSION")]));
        test_matches.insert(3, HashMap::from([("varname", "CLASS_VAR"), ("varvalue", "5")]));
        test_matches.insert(4, HashMap::from([("varname", "instructions"), ("varvalue", "1")]));
        test_matches.insert(5, HashMap::from([("varname", "MAXDIM"), ("varvalue", "21201")]));
        test_matches.insert(6, HashMap::from([("varname", "CLASS_STR"), ("varvalue", "\"Bananas are very                  spacyyyyyyyyy\"    ")]));
        test_matches.insert(7, HashMap::from([("varname", "deserialized_objects"), ("varvalue", "{}")]));
        
        // Run tests.
        for (key_str, value_str) in test_strings.iter() {
            let num_spaces = test_string_indentations.get(&key_str).unwrap();
            let re = Regex::new(PATTERN_CLASS_VARIABLE.replace("INDENTATION", format!("{}", num_spaces).as_str()).as_str()).unwrap();
            let capt = re.captures(value_str);
            let map = test_matches.get(&key_str).unwrap();
            match capt {
                Some(a) => {
                    for (key, value) in map.iter() {
                        assert_eq!(&&a[*key], value);
                    }
                }, 
                None => panic!("ERROR: String '{}' should have matched 'PATTERN_CLASS_VARIABLE', but didn't.", value_str)
            }
        }
    }
    
    #[test]
    fn test_create_line() {
        let test_cases: Vec<(usize, &str)> = vec![(25, "Hi there"), (100, "This is some string with w31rd characters \
            !_(*)`~|\\[]{};:'\",.<>/?!@#$%^&*()_+-=說文閩音通한국어 키보드乇乂ㄒ尺卂　ㄒ卄丨匚匚Моя семья"), (1000000000, "Big line number")];
        
        for (line_number, text) in test_cases {
            let line = Line::new(line_number, text);
            let line_want = Line {number: line_number, text: text.to_string()};
            assert_eq!(line, line_want);
        }
    }
    
    #[test]
    fn test_line_is_assignment() {
        let test_lines: Vec<Line> = vec![
            Line::new( 1, "var = 1"), 
            Line::new(56, "variable: int = \"This is an = sign\""), 
            Line::new(34, "if glob == 5:"), 
            Line::new(69, "if blob >= \"False != True = = = \""), 
            Line::new(25, "qwerty <= [var = 5]"), 
            Line::new(62, "not_equal = var != 5"), 
            Line::new(43, "except ImportError:"), 
            Line::new(18, "    import numpy.core._internal as nic"), 
            Line::new(28, "        >>> lib = ctypes.cdll[<full_path_name>] # doctest: +SKIP"), 
            Line::new(28, "                base_ext = \".dylib\""), 
            Line::new(35, "            libname_ext = [libname + base_ext]"), 
            Line::new(28, "                libname_ext.insert(0, libname + so_ext)"), 
            Line::new(81, "            libname_ext = [libname]"), 
            Line::new(40, "            libdir = os.path.dirname(loader_path)"), 
            Line::new(43, "def _num_fromflags(flaglist):"), 
            Line::new(85, "def ndpointer(dtype=None, ndim=None, shape=None, flags=None):"), 
            Line::new(95, "    num = None"), 
            Line::new(53, "            shape = (shape,)"), 
            Line::new(73, "    if ndim is not None:"), 
            Line::new(92, "        name += \"_\"+\"x\".join(str(x) for x in shape)"), 
            Line::new(48, "    _pointer_type_cache[cache_key] = klass"), 
            Line::new( 4, "        dtype_native = dtype.newbyteorder('=')"), 
            Line::new(52, "var = [g=5, t=6]"), 
            Line::new(83, "d = {\"a\": g==5, \"b\": t=7}"), 
            Line::new(78, "tup   = (b=5, c=7, v==10)"), 
        ];
        
        let test_results: Vec<Option<usize>> = vec![
            Some(4), 
            Some(14), 
            None, 
            None, 
            None, 
            Some(10), 
            None, 
            None, 
            Some(16), 
            Some(25), 
            Some(24), 
            None, 
            Some(24), 
            Some(19), 
            None, 
            None, 
            Some(8), 
            Some(18), 
            None, 
            None, 
            Some(35), 
            Some(21), 
            Some(4), 
            Some(2), 
            Some(6), 
        ];
        
        for (line, expected_result) in test_lines.iter().zip(test_results.iter()) {
            let result: Option<usize> = line.is_assignment();
            assert_eq!(&result, expected_result);
        }
    }
    
    #[test]
    fn test_create_assignment() {
        let test_lines: Vec<Line> = vec![
            Line::new(15, "                self.banana = banana"), 
            Line::new(72, "            LOWER_GLOB = \"LowerClass class variable\""), 
            Line::new(63, "    class SubRect(object):"), 
            Line::new(43, "    class_var1 = 5"), 
            Line::new(90, "        print(\"Yes init\")"), 
            Line::new(26, "            self.gc_collected += info[\"collected\"]"), 
            Line::new(12, "            self.gc_collected = info[\"collected\"]"), 
            Line::new(83, "    def gc_callback(self, phase: str, info: Mapping[str, int]) -> None:"), 
            Line::new(13, "torch.repeat_interleave(x, dim=2, repeats=n_rep)"), 
            Line::new(76, "a = torch.repeat_interleave(x, dim=2, repeats=n_rep)"), 
        ];
        
        let test_results: Vec<Option<Assignment>> = vec![
            Some(Assignment {name: "self.banana".to_string(), value: "banana".to_string(), source: test_lines.get(0).unwrap().clone()}), 
            Some(Assignment {name: "LOWER_GLOB".to_string(), value: "\"LowerClass class variable\"".to_string(), source: test_lines.get(1).unwrap().clone()}), 
            None, 
            Some(Assignment {name: "class_var1".to_string(), value: "5".to_string(), source: test_lines.get(3).unwrap().clone()}), 
            None, 
            None, 
            Some(Assignment {name: "self.gc_collected".to_string(), value: "info[\"collected\"]".to_string(), source: test_lines.get(6).unwrap().clone()}), 
            None, 
            None, 
            Some(Assignment {name: "a".to_string(), value: "torch.repeat_interleave(x, dim=2, repeats=n_rep)".to_string(), source: test_lines.get(9).unwrap().clone()}), 
        ];
        
        for (line, expected_result) in test_lines.iter().zip(test_results.iter()) {
            let result: Option<Assignment> = Assignment::new(line);
            assert_eq!(&result, expected_result);
        }
    }
    
    #[test]
    fn test_create_function() {
        let lines_str: Vec<String> = get_lines_for_test("test/create_function.py");
        let lines: Vec<Line> = vec_str_to_vec_line(&lines_str);
        let func: Function = Function::new(&lines);
        
        let func_name_want: String = String::from("func_name");
        let func_params_want: Vec<String> = vec!["param1".to_string(), "param2".to_string(), "param3=5".to_string(), "*args".to_string(), "**kwargs".to_string()];
        let func_want: Function = Function {name: func_name_want, parameters: func_params_want, source: lines};
        assert_eq!(func, func_want);
    }
    
    #[test]
    fn test_create_class() {
        let lines_str: Vec<String> = get_lines_for_test("test/create_class.py");
        let lines: Vec<Line> = vec_str_to_vec_line(&lines_str);
        let class: Class = Class::new(&lines);
        
        let class_name_want: String = String::from("Rect");
        let class_parent_want: String = String::from("Shape");
        let class_variables_want: Vec<Assignment> = vec![
            Assignment::new(lines.get(2).unwrap()).unwrap(), 
            Assignment::new(lines.get(8).unwrap()).unwrap(), 
            Assignment::new(lines.get(9).unwrap()).unwrap(), 
            Assignment::new(lines.get(15).unwrap()).unwrap()
        ];
        let class_methods_want: Vec<ClassMethod> = vec![ClassMethod::new(&lines[4..=6].to_vec()), ClassMethod::new(&lines[11..=13].to_vec())];
        let class_classes_want: Vec<Class> = vec![];
        let class_want: Class = Class {name: class_name_want, parent: class_parent_want, variables: class_variables_want, methods: class_methods_want, classes: class_classes_want};
        
        assert_eq!(class, class_want);
    }
    
    #[test]
    fn test_create_classmethod() {
        let lines_str: Vec<String> = get_lines_for_test("test/create_classmethod.py");
        let lines: Vec<Line> = vec_str_to_vec_line(&lines_str);
        let classmethod: ClassMethod = ClassMethod::new(&lines);
        
        let classmethod_name_want: String = String::from("class_method");
        let classmethod_params_want: Vec<String> = vec!["self".to_string(), "a".to_string(), "b=10".to_string(), "c=5".to_string(), 
            "d = 15".to_string(), "*args".to_string(), "**kwargs".to_string()];
        let classmethod_source_want: Vec<Line> = remove_empty_lines(lines);
        let classmethod_want = ClassMethod {name: classmethod_name_want, parameters: classmethod_params_want, source: classmethod_source_want};
        assert_eq!(classmethod, classmethod_want);
    }
    
    #[test]
    fn test_function_at_end_of_file_no_newline() {
        let lines_str: Vec<String> = get_lines_for_test("test/function_at_end_of_file_no_newline.py");
        let lines: Vec<Line> = vec_str_to_vec_line(&lines_str);
        let function: Function = Function::new(&lines);
        
        let function_name_want: String = "function".to_string();
        let function_parameters_want: Vec<String> = vec!["param1".to_string(), "param2=5".to_string()];
        let function_source_want: Vec<Line> = remove_empty_lines(lines);
        let function_want: Function = Function {name: function_name_want, parameters: function_parameters_want, source: function_source_want};
        assert_eq!(function, function_want);
    }
    
    #[test]
    fn test_file() {
        let files: Vec<&str> = vec![
            "test/mypy_gclogger.py", 
            "test/recursive_classes.py", 
        ];
        
        let expected_results: Vec<File> = vec![
            File {
                name: "mypy_gclogger".to_string(), 
                imports: vec!["annotations".to_string(), "gc".to_string(), "time".to_string(), "Mapping".to_string()], 
                global_variables: vec!["GLOB_NAME".to_string(), "GLOB_PARAMETER".to_string(), "GLOB_OBJ".to_string()], 
                functions: vec![
                    Function {
                        name: "random_function".to_string(), 
                        parameters: vec!["param1".to_string(), "p2".to_string(), "p3".to_string(), "p4".to_string(), "p5=3".to_string()], 
                        source: vec![
                            Line::new(13, "def random_function(param1, p2, p3, p4, p5=3):"),
                            Line::new(14, "    print(\"hihi\")"),
                            Line::new(15, "    for i in range(10):"),
                            Line::new(16, "        if i % 2 == 0:"),
                            Line::new(17, "            print(f\"number {i}\")"),
                            Line::new(18, "            if i % 3 == 0:"),
                            Line::new(19, "                print(\"Divisible by 6!\")"),
                            Line::new(20, "        else:"),
                            Line::new(21, "            print(\"Do nothing\")"),
                            Line::new(22, "            continue"),
                            Line::new(24, "    print(\"End of function\")")
                        ]
                    }
                ], // end of functions
                classes: vec![
                    Class {
                        name: "GcLogger".to_string(), 
                        parent: "".to_string(), 
                        variables: vec![], 
                        methods: vec![
                            ClassMethod {
                                name: "__enter__".to_string(), 
                                parameters: vec!["self".to_string()], 
                                source: vec![
                                    Line::new(29, "    def __enter__(self) -> GcLogger:"),
                                    Line::new(30, "        self.gc_start_time: float | None = None"),
                                    Line::new(31, "        self.gc_time = 0.0"),
                                    Line::new(32, "        self.gc_calls = 0"),
                                    Line::new(33, "        self.gc_collected = 0"),
                                    Line::new(34, "        self.gc_uncollectable = 0"),
                                    Line::new(35, "        gc.callbacks.append(self.gc_callback)"),
                                    Line::new(36, "        self.start_time = time.time()"),
                                    Line::new(37, "        return self")
                                ]
                            }, 
                            ClassMethod {
                                name: "gc_callback".to_string(), 
                                parameters: vec!["self".to_string(), "phase: str".to_string(), "info: Mapping[str, int]".to_string()], 
                                source: vec![
                                    Line::new(39, "    def gc_callback(self, phase: str, info: Mapping[str, int]) -> None:"),
                                    Line::new(40, "        if phase == \"start\":"),
                                    Line::new(41, "            assert self.gc_start_time is None, \"Start phase out of sequence\""),
                                    Line::new(42, "            self.gc_start_time = time.time()"),
                                    Line::new(43, "        elif phase == \"stop\":"),
                                    Line::new(44, "            assert self.gc_start_time is not None, \"Stop phase out of sequence\""),
                                    Line::new(45, "            self.gc_calls += 1"),
                                    Line::new(46, "            self.gc_time += time.time() - self.gc_start_time"),
                                    Line::new(47, "            self.gc_start_time = None"),
                                    Line::new(48, "            self.gc_collected += info[\"collected\"]"),
                                    Line::new(49, "            self.gc_uncollectable += info[\"uncollectable\"]"),
                                    Line::new(50, "        else:"),
                                    Line::new(51, "            assert False, f\"Unrecognized gc phase ({phase!r})\"")
                                ]
                            }, 
                            ClassMethod {
                                name: "__exit__".to_string(), 
                                parameters: vec!["self".to_string(), "*args: object".to_string()], 
                                source: vec![
                                        Line::new(53, "    def __exit__(self, *args: object) -> None:"),
                                        Line::new(54, "        while self.gc_callback in gc.callbacks:"),
                                        Line::new(55, "            gc.callbacks.remove(self.gc_callback)")
                                ]
                            }, 
                            ClassMethod {
                                name: "get_stats".to_string(), 
                                parameters: vec!["self".to_string()], 
                                source: vec![
                                        Line::new(57, "    def get_stats(self) -> Mapping[str, float]:"),
                                        Line::new(58, "        end_time = time.time()"),
                                        Line::new(59, "        result = {}"),
                                        Line::new(60, "        result[\"gc_time\"] = self.gc_time"),
                                        Line::new(61, "        result[\"gc_calls\"] = self.gc_calls"),
                                        Line::new(62, "        result[\"gc_collected\"] = self.gc_collected"),
                                        Line::new(63, "        result[\"gc_uncollectable\"] = self.gc_uncollectable"),
                                        Line::new(64, "        result[\"build_time\"] = end_time - self.start_time"),
                                        Line::new(65, "        return result")
                                ]
                            }
                        ], 
                        classes: vec![], 
                    }, // end of class
                ] // end of classes
            }, // end of file
            File {
                name: "recursive_classes".to_string(), 
                imports: vec!["math".to_string()], 
                global_variables: vec!["SETTING".to_string()], 
                functions: vec![
                    Function {
                        name: "main".to_string(), 
                        parameters: vec![], 
                        source: vec![
                            Line::new(40, "def main():"), 
                            Line::new(41, "    upper = UpperClass(5, 6)")
                        ]
                    }
                ], // end of functions
                classes: vec![
                    Class {
                        name: "UpperClass".to_string(), 
                        parent: "object".to_string(), 
                        variables: vec![
                            Assignment::new(&Line::new(6, "    BANANA = \"Banana\"")).unwrap()
                        ], 
                        methods: vec![
                            ClassMethod {
                                name: "__init__".to_string(), 
                                parameters: vec!["self".to_string(), "a".to_string(), "b".to_string()], 
                                source: vec![
                                    Line::new(29, "    def __init__(self, a, b):"),
                                    Line::new(30, "        def define_c():"),
                                    Line::new(31, "            self.c = 5"),
                                    Line::new(33, "        define_c()"),
                                    Line::new(34, "        self.a = [a, b, self.c + 1]"),
                                    Line::new(35, "        self.b = 56"),
                                ]
                            }, 
                            ClassMethod {
                                name: "print".to_string(), 
                                parameters: vec!["self".to_string()], 
                                source: vec![
                                    Line::new(37, "    def print(self):"), 
                                    Line::new(38, "        print(self.a, self.b, self.c)")
                                ]
                            }
                        ], 
                        classes: vec![
                            Class {
                                name: "MiddleClass".to_string(), 
                                parent: "Rect".to_string(), 
                                variables: vec![], 
                                methods: vec![
                                    ClassMethod {
                                        name: "__init__".to_string(), 
                                        parameters: vec!["self".to_string()], 
                                        source: vec![
                                            Line::new(9, "        def __init__(self):"), 
                                            Line::new(10, "            self.width = 5"), 
                                            Line::new(11, "            self.height = 10"), 
                                        ]
                                    }, 
                                    ClassMethod {
                                        name: "get_width".to_string(), 
                                        parameters: vec!["self".to_string(), "pineapple=25".to_string()], 
                                        source: vec![
                                            Line::new(26, "        def get_width(self, pineapple=25):"),
                                            Line::new(27, "            return self.width")
                                        ]
                                    }
                                ], 
                                classes: vec![
                                    Class {
                                        name: "LowerClass".to_string(), 
                                        parent: "Shape, Banana".to_string(), 
                                        variables: vec![
                                            Assignment::new(&Line::new(15, "            LOWER_GLOB = \"LowerClass class variable\"")).unwrap(), 
                                            Assignment::new(&Line::new(16, "            SOME_OTHER_THING = \"Apple\"")).unwrap(), 
                                        ], 
                                        methods: vec![
                                            ClassMethod {
                                                name: "__init__".to_string(), 
                                                parameters: vec!["self".to_string(), "banana".to_string(), "apple".to_string()], 
                                                source: vec![
                                                    Line::new(18, "            def __init__(self, banana, apple):"),
                                                    Line::new(19, "                self.banana = banana"),
                                                    Line::new(20, "                self.apple = apple"),
                                                    Line::new(21, "                self.mango = (banana * apple) / math.sqrt(2)"),
                                                ]
                                            }, 
                                            ClassMethod {
                                                name: "pear".to_string(), 
                                                parameters: vec!["self".to_string(), "orange".to_string()], 
                                                source: vec![
                                                    Line::new(23, "            def pear(self, orange):"), 
                                                    Line::new(24, "                return self.apple * self.banana * orange")
                                                ]
                                            }
                                        ], 
                                        classes: vec![]
                                    }
                                ]
                            }
                        ]
                    }
                ] // end of classes
            }, // end of file
        ]; // end of files
        
        // Read lines from files and create File objects from them, then compare the File objects to the File objects in the vector above.
        for (filename, expected_file) in std::iter::zip(files, expected_results) {
            // Create file object from filename.
            let lines_str: Vec<String> = get_lines_for_test(filename);
            let lines: Vec<Line> = vec_str_to_vec_line(&lines_str);
            let file: File = File::new(filename, &lines);
            
            // Compare file object to expected file object.
            assert_eq!(file, expected_file);
        }
    }
    
}
