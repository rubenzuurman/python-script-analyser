use std::fs;
use std::io;
use std::path::Path;
use std::ffi::OsStr;

use regex::Regex;

static PATTERN_INDENTATION: &str = r"^(?P<indentation> *).*$";
static PATTERN_IMPORT: &str = r"^[\t ]*import[\t ]+(?P<modules>[\w, \t]+)$";
static PATTERN_FROM_IMPORT: &str = r"^[\t ]*from[\t ]+(?P<module>\w+)[\t ]+import[\t ]+(?P<objects>[\w ,]+)$";
static PATTERN_GLOBAL_VARIABLE: &str = r"^(?P<varname>\w+)[\t ]*=[\t ]*.*$";
static PATTERN_FUNCTION_START: &str = r"^(?P<indentation>[\t ]*)def[\t ]+(?P<name>\w+)[\t ]*\((?P<params>.*)\)[\t ]*(->[\t ]*[\w, \[\]]+[\t ]*)?:[\t ]*$";
static PATTERN_CLASS_START: &str = r"^(?P<indentation>[\t ]*)class[\t ]+(?P<name>\w+)[\t ]*(\((?P<parent>[\w \t]*)\))?[\t ]*:[\t ]*$";
static PATTERN_CLASS_VARIABLE: &str = r"^[\t ]{INDENTATION}(?P<varname>\w+)[\t ]*=[\t ]*(?P<varvalue>.+)[\t ]*$"; // INDENTATION will be replaced with the current class indentation when this pattern is used.

#[derive(Clone, Debug)]
pub struct Line {
    number: usize, 
    text: String, 
}

impl Line {
    
    pub fn create(number: usize, text: &str) -> Self {
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

pub struct StructureTracker {
    active: bool, 
    indentation_length: usize, 
    indentation_set: bool, 
    source: Vec<Line>, 
}

impl StructureTracker {
    
    pub fn create() -> Self {
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
    
    pub fn create(filename: &str, source: &Vec<Line>) -> Self {
        // Get filename from path.
        let path = Path::new(filename);
        let name: &OsStr = path.file_stem().unwrap();
        
        // Print warning if the extension is not 'py'.
        let extension: &OsStr = path.extension().unwrap();
        if extension != OsStr::new("py") {
            eprintln!("WARNING: The input file might not be a python file (extension='{}', not 'py').", extension.to_str().unwrap());
        }
        
        // Initialize structure tracker (used for tracking functions and classes).
        let mut function_tracker: StructureTracker = StructureTracker::create();
        let mut class_tracker: StructureTracker = StructureTracker::create();
        
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
                        let function: Function = Function::create(function_tracker.get_source());
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
                        let class: Class = Class::create(class_tracker.get_source());
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
            let function: Function = Function::create(function_tracker.get_source());
            functions.push(function);
        }
        if class_tracker.is_active() {
            // End of class, create and push function.
            let class: Class = Class::create(class_tracker.get_source());
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
    
    pub fn create(source: &Vec<Line>) -> Self {
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
                    parameters.push(String::from(re_replace_space.replace_all(param.trim(), " ")));
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
    variables: Vec<String>, 
    methods: Vec<ClassMethod>, // Methods in the class.
    source: Vec<Line>, 
}

impl Class {
    
    pub fn create(source: &Vec<Line>) -> Self {
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
        let mut variables: Vec<String> = Vec::new();
        for line in source.iter() {
            let class_var_captures = re_class_var.captures(line.get_text());
            match class_var_captures {
                Some(a) => variables.push(a["varname"].to_string()), 
                None => continue
            }
        }
        
        // Initialize structure tracker (used for tracking methods).
        let mut method_tracker: StructureTracker = StructureTracker::create();
        
        // Initialize methods vector.
        let mut methods: Vec<ClassMethod> = Vec::new();
        
        // Scan source for class methods.
        for line in source.iter() {
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
                        let method: ClassMethod = ClassMethod::create(method_tracker.get_source());
                        println!("Adding classmethod with name '{}' to class '{}'.", &method.get_name(), name);
                        methods.push(method);
                        
                        // Reset tracker.
                        method_tracker.reset();
                    }
                }
            }
            
            if method_tracker.is_active() {
                continue;
            }
            
            // Initialize regex and check for method start.
            let re_function_start = Regex::new(PATTERN_FUNCTION_START).unwrap();
            let function_start_capt = re_function_start.captures(line.get_text());
            match function_start_capt {
                Some(_) => {
                    method_tracker.start();
                    method_tracker.add_line(&line);
                }, 
                None => continue
            }
        }
        
        // Check if a method was getting collected but the source ended.
        if method_tracker.is_active() {
            // Create classmethod object and add to methods vector.
            let method: ClassMethod = ClassMethod::create(method_tracker.get_source());
            println!("Adding classmethod with name '{}' to class '{}'.", &method.get_name(), name);
            methods.push(method);
        }
        
        return Class {
            name: name, 
            parent: parent, 
            variables: variables, 
            methods: methods, 
            source: source.to_vec()
        };
    }
    
    pub fn get_name(&self) -> &String {
        return &self.name;
    }
    
    pub fn get_parent(&self) -> &String {
        return &self.parent;
    }
    
    pub fn get_variables(&self) -> &Vec<String> {
        return &self.variables;
    }
    
    pub fn get_methods(&self) -> &Vec<ClassMethod> {
        return &self.methods;
    }
    
    pub fn get_source(&self) -> &Vec<Line> {
        return &self.source;
    }
    
}

impl PartialEq for Class {
    
    fn eq(&self, other: &Self) -> bool {
        return self.get_name() == other.get_name() 
            && self.get_parent() == other.get_parent() 
            && self.get_variables() == other.get_variables() 
            && self.get_methods() == other.get_methods() 
            && self.get_source() == other.get_source();
    }
    
}

#[derive(Debug)]
pub struct ClassMethod {
    name: String, 
    parameters: Vec<String>, 
    source: Vec<Line>, // Lines in the method.
}

impl ClassMethod {
    
    fn create(source: &Vec<Line>) -> Self {
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
        lines.push(Line::create(index + 1, text));
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
    
    #[test]
    fn test_create_line() {
        let test_cases: Vec<(usize, &str)> = vec![(25, "Hi there"), (100, "This is some string with w31rd characters \
            !_(*)`~|\\[]{};:'\",.<>/?!@#$%^&*()_+-=說文閩音通한국어 키보드乇乂ㄒ尺卂　ㄒ卄丨匚匚Моя семья"), (1000000000, "Big line number")];
        
        for (line_number, text) in test_cases {
            let line = Line::create(line_number, text);
            let line_want = Line {number: line_number, text: text.to_string()};
            assert_eq!(line, line_want);
        }
    }
    
    #[test]
    fn test_create_function() {
        let lines_str: Vec<String> = get_lines_for_test("test/create_function.py");
        let lines: Vec<Line> = vec_str_to_vec_line(&lines_str);
        let func: Function = Function::create(&lines);
        
        let func_name_want: String = String::from("func_name");
        let func_params_want: Vec<String> = vec!["param1".to_string(), "param2".to_string(), "param3=5".to_string(), "*args".to_string(), "**kwargs".to_string()];
        let func_want: Function = Function {name: func_name_want, parameters: func_params_want, source: lines};
        assert_eq!(func, func_want);
    }
    
    #[test]
    fn test_create_class() {
        let lines_str: Vec<String> = get_lines_for_test("test/create_class.py");
        let lines: Vec<Line> = vec_str_to_vec_line(&lines_str);
        let class: Class = Class::create(&lines);
        
        let class_name_want: String = String::from("Rect");
        let class_parent_want: String = String::from("Shape");
        let class_variables_want: Vec<String> = vec!["STATIC_A".to_string(), "STATIC_B".to_string(), "ANOTHER_STATIC".to_string(), "MORE_STATIC".to_string()];
        let class_methods_want: Vec<ClassMethod> = vec![ClassMethod::create(&lines[4..=6].to_vec()), ClassMethod::create(&lines[11..=13].to_vec())];
        let class_source_want: Vec<Line> = remove_empty_lines(lines);
        let class_want: Class = Class {name: class_name_want, parent: class_parent_want, variables: class_variables_want, methods: class_methods_want, source: class_source_want};
        
        assert_eq!(class, class_want);
    }
    
    #[test]
    fn test_create_classmethod() {
        let lines_str: Vec<String> = get_lines_for_test("test/create_classmethod.py");
        let lines: Vec<Line> = vec_str_to_vec_line(&lines_str);
        let classmethod: ClassMethod = ClassMethod::create(&lines);
        
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
        let function: Function = Function::create(&lines);
        
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
                            Line::create(13, "def random_function(param1, p2, p3, p4, p5=3):"),
                            Line::create(14, "    print(\"hihi\")"),
                            Line::create(15, "    for i in range(10):"),
                            Line::create(16, "        if i % 2 == 0:"),
                            Line::create(17, "            print(f\"number {i}\")"),
                            Line::create(18, "            if i % 3 == 0:"),
                            Line::create(19, "                print(\"Divisible by 6!\")"),
                            Line::create(20, "        else:"),
                            Line::create(21, "            print(\"Do nothing\")"),
                            Line::create(22, "            continue"),
                            Line::create(24, "    print(\"End of function\")")
                        ]
                    }
                ], 
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
                                     Line::create(29, "    def __enter__(self) -> GcLogger:"),
                                    Line::create(30, "        self.gc_start_time: float | None = None"),
                                    Line::create(31, "        self.gc_time = 0.0"),
                                    Line::create(32, "        self.gc_calls = 0"),
                                    Line::create(33, "        self.gc_collected = 0"),
                                    Line::create(34, "        self.gc_uncollectable = 0"),
                                    Line::create(35, "        gc.callbacks.append(self.gc_callback)"),
                                    Line::create(36, "        self.start_time = time.time()"),
                                    Line::create(37, "        return self")
                                ]
                            }, 
                            ClassMethod {
                                name: "gc_callback".to_string(), 
                                parameters: vec!["self".to_string(), "phase: str".to_string(), "info: Mapping[str, int]".to_string()], 
                                source: vec![
                                    Line::create(39, "    def gc_callback(self, phase: str, info: Mapping[str, int]) -> None:"),
                                    Line::create(40, "        if phase == \"start\":"),
                                    Line::create(41, "            assert self.gc_start_time is None, \"Start phase out of sequence\""),
                                    Line::create(42, "            self.gc_start_time = time.time()"),
                                    Line::create(43, "        elif phase == \"stop\":"),
                                    Line::create(44, "            assert self.gc_start_time is not None, \"Stop phase out of sequence\""),
                                    Line::create(45, "            self.gc_calls += 1"),
                                    Line::create(46, "            self.gc_time += time.time() - self.gc_start_time"),
                                    Line::create(47, "            self.gc_start_time = None"),
                                    Line::create(48, "            self.gc_collected += info[\"collected\"]"),
                                    Line::create(49, "            self.gc_uncollectable += info[\"uncollectable\"]"),
                                    Line::create(50, "        else:"),
                                    Line::create(51, "            assert False, f\"Unrecognized gc phase ({phase!r})\"")
                                ]
                            }, 
                            ClassMethod {
                                name: "__exit__".to_string(), 
                                parameters: vec!["self".to_string(), "*args: object".to_string()], 
                                source: vec![
                                        Line::create(53, "    def __exit__(self, *args: object) -> None:"),
                                        Line::create(54, "        while self.gc_callback in gc.callbacks:"),
                                        Line::create(55, "            gc.callbacks.remove(self.gc_callback)")
                                ]
                            }, 
                            ClassMethod {
                                name: "get_stats".to_string(), 
                                parameters: vec!["self".to_string()], 
                                source: vec![
                                        Line::create(57, "    def get_stats(self) -> Mapping[str, float]:"),
                                        Line::create(58, "        end_time = time.time()"),
                                        Line::create(59, "        result = {}"),
                                        Line::create(60, "        result[\"gc_time\"] = self.gc_time"),
                                        Line::create(61, "        result[\"gc_calls\"] = self.gc_calls"),
                                        Line::create(62, "        result[\"gc_collected\"] = self.gc_collected"),
                                        Line::create(63, "        result[\"gc_uncollectable\"] = self.gc_uncollectable"),
                                        Line::create(64, "        result[\"build_time\"] = end_time - self.start_time"),
                                        Line::create(65, "        return result")
                                ]
                            }
                        ], 
                        source: vec![
                            Line::create(26, "class GcLogger:"),
                            Line::create(27, "    \"\"\"Context manager to log GC stats and overall time.\"\"\""),
                            Line::create(29, "    def __enter__(self) -> GcLogger:"),
                            Line::create(30, "        self.gc_start_time: float | None = None"),
                            Line::create(31, "        self.gc_time = 0.0"),
                            Line::create(32, "        self.gc_calls = 0"),
                            Line::create(33, "        self.gc_collected = 0"),
                            Line::create(34, "        self.gc_uncollectable = 0"),
                            Line::create(35, "        gc.callbacks.append(self.gc_callback)"),
                            Line::create(36, "        self.start_time = time.time()"),
                            Line::create(37, "        return self"),
                            Line::create(39, "    def gc_callback(self, phase: str, info: Mapping[str, int]) -> None:"),
                            Line::create(40, "        if phase == \"start\":"),
                            Line::create(41, "            assert self.gc_start_time is None, \"Start phase out of sequence\""),
                            Line::create(42, "            self.gc_start_time = time.time()"),
                            Line::create(43, "        elif phase == \"stop\":"),
                            Line::create(44, "            assert self.gc_start_time is not None, \"Stop phase out of sequence\""),
                            Line::create(45, "            self.gc_calls += 1"),
                            Line::create(46, "            self.gc_time += time.time() - self.gc_start_time"),
                            Line::create(47, "            self.gc_start_time = None"),
                            Line::create(48, "            self.gc_collected += info[\"collected\"]"),
                            Line::create(49, "            self.gc_uncollectable += info[\"uncollectable\"]"),
                            Line::create(50, "        else:"),
                            Line::create(51, "            assert False, f\"Unrecognized gc phase ({phase!r})\""),
                            Line::create(53, "    def __exit__(self, *args: object) -> None:"),
                            Line::create(54, "        while self.gc_callback in gc.callbacks:"),
                            Line::create(55, "            gc.callbacks.remove(self.gc_callback)"),
                            Line::create(57, "    def get_stats(self) -> Mapping[str, float]:"),
                            Line::create(58, "        end_time = time.time()"),
                            Line::create(59, "        result = {}"),
                            Line::create(60, "        result[\"gc_time\"] = self.gc_time"),
                            Line::create(61, "        result[\"gc_calls\"] = self.gc_calls"),
                            Line::create(62, "        result[\"gc_collected\"] = self.gc_collected"),
                            Line::create(63, "        result[\"gc_uncollectable\"] = self.gc_uncollectable"),
                            Line::create(64, "        result[\"build_time\"] = end_time - self.start_time"),
                            Line::create(65, "        return result")
                        ]
                    }, // end of class
                ] // end of classes
            }, // end of file
        ]; // end of files
        
        // Read lines from files and create File objects from them, then compare the File objects to the File objects in the vector above.
        for (filename, expected_file) in std::iter::zip(files, expected_results) {
            // Create file object from filename.
            let lines_str: Vec<String> = get_lines_for_test(filename);
            let lines: Vec<Line> = vec_str_to_vec_line(&lines_str);
            let file: File = File::create(filename, &lines);
            
            // Compare file object to expected file object.
            assert_eq!(file, expected_file);
        }
    }
    
}
