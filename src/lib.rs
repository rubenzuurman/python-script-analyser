use std::fs;
use std::io;

use regex::Regex;

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    number: usize, 
    text: String, 
}

impl Line {
    
    pub fn create(number: usize, text: &str) -> Self {
        return Line {number: number, text: text.to_string()};
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
        let mut line_space: &str = "    ";
        if self.number >= 10 {
            line_space = "   ";
        } else if self.number >= 100 {
            line_space = "  ";
        } else if self.number >= 1000 {
            line_space = " ";
        } else {
            line_space = " ";
        }
        print!("Line{}{}: {}", line_space, self.number, self.text);
        return Ok(());
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
        let re_name_params = Regex::new(r"^ *def (?<name>\w+)\((?<params>.*)\):$").unwrap();
        let re_replace_space = Regex::new(" +").unwrap();
        
        // Match regex and initialize function properties.
        let name_params_captures = re_name_params.captures(first_line);
        let mut name: String = String::from("");
        let mut parameters: Vec<String> = Vec::new();
        
        // Match regex captures and set function properties.
        match name_params_captures {
            Some(a) => {
                name = String::from(&a["name"]);
                for param in a["params"].split(",") {
                    parameters.push(String::from(re_replace_space.replace_all(param.trim(), " ")));
                }
            }, 
            None => {
                eprintln!("Invalid function definition on the first line of the source '{}'.", first_line);
                return Function {name: "name", parameters: Vec::new(), source: Vec::new()};
            }
        }
        
        // Return function object.
        return Function {name: name, parameters: parameters, source: source.to_vec()};
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

#[derive(Debug, PartialEq)]
pub struct Class {
    name: String, 
    parent: String, 
    variables: Vec<String>, 
    methods: Vec<ClassMethod>, // Methods in the class.
    source: Vec<Line>, 
}

impl Class {
    
    pub fn create(source: &Vec<Line>) -> Self {
        // Get first line of the source.
        let first_line: &str = source.get(0).unwrap().get_text();
        
        // Initialize regex for getting the class name when no parent class/a parent class is present.
        let re_name = Regex::new(r"^ *class (?<name>\w+): *$").unwrap();
        let re_name_parent = Regex::new(r"^ *class (?<name>\w+)\((?<parent>\w+)\): *$").unwrap();
        
        // Initialize class properties.
        let mut name: String = "".to_string();
        let mut parent: String = "".to_string();
        
        // Check if this class has a parent class and get name and parent.
        let name_parent_captures = re_name_parent.captures(&first_line);
        let name_captures = re_name.captures(&first_line);
        match name_parent_captures {
            Some(a) => {
                name = a["name"].to_string();
                parent = a["parent"].to_string();
            }, 
            None => {
                match name_captures {
                    Some(b) => {
                        name = b["name"].to_string();
                        parent = "".to_string();
                    }, 
                    None => {
                        eprintln!("Invalid class definition on the first line of the source '{}'.", first_line);
                        return Class {name: "name".to_string(), parent: "parent".to_string(), variables: Vec::new(), methods: Vec::new(), source: Vec::new()};
                    }
                }
            }
        }
        
        // Scan source for static variables.
        let re_class_var = Regex::new(r#"^ *(?<varname>\w+) *= *(?<varvalue>.+) *$"#).unwrap();
        let mut variables: Vec<String> = Vec::new();
        for line in source.iter() {
            let class_var_captures = re_class_var.captures(line.get_text());
            match class_var_captures {
                Some(a) => {
                    variables.push(a["varname"].to_string());
                }, 
                None => {
                    continue;
                }
            }
        }
        
        // Initialize regex patterns for detecting methods and indentation length.
        let re_start_function = Regex::new(r#"^(?<indentation> *)def \w+\(.*\): *$"#).unwrap();
        let re_get_indentation = Regex::new(r"^(?<indentation> *).*$").unwrap();
        
        // Initialize variables used to track methods.
        let mut in_function: bool = false;
        let mut current_function_indentation_length: usize = 0;
        let mut current_function_indentation_set: bool = false;
        let mut current_function_source: Vec<Line> = Vec::new();
        
        // Initialize classmethods vector.
        let mut classmethods: Vec<ClassMethod> = Vec::new();
        
        // Scan source for class methods.
        for (index, line) in source.iter().enumerate() {
            // Skip empty lines.
            if line.get_text().trim() == "" {
                continue;
            }
            
            // Get indentation length.
            let get_indentation_captures = re_get_indentation.captures(line.get_text()).unwrap();
            let indentation_length: usize = get_indentation_captures["indentation"].to_string().len();
            if in_function {
                // Check if the indentation length is set.
                if !current_function_indentation_set {
                    // Set indentation length if not.
                    current_function_indentation_length = indentation_length;
                    current_function_indentation_set = true;
                    
                    current_function_source.push(line.clone());
                } else {
                    // Check indentation length if it is set.
                    if indentation_length >= current_function_indentation_length {
                        // Line in method.
                        current_function_source.push(line.clone());
                    }  else {
                        // End of method.
                        // Create classmethod object and add to classmethods vector.
                        let classmethod: ClassMethod = ClassMethod::create(&current_function_source);
                        println!("Adding classmethod with name '{}' to class '{}'.", classmethod.get_name(), name);
                        classmethods.push(classmethod);
                        
                        // Reset method tracking variables.
                        in_function = false;
                        current_function_indentation_length = 0;
                        current_function_indentation_set = false;
                        current_function_source = Vec::new();
                    }
                }
            }
            
            let start_function_captures = re_start_function.captures(line.get_text());
            match start_function_captures {
                Some(a) => {
                    if !in_function {
                        in_function = true;
                        
                        current_function_source.push(line.clone());
                    }
                }, 
                None => {
                    continue;
                }
            }
        }
        
        // Check if a method was getting collected but the source ended.
        if in_function {
            // Create classmethod object and add to classmethods vector.
            let classmethod: ClassMethod = ClassMethod::create(&current_function_source);
            println!("Adding classmethod with name '{}' to class '{}'.", classmethod.get_name(), name);
            classmethods.push(classmethod);
        }
        
        return Class {name: name, parent: parent, variables: variables, methods: classmethods, source: source.to_vec()};
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

#[derive(Debug, PartialEq)]
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
        let re_name_params = Regex::new(r"^ *def (?<name>\w+)\((?<params>.*)\): *$").unwrap();
        let name_params_captures = re_name_params.captures(&first_line);
        
        // Get name and params from first line.
        let mut name: String = "".to_string();
        let mut params: String = "".to_string();
        match name_params_captures {
            Some(a) => {
                name = a["name"].to_string();
                params = a["params"].to_string();
            }, 
            None => {
                eprintln!("This state should not be reached(?), method definition does not match. '{}'", first_line);
                return ClassMethod {name: "name".to_string(), parameters: vec![], source: vec![]};
            }
        }
        
        // Get parameters from params string.
        let mut parameters: Vec<String> = Vec::new();
        for param in params.split(",") {
            parameters.push(String::from(param.trim()));
        }
        
        return ClassMethod {name: name, parameters: parameters, source: source.to_vec()};
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

pub fn get_file_lines(filename: &str) -> Result<Vec<String>, io::Error> {
    let mut result: Vec<String> = Vec::new();
    let contents = fs::read_to_string(filename)?;
    for line in contents.lines() {
        result.push(line.to_string());
    }
    return Ok(result);
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    fn get_lines_for_test(filename: &str) -> Vec<String> {
        return get_file_lines(filename).unwrap();
    }
    
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
        let mut lines: Vec<Line> = Vec::new();
        for (index, s) in lines_str.iter().enumerate() {
            lines.push(Line::create(index, &s));
        }
        let func: Function = Function::create(&lines);
        
        let func_name_want: String = String::from("func_name");
        let func_params_want: Vec<String> = vec!["param1".to_string(), "param2".to_string(), "param3=5".to_string(), "*args".to_string(), "**kwargs".to_string()];
        let func_want: Function = Function {name: func_name_want, parameters: func_params_want, source: lines};
        assert_eq!(func, func_want);
    }
    
    #[test]
    fn test_create_class() {
        let lines_str: Vec<String> = get_lines_for_test("test/create_class.py");
        let mut lines: Vec<Line> = Vec::new();
        for (index, s) in lines_str.iter().enumerate() {
            lines.push(Line::create(index, &s));
        }
        let class: Class = Class::create(&lines);
        
        let class_name_want: String = String::from("Rect");
        let class_parent_want: String = String::from("Shape");
        let class_variables_want: Vec<String> = vec!["STATIC_A".to_string(), "STATIC_B".to_string(), "ANOTHER_STATIC".to_string(), "MORE_STATIC".to_string()];
        let class_methods_want: Vec<ClassMethod> = vec![ClassMethod::create(&lines[4..=6].to_vec()), ClassMethod::create(&lines[11..=13].to_vec())];
        let class_source_want: Vec<Line> = lines;
        let class_want: Class = Class {name: class_name_want, parent: class_parent_want, variables: class_variables_want, methods: class_methods_want, source: class_source_want};
        assert_eq!(class, class_want);
    }
    
    #[test]
    fn test_create_classmethod() {
        let lines_str: Vec<String> = get_lines_for_test("test/create_classmethod.py");
        let mut lines: Vec<Line> = Vec::new();
        for (index, s) in lines_str.iter().enumerate() {
            lines.push(Line::create(index, &s));
        }
        let classmethod: ClassMethod = ClassMethod::create(&lines);
        
        let classmethod_name_want: String = String::from("class_method");
        let classmethod_params_want: Vec<String> = vec!["self".to_string(), "a".to_string(), "b=10".to_string(), "c=5".to_string(), 
            "d = 15".to_string(), "*args".to_string(), "**kwargs".to_string()];
        let classmethod_source_want: Vec<Line> = lines;
        let classmethod_want = ClassMethod {name: classmethod_name_want, parameters: classmethod_params_want, source: classmethod_source_want};
        assert_eq!(classmethod, classmethod_want);
    }
    
}
