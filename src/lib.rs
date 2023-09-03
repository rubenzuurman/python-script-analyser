use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::ffi::OsStr;

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use regex::Regex;

static PATTERN_INDENTATION: &str = r"^(?P<indentation>[\t ]*).*$";
static PATTERN_IMPORT: &str = r"^[\t ]*import[\t ]+(?P<modules>[\w, \t\.]+)$";
static PATTERN_FROM_IMPORT: &str = r"^[\t ]*from[\t ]+(?P<module>[\w\.]+)[\t ]+import[\t ]+(?P<objects>[\w ,\t]+)$";
static PATTERN_FUNCTION_START: &str = r"^(?P<indentation>[\t ]*)def[\t ]+(?P<name>\w+)[\t ]*\((?P<params>.*)\)[\t ]*(->[\t ]*[\w, \t\[\]]+[\t ]*)?:[\t ]*$";
static PATTERN_CLASS_START: &str = r"^(?P<indentation>[\t ]*)class[\t ]+(?P<name>\w+)[\t ]*(\((?P<parent>[\w \t\[\]\.,]*)\))?[\t ]*:[\t ]*$";
static PATTERN_CLASS_VARIABLE: &str = r"^[\t ]{INDENTATION}(?P<varname>\w+)[\t ]*(:.*)?[\t ]*=[\t ]*(?P<varvalue>.+)[\t ]*$"; // INDENTATION will be replaced with the current class indentation when this pattern is used.
static PATTERN_WHILE_LOOP: &str = r"^[\t ]*while[\t ]+(?P<condition>.*)[\t ]*:[\t ]*$";
static PATTERN_FOR_LOOP: &str = r"^[\t ]*for[\t ]+(?P<itervar>[a-zA-Z_]{1}\w*)[\t ]+in[\t ]+(?P<iterator>[a-zA-Z_]{1}.*)[\t ]*:[\t ]*$";

static PATTERN_FUNCTION_CALL_EXPRESSION: &str = r"^(?P<name>[a-zA-Z_]{1}\w*)\((?P<arguments>.*)\)$";
static PATTERN_ARRAY_DICT_ACCESS_EXPRESSION: &str = r"^(?P<name>[a-zA-Z_]{1}\w*)\[(?P<index>.+)\]$";
static PATTERN_VARIABLE_NAME_EXPRESSION: &str = r"^[a-zA-Z_]{1}\w*$";
static PATTERN_WITH_STATEMENT: &str = r"^[\t ]*with[\t ]+(?P<expression>.*)[\t ]+as[\t ]+(?P<alias>[a-zA-Z_]{1}\w+)[\t ]*:[\t ]*$";

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
        A line is an assignment if it contains exactly one equal sign (not preceded by a less than sign, greater than sign, or exclamation mark) which is not enclosed by any of the following:
            Single quotations
            Double quotations
            Normal brackets
            Square brackets
            Curly brackets
        These prefixs for the equal sign are allowed: plus sign, minus sign, slash, asterisk, percent, hat, ampersand, pipe symbol, or tilde.
        */
        let mut in_single_quotations: bool = false;
        let mut in_double_quotations: bool = false;
        let mut in_brackets_depth: i32 = 0;
        let mut in_square_brackets_depth: i32 = 0;
        let mut in_curly_brackets_depth: i32 = 0;
        
        let mut first_half: bool = true;
        let mut equals_index: usize = 0;
        for (index, c) in self.get_text().chars().enumerate() {
            match c {
                '\'' => {
                    if index == 0 {
                        in_single_quotations = !in_single_quotations;
                    } else {
                        if !(self.get_text().chars().nth(index - 1).unwrap() == '\\') {
                            in_single_quotations = !in_single_quotations;
                        }
                    }
                }, 
                '\"' => {
                    if index == 0 {
                        in_double_quotations = !in_double_quotations;
                    } else {
                        if !(self.get_text().chars().nth(index - 1).unwrap() == '\\') {
                            in_double_quotations = !in_double_quotations;
                        }
                    }
                }, 
                '(' => {
                    if !(in_single_quotations || in_double_quotations) {
                        in_brackets_depth += 1;
                    }
                }, 
                ')' => {
                    if !(in_single_quotations || in_double_quotations) {
                        if in_brackets_depth > 0 {
                            in_brackets_depth -= 1;
                        }
                    }
                }, 
                '[' => {
                    if !(in_single_quotations || in_double_quotations) {
                        in_square_brackets_depth += 1;
                    }
                }, 
                ']' => {
                    if !(in_single_quotations || in_double_quotations) {
                        if in_square_brackets_depth > 0 {
                            in_square_brackets_depth -= 1;
                        }
                    }
                }, 
                '{' => {
                    if !(in_single_quotations || in_double_quotations) {
                        in_curly_brackets_depth += 1;
                    }
                }, 
                '}' => {
                    if !(in_single_quotations || in_double_quotations) {
                        if in_curly_brackets_depth > 0 {
                            in_curly_brackets_depth -= 1;
                        }
                    }
                }, 
                '=' => {
                    // Check if this is the first character, in which case this is not an assignment.
                    if index == 0 {
                        return None;
                    }
                    
                    // Check if the previous character was not '>', '<', '!', '+', or '-'.
                    let prev_char: char = self.get_text().chars().nth(index - 1).unwrap();
                    if prev_char == '>' || prev_char == '<' || prev_char == '!' {
                        continue;
                    }
                    
                    // Check if not in quotations or brackets.
                    if !(in_single_quotations || in_double_quotations || in_brackets_depth > 0 || in_square_brackets_depth > 0 || in_curly_brackets_depth > 0) {
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
                '#' => {
                    // Check if not in quotations or brackets.
                    if !(in_single_quotations || in_double_quotations) {
                        break;
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
    
    pub fn as_string(&self, indentation_length: usize) -> String {
        // Set up indentation.
        let v: Vec<char> = vec![' '; indentation_length];
        let s: String = v.iter().collect();
        let spaces: &str = s.as_str();
        
        // Set up space before number after "Line".
        let line_space: &str = match self.number {
            n if n >= 1000 => " ", 
            n if n >= 100  => "  ", 
            n if n >= 10   => "   ", 
            n if n >= 1    => "    ", 
            _              => "    "
        };
        
        // Build string.
        return format!("{}Line{}{}: {}\n", spaces, line_space, self.get_number(), self.get_text());
    }
    
}

impl PartialEq for Line {
    
    fn eq(&self, other: &Self) -> bool {
        return self.number == other.number 
            && self.text == other.text;
    }
    
}

#[derive(Clone, Debug)]
pub struct Assignment {
    
    name: String, 
    value: String, 
    source: Line
    
}

impl Assignment {
    
    pub fn new(line: &Line) -> Option<Self> {
        // This function checks if the line contains an assignment. If it does, it results Some(Assignment), else it returns None. This Option<T> can then be matched by the caller.
        let dummy_line: Line = Line::new(1, &remove_single_line_comment_from_line(&line));
        match dummy_line.is_assignment() {
            // Return none if the line does not contain an assignment.
            None => return None, 
            // Return some if the line does contain an assignment.
            Some(equals_index) => {
                // Split line text at index.
                let var: &str = &dummy_line.get_text().as_str()[..equals_index];
                let val: &str = &dummy_line.get_text().as_str()[equals_index+1..];
                
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
                    // Check if the variable name ends with + - / * // ** % ^ & |.
                    let mut var_trim: String = var.trim().to_string();
                    let mut val_trim: String = val.trim().to_string();
                    
                    let suffixes: Vec<&str> = vec!["//", "**", "+", "-", "/", "*", "%", "^", "&", "|"];
                    
                    for suffix in suffixes {
                        if var_trim.ends_with(suffix) {
                            val_trim = format!("{} ({})", var_trim, val_trim);
                            for _ in 0..suffix.len() {
                                var_trim.pop();
                            }
                        }
                    }
                    
                    return Some(Assignment {
                        name: var_trim.trim().to_string(), 
                        value: val_trim.trim().to_string(), 
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
    
    pub fn as_string(&self, indentation_length: usize) -> String {
        // Set up indentation.
        let v: Vec<char> = vec![' '; indentation_length];
        let s: String = v.iter().collect();
        let spaces: &str = s.as_str();
        
        // Build string.
        return format!("{}Assignment({} = {})\n", spaces, self.get_name(), self.get_value());
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

pub struct MultilineCommentTracker {
    active: bool
}

impl MultilineCommentTracker {
    
    fn new() -> Self {
        return MultilineCommentTracker {
            active: false
        };
    }
    
    fn activate(&mut self) {
        self.active = true;
    }
    
    fn deactivate(&mut self) {
        self.active = false;
    }
    
    fn is_active(&self) -> bool {
        return self.active;
    }
    
    fn is_begin_of_multiline_comment(&self, line: &Line) -> bool {
        // This method is only ever called when active is false.
        // Check if this line is the start and/or end of a multiline comment.
        let is_ml_comment_start: bool = line_is_multiline_comment_start(&line);
        let is_ml_comment_end: bool = line_is_multiline_comment_end(&line);
        
        // Check if the line is start and end.
        if is_ml_comment_start && is_ml_comment_end {
            if line.get_text().matches("\"").count() >= 6 || line.get_text().matches("\'").count() >= 6 {
                return false;
            } else {
                return true;
            }
        // Check if the line is only start.
        } else if is_ml_comment_start {
            return true;
        // Check if the line is only end or none at all.
        } else {
            return false;
        }
    }
    
    fn is_end_of_multiline_comment(&self, line: &Line) -> bool {
        // This method is only ever called when active is true.
        // Check if this line is the end of a multiline comment.
        return line_is_multiline_comment_end(&line);
    }
    
}


#[derive(Clone, Debug)]
pub struct File {
    name: String, 
    imports: Vec<String>, 
    global_variables: Vec<Assignment>, 
    functions: Vec<Function>, 
    classes: Vec<Class>, 
}

impl File {
    
    pub fn new(filepath: &str, source: &Vec<Line>, writer: &mut BufWriter<Box<dyn Write>>) -> Self {
        // Get filename from path.
        let path = Path::new(filepath);
        let name: &str = match path.file_stem() {
            Some(a) => match a.to_str() {
                Some(b) => b, 
                None => {
                    write_to_writer(writer, format!("WARNING: Filename '{:?}' is not valid utf-8, leaving filename field empty.", a).as_bytes());
                    ""
                }
            }, 
            None => ""
        };
        
        // Print warning if the extension is not 'py'.
        match path.extension().and_then(OsStr::to_str) {
            Some(extension) => {
                if extension != "py" {
                    write_to_writer(writer, format!("WARNING: The input file might not be a python file (extension='{}', not 'py').\n", extension).as_bytes());
                }
            }, 
            None => {
                write_to_writer(writer, b"WARNING: The input file might not be a python file (file has no extension).\n")
            }
        }
        
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
        
        // Initialize structure tracker (used for tracking functions and classes).
        let mut function_tracker: StructureTracker = StructureTracker::new();
        let mut class_tracker: StructureTracker = StructureTracker::new();
        let mut ml_comment_tracker: MultilineCommentTracker = MultilineCommentTracker::new();
        
        // Iterate over lines and detect things.
        let mut imports: Vec<String> = Vec::new();
        let mut global_vars: Vec<Assignment> = Vec::new();
        let mut functions: Vec<Function> = Vec::new();
        let mut classes: Vec<Class> = Vec::new();
        for line in source.iter() {
            // Check if currently in a function or a class.
            let indentation_length = get_indentation_length(line);
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
                        let function: Function = Function::new(function_tracker.get_source(), writer);
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
                        let class: Class = Class::new(class_tracker.get_source(), writer);
                        classes.push(class);
                        // Reset tracker.
                        class_tracker.reset();
                    }
                }
            }
            
            if function_tracker.is_active() || class_tracker.is_active() {
                continue;
            }
            
            // Check if this line is the start or end of a multiline comment.
            if ml_comment_tracker.is_active() {
                if ml_comment_tracker.is_end_of_multiline_comment(&line) {
                    ml_comment_tracker.deactivate();
                }
            } else {
                if ml_comment_tracker.is_begin_of_multiline_comment(&line) {
                    ml_comment_tracker.activate();
                }
            }
            if ml_comment_tracker.is_active() {
                continue;
            }
            
            // Detect imports.
            match line_is_import(&line, writer) {
                Some(a) => {
                    for module in a.iter() {
                        imports.push(module.clone());
                    }
                }, 
                None => ()
            }
            
            // Detect global variables.
            if let Some(_) = line.is_assignment() {
                match Assignment::new(line) {
                    Some(a) => global_vars.push(a), 
                    None => write_to_writer(writer, format!("WARNING: '{}' should have been an assignment, but wasn't. This is not supposed to happen. (File::new())\n", line.as_string(0)).as_bytes()), 
                }
            }
            
            // Detect functions.
            if line_is_function_start(&line) {
                // Start function tracker.
                function_tracker.start();
                function_tracker.add_line(&line);
            }
            
            // Detect classes.
            if line_is_class_start(&line) {
                // Start class tracker.
                class_tracker.start();
                class_tracker.add_line(&line);
            }
        }
        
        // Check if the function tracker or class tracker is still active.
        if function_tracker.is_active() {
            // End of function, create and push function.
            let function: Function = Function::new(function_tracker.get_source(), writer);
            functions.push(function);
        }
        if class_tracker.is_active() {
            // End of class, create and push function.
            let class: Class = Class::new(class_tracker.get_source(), writer);
            classes.push(class);
        }
        
        // Return file.
        return File {
            name: name.to_string(), 
            imports: imports, 
            global_variables: global_vars, 
            functions: functions, 
            classes: classes
        };
    }
    
    pub fn scan(&self, writer: &mut BufWriter<Box<dyn Write>>) {
        // Define function to check if the scope contains a variable name.
        fn scope_contains(scope: &Vec<(usize, String)>, item: &str) -> bool {
            for var in scope {
                if var.1 == item {
                    return true;
                }
            }
            return false;
        }
        
        // Initialize current scope.
        let mut scope: Vec<(usize, String)> = vec![
            (0, "False".to_string()), 
            (0, "True".to_string()), 
        ];
        
        // Get list of imports and add them to current scope.
        for import in self.get_imports() {
            scope.push((0, import.clone()));
        }
        
        // Add function names and class names to scope.
        for function in self.get_functions() {
            scope.push((0, function.get_name().clone()));
        }
        for class in self.get_classes() {
            scope.push((0, class.get_name().clone()));
        }
        
        // Check global variables and add to current scope.
        for var in self.get_global_variables() {
            //write_to_writer(writer, format!("[Line {}] Doing `{}`=`{}`.\n", var.get_source().get_number(), var.get_name(), var.get_value()).as_bytes());
            
            // Check if the name contains a dot (meaning a function/class/variable is assigned on the value, meaning the name should not be added to the scope).
            if !var.get_name().contains(".") {
                scope.push((get_indentation_length(var.get_source()), var.get_name().clone()));
            }
            
            // The variable name contains a dot (meaning a function/class/variable is called from the value).
            let temp_result_name: Vec<String> = handle_assignment_expression(var.get_name().clone(), true, false);
            for variable in temp_result_name {
                if !scope_contains(&scope, &variable) {
                    write_to_writer(writer, format!("[Line {}] WARNING: Variable name does not exist or is out of scope '{}'.\n", var.get_source().get_number(), variable).as_bytes());
                }
            }
            
            // Check the variable value for undefined variables.
            let temp_result_value: Vec<String> = handle_assignment_expression(var.get_value().clone(), true, false);
            for variable in temp_result_value {
                if !scope_contains(&scope, &variable) {
                    write_to_writer(writer, format!("[Line {}] WARNING: Variable name does not exist or is out of scope '{}'.\n", var.get_source().get_number(), variable).as_bytes());
                }
            }
        }
        
        // Scan functions.
        for function in self.get_functions() {
            function.scan(writer, &mut scope);
        }
        
        // Scan classes.
        for class in self.get_classes() {
            class.scan(writer, &mut scope);
        }
    }
    
    pub fn get_name(&self) -> &String {
        return &self.name;
    }
    
    pub fn get_imports(&self) -> &Vec<String> {
        return &self.imports;
    }
    
    pub fn get_global_variables(&self) -> &Vec<Assignment> {
        return &self.global_variables;
    }
    
    pub fn get_functions(&self) -> &Vec<Function> {
        return &self.functions;
    }
    
    pub fn get_classes(&self) -> &Vec<Class> {
        return &self.classes;
    }
    
    pub fn as_string(&self, indentation_length: usize) -> String {
        // Set up indentation.
        let v: Vec<char> = vec![' '; indentation_length];
        let s: String = v.iter().collect();
        let spaces: &str = s.as_str();
        let spaces_extra_tab: &str = &(spaces.to_owned() + "    ");
        
        // Build string.
        let mut string: String = "".to_string();
        
        // Push name and imports.
        string.push_str(format!("{}File [\n", spaces).as_str());
        string.push_str(format!("{}name: {}\n", spaces_extra_tab, self.get_name()).as_str());
        string.push_str(format!("{}imports: {:?}\n", spaces_extra_tab, self.get_imports()).as_str());
        
        // Push global variables.
        if self.get_global_variables().len() > 0{
            string.push_str(format!("{}global variables [\n", spaces_extra_tab).as_str());
            for global_var in self.get_global_variables() {
                string.push_str(global_var.as_string(indentation_length + 8).as_str());
            }
            string.push_str(format!("{}]\n", spaces_extra_tab).as_str());
        } else {
            string.push_str(format!("{}global variables []\n", spaces_extra_tab).as_str());
        }
        
        // Push functions.
        if self.get_functions().len() > 0 {
            string.push_str(format!("{}functions [\n", spaces_extra_tab).as_str());
            for function in self.get_functions() {
                string.push_str(function.as_string(indentation_length + 8).as_str());
            }
            string.push_str(format!("{}]\n", spaces_extra_tab).as_str());
        } else {
            string.push_str(format!("{}functions []\n", spaces_extra_tab).as_str());
        }
        
        // Push classes.
        if self.get_classes().len() > 0 {
            string.push_str(format!("{}classes [\n", spaces_extra_tab).as_str());
            for class in self.get_classes() {
                string.push_str(class.as_string(indentation_length + 8).as_str());
            }
            string.push_str(format!("{}]\n", spaces_extra_tab).as_str());
        } else {
            string.push_str(format!("{}classes []\n", spaces_extra_tab).as_str());
        }
        
        string.push_str(format!("{}]\n", spaces).as_str());
        
        return string;
    }
    
}

impl PartialEq for File {
    
    fn eq(&self, other: &Self) -> bool {
        return self.get_name() == other.get_name() 
            && self.get_imports() == other.get_imports() 
            && self.get_global_variables() == other.get_global_variables() 
            && self.get_functions() == other.get_functions() 
            && self.get_classes() == other.get_classes();
    }
    
}

#[derive(Clone, Debug)]
pub struct Function {
    name: String, 
    parameters: Vec<String>, 
    functions: Vec<Function>, 
    source: Vec<Line>, 
}

impl Function {
    
    pub fn new(source: &Vec<Line>, writer: &mut BufWriter<Box<dyn Write>>) -> Self {
        // Get clone of source.
        let mut source: Vec<Line> = source.clone();
        
        // Remove empty lines.
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
        let first_line: &str = &remove_single_line_comment_from_line(source.get(0).unwrap());
        
        // Initialize regex for getting the function name and the parameters from the function definition.
        let re_function_start = Regex::new(PATTERN_FUNCTION_START).unwrap();
        
        // Match regex and initialize function properties.
        let function_start_capt = re_function_start.captures(first_line);
        let (name, params): (String, String) = match function_start_capt {
            Some(a) => (a["name"].to_string(), a["params"].to_string()), 
            None => {
                write_to_writer(writer, format!("WARNING: Invalid function definition on the first line of the source '{}'.\n", first_line).as_bytes());
                return Function::default();
            }
        };
        
        /* Get parameters from params string. When a comma is surrounded by either of the following
            '', "", (), [], {}
        skip it.
        */
        let mut in_quotations: bool = false;
        let mut in_double_quotations: bool = false;
        let mut in_brackets_depth: u32 = 0;
        let mut in_square_brackets_depth: u32 = 0;
        let mut in_curly_brackets_depth: u32 = 0;
        
        let mut parameters: Vec<String> = Vec::new();
        parameters.push("".to_string());
        for (index, c) in params.chars().enumerate() {
            let last_parameter: &String = parameters.last().unwrap();
            if last_parameter == "," {
                *parameters.last_mut().unwrap() = "".to_string();
            }
            match c {
                '\'' => {
                    if index == 0 {
                        in_quotations = !in_quotations;
                    } else {
                        if !(params.chars().nth(index - 1).unwrap() == '\\') {
                            in_quotations = !in_quotations;
                        }
                    }
                }, 
                '\"' => {
                    if index == 0 {
                        in_double_quotations = !in_double_quotations;
                    } else {
                        if !(params.chars().nth(index - 1).unwrap() == '\\') {
                            in_double_quotations = !in_double_quotations;
                        }
                    }
                }, 
                '(' => {
                    if !(in_quotations || in_double_quotations) {
                        in_brackets_depth += 1;
                    }
                }, 
                ')' => {
                    if !(in_quotations || in_double_quotations) {
                        in_brackets_depth -= 1;
                    }
                }, 
                '[' => {
                    if !(in_quotations || in_double_quotations) {
                        in_square_brackets_depth += 1;
                    }
                }, 
                ']' => {
                    if !(in_quotations || in_double_quotations) {
                        in_square_brackets_depth -= 1;
                    }
                }, 
                '{' => {
                    if !(in_quotations || in_double_quotations) {
                        in_curly_brackets_depth += 1;
                    }
                }, 
                '}' => {
                    if !(in_quotations || in_double_quotations) {
                        in_curly_brackets_depth -= 1;
                    }
                }, 
                ',' => {
                    // Check if not in quotations or brackets.
                    if !(in_quotations || in_double_quotations || in_brackets_depth > 0 || in_square_brackets_depth > 0 || in_curly_brackets_depth > 0) {
                        parameters.push("".to_string());
                    }
                }, 
                _ => ()
            }
            parameters.last_mut().unwrap().push(c);
        }
        
        // Remove all spaces not in quotations, then add spaces after commas and colons.
        for param in parameters.iter_mut() {
            let mut in_single_quotations: bool = false;
            let mut in_double_quotations: bool = false;
            
            // Remove all spaces not in quotations.
            let mut string_builder: String = "".to_string();
            for (index, c) in param.chars().enumerate() {
                match c {
                    '\"' => {
                        if index == 0 {
                            in_double_quotations = !in_double_quotations;
                        } else {
                            if !(param.chars().nth(index - 1).unwrap() == '\\') {
                                in_double_quotations = !in_double_quotations;
                            }
                        }
                        string_builder.push(c);
                    }, 
                    '\'' => {
                        if index == 0 {
                            in_single_quotations = !in_single_quotations;
                        } else {
                            if !(param.chars().nth(index - 1).unwrap() == '\\') {
                                in_single_quotations = !in_single_quotations;
                            }
                        }
                        string_builder.push(c);
                    }, 
                    ' ' => {
                        if in_single_quotations || in_double_quotations {
                            string_builder.push(c);
                        }
                    }, 
                    _ => string_builder.push(c)
                }
            }
            
            let mut in_single_quotations: bool = false;
            let mut in_double_quotations: bool = false;
            
            // Add spaces after commas and colons.
            let mut new_string_builder: String = "".to_string();
            for (index, c) in string_builder.chars().enumerate() {
                match c {
                    ',' | ':' => {
                        new_string_builder.push(c);
                        if !(in_single_quotations || in_double_quotations) {
                            new_string_builder.push(' ');
                        }
                    }, 
                    '\'' => {
                        if index == 0 {
                            in_single_quotations = !in_single_quotations;
                        } else {
                            if !(string_builder.chars().nth(index - 1).unwrap() == '\\') {
                                in_single_quotations = !in_single_quotations;
                            }
                        }
                        new_string_builder.push(c);
                    }, 
                    '\"' => {
                        if index == 0 {
                            in_double_quotations = !in_double_quotations;
                        } else {
                            if !(string_builder.chars().nth(index - 1).unwrap() == '\\') {
                                in_double_quotations = !in_double_quotations;
                            }
                        }
                        new_string_builder.push(c);
                    }, 
                    _ => new_string_builder.push(c), 
                }
            }
            
            // Update parameter.
            *param = new_string_builder.clone();
        }
        
        // Remove empty parameters.
        let mut indices_to_remove: Vec<usize> = Vec::new();
        for (index, param) in parameters.iter().enumerate() {
            if param.is_empty() {
                indices_to_remove.push(index);
            }
        }
        for index in indices_to_remove.iter().rev() {
            parameters.remove(*index);
        }
        
        // Initialize function tracker.
        let mut function_tracker: StructureTracker = StructureTracker::new();
        let mut ml_comment_tracker: MultilineCommentTracker = MultilineCommentTracker::new();
        
        // Iterate over lines and detect function start.
        let mut functions: Vec<Function> = Vec::new();
        for (index, line) in source.iter().enumerate() {
            // Check if currently in a function.
            let indentation_length = get_indentation_length(line);
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
                        let function: Function = Function::new(function_tracker.get_source(), writer);
                        functions.push(function);
                        // Reset tracker.
                        function_tracker.reset();
                    }
                }
            }
            
            if function_tracker.is_active() {
                continue;
            }
            
            // Check if this line is the start or end of a multiline comment.
            if ml_comment_tracker.is_active() {
                if ml_comment_tracker.is_end_of_multiline_comment(&line) {
                    ml_comment_tracker.deactivate();
                }
            } else {
                if ml_comment_tracker.is_begin_of_multiline_comment(&line) {
                    ml_comment_tracker.activate();
                }
            }
            if ml_comment_tracker.is_active() {
                continue;
            }
            
            // Detect function start.
            if line_is_function_start(&line) {
                // Check if this is the first line of the function.
                if index == 0 {
                    continue;
                }
                
                // Start function tracker.
                function_tracker.start();
                function_tracker.add_line(&line);
            }
        }
        
        // Check if the function tracker is still active.
        if function_tracker.is_active() {
            // End of function, create and push function.
            let function: Function = Function::new(function_tracker.get_source(), writer);
            functions.push(function);
        }
        
        // Return function object.
        return Function {
            name: name, 
            parameters: parameters, 
            functions: functions, 
            source: remove_empty_lines(source.to_vec())
        };
    }
    
    pub fn default() -> Self {
        return Function {
            name: "".to_string(), 
            parameters: Vec::new(), 
            functions: Vec::new(), 
            source: Vec::new()
        };
    }
    
    pub fn scan(&self, writer: &mut BufWriter<Box<dyn Write>>, scope: &Vec<(usize, String)>) {
        // Define function to check if the scope contains a variable name.
        fn scope_contains(scope: &Vec<(usize, String)>, item: &str) -> bool {
            for var in scope {
                if var.1 == item {
                    return true;
                }
            }
            return false;
        }
        
        // Clone scope (everything inside this function is local to this scope).
        let mut scope: Vec<(usize, String)> = scope.clone();
        
        // Get function indentation.
        let function_indentation: usize = get_indentation_length(self.get_source().get(1).unwrap());
        
        // Add function parameters to scope.
        for parameter in self.get_parameters() {
            let mut param_split: &str = parameter.split("=").next().unwrap();
            if param_split.starts_with("**") {
                param_split = &param_split[2..];
            } else if param_split.starts_with("*") {
                param_split = &param_split[1..];
            }
            scope.push((function_indentation, param_split.to_string()));
        }
        
        // Initialize regex patterns.
        let re_for_loop = Regex::new(PATTERN_FOR_LOOP).unwrap();
        let re_while_loop = Regex::new(PATTERN_WHILE_LOOP).unwrap();
        let re_with_statement = Regex::new(PATTERN_WITH_STATEMENT).unwrap();
        
        // Loop over the source.
        for (index, line) in self.get_source().iter().enumerate() {
            // Skip first line (which is the function definition).
            if index == 0 {
                continue;
            }
            
            // Remove all variables that are out of scope on the current line.
            let current_indentation: usize = get_indentation_length(&line);
            let mut indices_to_remove: Vec<usize> = Vec::new();
            for (index, (indentation, _)) in scope.iter().enumerate() {
                if indentation > &current_indentation {
                    indices_to_remove.push(index);
                }
            }
            for index in indices_to_remove.iter().rev() {
                scope.remove(*index);
            }
            
            // Check if the line is a return statement.
            if line.get_text().trim().starts_with("return ") {
                let result: Vec<String> = handle_assignment_expression(line.get_text().trim()[7..].to_string(), true, false);
                for variable in result {
                    if !scope_contains(&scope, &variable) {
                        write_to_writer(writer, format!("[Line {}] WARNING: Variable name does not exist or is out of scope '{}'.\n", line.get_number(), variable).as_bytes());
                    }
                }
            }
            
            // Check if the line is an if statement.
            if line.get_text().trim().starts_with("if ") {
                let result: Vec<String> = handle_assignment_expression(line.get_text().trim()[3..].to_string(), true, false);
                for variable in result {
                    if !scope_contains(&scope, &variable) {
                        write_to_writer(writer, format!("[Line {}] WARNING: Variable name does not exist or is out of scope '{}'.\n", line.get_number(), variable).as_bytes());
                    }
                }
            }
            if line.get_text().trim().starts_with("elif ") {
                let result: Vec<String> = handle_assignment_expression(line.get_text().trim()[5..].to_string(), true, false);
                for variable in result {
                    if !scope_contains(&scope, &variable) {
                        write_to_writer(writer, format!("[Line {}] WARNING: Variable name does not exist or is out of scope '{}'.\n", line.get_number(), variable).as_bytes());
                    }
                }
            }
            if line.get_text().trim() == "else:" {}
            
            // If the line is a for loop, define the iteration variable using the indentation of the next line, check if the iterator is defined in this scope.
            let capt_for = re_for_loop.captures(line.get_text());
            match capt_for {
                Some(a) => {
                    // Get itervar from regex.
                    let itervar: &str = &a["itervar"].trim().to_string();
                    // Check if this is the last line.
                    if index == self.get_source().len() - 1 {
                        write_to_writer(writer, format!("[Line {}] WARNING: For loop on the last line of the function '{}'.\n", line.get_number(), self.get_name()).as_bytes());
                        continue;
                    }
                    // Get indentation of the next line.
                    let next_line_indentation: usize = get_indentation_length(self.get_source().get(index + 1).unwrap());
                    // Add itervar to scope with indentation of next line.
                    scope.push((next_line_indentation, itervar.to_string()));
                    
                    // Get iterator from regex.
                    let iterator: &str = &a["iterator"].trim().to_string();
                    // Handle iterator expression.
                    let temp_result: HashMap<String, Vec<String>> = handle_assignment_right_side_single(iterator.to_string());
                    for entry in temp_result.get("check").unwrap() {
                        if !scope_contains(&scope, entry) {
                            write_to_writer(writer, format!("[Line {}] WARNING: Variable name does not exist or is out of scope '{}'.\n", line.get_number(), entry).as_bytes());
                        }
                    }
                }, 
                None => {
                    // Check if the expression is a while loop.
                    let capt_while = re_while_loop.captures(line.get_text());
                    match capt_while {
                        Some(b) => {
                            // Get condition from regex.
                            let condition: &str = &b["condition"].trim().to_string();
                            // Check if this is the last line.
                            if index == self.get_source().len() - 1 {
                                write_to_writer(writer, format!("[Line {}] WARNING: While loop on the last line of the function '{}'.\n", line.get_number(), self.get_name()).as_bytes());
                                continue;
                            }
                            // Handle condition expression.
                            let temp_result: HashMap<String, Vec<String>> = handle_assignment_right_side_single(condition.to_string());
                            for entry in temp_result.get("check").unwrap() {
                                if !scope_contains(&scope, entry) {
                                    write_to_writer(writer, format!("[Line {}] WARNING: Variable name does not exist or is out of scope '{}'.\n", line.get_number(), entry).as_bytes());
                                }
                            }
                        }, 
                        None => {
                            // Check if the expression is a with statement.
                            let capt_with = re_with_statement.captures(&line.get_text());
                            match capt_with {
                                Some(c) => {
                                    let next_line_indentation: usize = get_indentation_length(self.get_source().get(index + 1).unwrap());
                                    
                                    let expression_result: Vec<String> = handle_assignment_expression(c["expression"].to_string(), true, false);
                                    for variable in expression_result {
                                        if !scope_contains(&scope, &variable) {
                                            write_to_writer(writer, format!("[Line {}] WARNING: Variable name does not exist or is out of scope '{}'.\n", line.get_number(), variable).as_bytes());
                                        }
                                    }
                                    
                                    let alias: String = c["alias"].to_string();
                                    scope.push((next_line_indentation, alias));
                                }, 
                                None => {
                                    match Assignment::new(&line) {
                                        Some(d) => {
                                            // Get variables from assignment.
                                            let variables: HashMap<String, Vec<String>> = get_variables_from_assignment(d);
                                            
                                            // Check used variables.
                                            for variable in variables.get("check").unwrap() {
                                                if !scope_contains(&scope, variable) {
                                                    write_to_writer(writer, format!("[Line {}] WARNING: Variable name does not exist or is out of scope '{}'.\n", line.get_number(), variable).as_bytes());
                                                }
                                            }
                                            
                                            // Add new variables to scope.
                                            for variable in variables.get("new").unwrap() {
                                                scope.push((current_indentation, variable.clone()));
                                            }
                                            
                                        }, 
                                        None => {
                                            // Get variables from non-assignment.
                                            let variables: HashMap<String, Vec<String>> = handle_assignment_right_side_single(line.get_text().clone());
                                            
                                            // Check used variables.
                                            for variable in variables.get("check").unwrap() {
                                                if !scope_contains(&scope, variable) {
                                                    write_to_writer(writer, format!("[Line {}] WARNING: Variable name does not exist or is out of scope '{}'.\n", line.get_number(), variable).as_bytes());
                                                }
                                            }
                                            
                                            // Add new variables to scope.
                                            for variable in variables.get("new").unwrap() {
                                                scope.push((current_indentation, variable.clone()));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    pub fn get_name(&self) -> &String {
        return &self.name;
    }
    
    pub fn get_parameters(&self) -> &Vec<String> {
        return &self.parameters;
    }
    
    pub fn get_functions(&self) -> &Vec<Function> {
        return &self.functions;
    }
    
    pub fn get_source(&self) -> &Vec<Line> {
        return &self.source;
    }
    
    pub fn as_string(&self, indentation_length: usize) -> String {
        // Set up indentation.
        let v: Vec<char> = vec![' '; indentation_length];
        let s: String = v.iter().collect();
        let spaces: &str = s.as_str();
        let spaces_extra_tab: &str = &(spaces.to_owned() + "    ");
        
        // Build string.
        let mut string: String = "".to_string();
        
        // Push name and parameters.
        string.push_str(format!("{}Function [\n", spaces).as_str());
        string.push_str(format!("{}name: {}\n", spaces_extra_tab, self.get_name()).as_str());
        string.push_str(format!("{}parameters: {:?}\n", spaces_extra_tab, self.get_parameters()).as_str());
        
        // Push functions.
        if self.get_functions().len() > 0 {
            string.push_str(format!("{}functions [\n", spaces_extra_tab).as_str());
            for function in self.get_functions() {
                string.push_str(format!("{}", function.as_string(indentation_length + 8)).as_str());
            }
            string.push_str(format!("{}]\n", spaces_extra_tab).as_str());
        } else {
            string.push_str(format!("{}functions []\n", spaces_extra_tab).as_str());
        }
        
        // Push source.
        if self.get_source().len() > 0 {
            string.push_str(format!("{}source [\n", spaces_extra_tab).as_str());
            for line in self.get_source() {
                string.push_str(format!("{}", line.as_string(indentation_length + 8)).as_str());
            }
            string.push_str(format!("{}]\n", spaces_extra_tab).as_str());
        } else {
            string.push_str(format!("{}source []\n", spaces_extra_tab).as_str());
        }
        
        string.push_str(format!("{}]\n", spaces).as_str());
        
        return string;
    }
    
}

impl PartialEq for Function {
    
    fn eq(&self, other: &Self) -> bool {
        return self.get_name() == other.get_name() 
            && self.get_parameters() == other.get_parameters() 
            && self.get_functions() == other.get_functions() 
            && self.get_source() == other.get_source();
    }
    
}

#[derive(Clone, Debug)]
pub struct Class {
    name: String, 
    parent: String, 
    variables: Vec<Assignment>, 
    methods: Vec<Function>, 
    classes: Vec<Class>, 
}

impl Class {
    
    pub fn new(source: &Vec<Line>, writer: &mut BufWriter<Box<dyn Write>>) -> Self {
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
        let first_line: &str = &remove_single_line_comment_from_line(source.get(0).unwrap());
        
        // Initialize regex for getting the class name when no parent class/a parent class is present.
        let re_class_start = Regex::new(PATTERN_CLASS_START).unwrap();
        
        // Initialize class properties, check if this class has a parent class and get name and parent.
        let class_start_capt = re_class_start.captures(&first_line);
        let (name, parent): (String, String) = match class_start_capt {
            Some(a) => {
                let name: String = a.name("name").unwrap().as_str().to_string();
                let parent: String = a.name("parent").map(|m| m.as_str()).unwrap_or("").to_string();
                (name, parent)
            }, 
            None => {
                write_to_writer(writer, format!("WARNING: Invalid class definition on the first line of the source '{}'.\n", first_line).as_bytes());
                return Class::default();
            }
        };
        
        // Scan source for static variables.
        // Get indentation length from second line (empty lines are not present). The indentation pattern will always match.
        let second_line: &Line = source.get(1).unwrap();
        let num_spaces: usize = get_indentation_length(second_line);
        
        // Initialize structure tracker (used for tracking methods).
        let mut method_tracker: StructureTracker = StructureTracker::new();
        let mut class_tracker: StructureTracker = StructureTracker::new();
        let mut ml_comment_tracker: MultilineCommentTracker = MultilineCommentTracker::new();
        
        // Initialize methods vector.
        let mut methods: Vec<Function> = Vec::new();
        let mut classes: Vec<Class> = Vec::new();
        let mut variables: Vec<Assignment> = Vec::new();
        
        // Initialize regex objects for methods and classes.
        let re_function_start = Regex::new(PATTERN_FUNCTION_START).unwrap();
        let re_class_start = Regex::new(PATTERN_CLASS_START).unwrap();
        let re_class_var = Regex::new(PATTERN_CLASS_VARIABLE.replace("INDENTATION", format!("{}", num_spaces).as_str()).as_str()).unwrap();
        
        // Scan source for class methods.
        for (index, line) in source.iter().enumerate() {
            // Get indentation length.
            let indentation_length: usize = get_indentation_length(line);
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
                        let method: Function = Function::new(method_tracker.get_source(), writer);
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
                        let class: Class = Class::new(class_tracker.get_source(), writer);
                        classes.push(class);
                        
                        // Reset tracker.
                        class_tracker.reset();
                    }
                }
            }
            
            if method_tracker.is_active() || class_tracker.is_active() {
                continue;
            }
            
            // Check if this line is the start or end of a multiline comment.
            if ml_comment_tracker.is_active() {
                if ml_comment_tracker.is_end_of_multiline_comment(&line) {
                    ml_comment_tracker.deactivate();
                }
            } else {
                if ml_comment_tracker.is_begin_of_multiline_comment(&line) {
                    ml_comment_tracker.activate();
                }
            }
            if ml_comment_tracker.is_active() {
                continue;
            }
            
            // Check for method start.
            let line_text: String = remove_single_line_comment_from_line(&line);
            let function_start_capt = re_function_start.captures(&line_text);
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
                    let class_start_capt = re_class_start.captures(&line_text);
                    match class_start_capt {
                        Some(_) => {
                            class_tracker.start();
                            class_tracker.add_line(&line);
                        }, 
                        None => {
                            let class_var_captures = re_class_var.captures(&line_text);
                            match class_var_captures {
                                Some(_) => {
                                    match Assignment::new(line) {
                                        Some(a) => variables.push(a), 
                                        None => write_to_writer(writer, format!("WARNING: '{}' should have been an assignment, but wasn't. This is not supposed to happen. (Class::new())\n", line.as_string(0)).as_bytes()), 
                                    }
                                }, 
                                None => continue
                            }
                        }
                    }
                }
            }
        }
        
        // Check if a method or class was getting collected when the source ended.
        if method_tracker.is_active() {
            // Create classmethod object and add to methods vector.
            let method: Function = Function::new(method_tracker.get_source(), writer);
            methods.push(method);
        }
        if class_tracker.is_active() {
            // Create class object and add to classes vector.
            let class: Class = Class::new(class_tracker.get_source(), writer);
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
    
    pub fn default() -> Self {
        return Class {
            name: "".to_string(), 
            parent: "".to_string(), 
            variables: Vec::new(), 
            methods: Vec::new(), 
            classes: Vec::new()
        };
    }
    
    pub fn scan(&self, writer: &mut BufWriter<Box<dyn Write>>, scope: &Vec<(usize, String)>) {
        // Define function to check if the scope contains a variable name.
        fn scope_contains(scope: &Vec<(usize, String)>, item: &str) -> bool {
            for var in scope {
                if var.1 == item {
                    return true;
                }
            }
            return false;
        }
        
        // Clone scope (everything inside this class is local to this scope).
        let mut scope: Vec<(usize, String)> = scope.clone();
        
        // Add 'self' to scope.
        scope.push((0, "self".to_string()));
        
        // Get class indentation.
        // TODO: Get class indentation from the first variable or function, depending on which exists.
        //let class_indentation: usize = get_indentation_length(self.get_variables().get(0).unwrap().get_source());
        let class_indentation: usize = get_indentation_length(self.get_methods().get(0).unwrap().get_source().get(0).unwrap());
        
        // Add class variables to scope.
        for class_var in self.get_variables() {
            let variables: HashMap<String, Vec<String>> = get_variables_from_assignment(class_var.clone());
            
            // Check used variables.
            for variable in variables.get("check").unwrap() {
                if !scope_contains(&scope, variable) {
                    write_to_writer(writer, format!("[Line {}] WARNING: Variable name does not exist or is out of scope '{}'.\n", class_var.get_source().get_number(), variable).as_bytes());
                }
            }
            
            // Add new variables to scope.
            for variable in variables.get("new").unwrap() {
                let real_variable: &str = &format!("{}.{}", self.get_name(), variable);
                scope.push((class_indentation, real_variable.to_string()));
            }
        }
        
        for method in self.get_methods() {
            method.scan(writer, &mut scope);
        }
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
    
    pub fn get_methods(&self) -> &Vec<Function> {
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
        let indentation: usize = get_indentation_length(lines.get(0).unwrap()) - 4;
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
    
    pub fn as_string(&self, indentation_length: usize) -> String {
        // Set up indentation.
        let v: Vec<char> = vec![' '; indentation_length];
        let s: String = v.iter().collect();
        let spaces: &str = s.as_str();
        let spaces_extra_tab: &str = &(spaces.to_owned() + "    ");
                
        // Build string.
        let mut string: String = "".to_string();
        
        // Push name and parent.
        string.push_str(format!("{}Class [\n", spaces).as_str());
        string.push_str(format!("{}name: {}\n", spaces_extra_tab, self.get_name()).as_str());
        string.push_str(format!("{}parent: {}\n", spaces_extra_tab, self.get_parent()).as_str());
        
        // Push variables.
        if self.get_variables().len() > 0 {
            string.push_str(format!("{}variables [\n", spaces_extra_tab).as_str());
            for assignment in self.get_variables() {
                string.push_str(assignment.as_string(indentation_length + 8).as_str());
            }
            string.push_str(format!("{}]\n", spaces_extra_tab).as_str());
        } else {
            string.push_str(format!("{}variables []\n", spaces_extra_tab).as_str());
        }
        
        // Push methods.
        if self.get_methods().len() > 0 {
            string.push_str(format!("{}methods [\n", spaces_extra_tab).as_str());
            for method in self.get_methods() {
                string.push_str(method.as_string(indentation_length + 8).as_str());
            }
            string.push_str(format!("{}]\n", spaces_extra_tab).as_str());
        } else {
            string.push_str(format!("{}methods []\n", spaces_extra_tab).as_str());
        }
        
        // Push classes.
        if self.get_classes().len() > 0 {
            string.push_str(format!("{}classes [\n", spaces_extra_tab).as_str());
            for class in self.get_classes() {
                string.push_str(class.as_string(indentation_length + 8).as_str());
            }
            string.push_str(format!("{}]\n", spaces_extra_tab).as_str());
        } else {
            string.push_str(format!("{}classes []\n", spaces_extra_tab).as_str());
        }
        
        string.push_str(format!("{}]\n", spaces).as_str());
        
        return string;
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

fn get_indentation_length(line: &Line) -> usize {
    // Initialize regex and capture.
    let re_indentation = Regex::new(PATTERN_INDENTATION).unwrap();
    let indentation_capt = re_indentation.captures(line.get_text());
    
    // Return indentation length.
    return indentation_capt.unwrap()["indentation"].to_string().len();
}

fn line_is_import(line: &Line, writer: &mut BufWriter<Box<dyn Write>>) -> Option<Vec<String>> {
    // Initialize regex.
    let re_import = Regex::new(PATTERN_IMPORT).unwrap();
    let re_from_import = Regex::new(PATTERN_FROM_IMPORT).unwrap();
    
    // Check if the line matches any of the regexes.
    let line_text: String = remove_single_line_comment_from_line(&line);
    let import_capt = re_import.captures(&line_text);
    let from_import_capt = re_from_import.captures(&line_text);
    
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
                    write_to_writer(writer, format!("WARNING: Line {}: Import cannot contain spaces '{}' (specifically '{}').\n", line.get_number(), line.get_text(), module).as_bytes());
                    indices_to_remove.push(index);
                } else if module.trim().is_empty() {
                    indices_to_remove.push(index);
                }
            }
            for index in indices_to_remove.iter().rev() {
                modules_vec.remove(*index);
            }
            
            // Return none if no modules are left.
            match modules_vec.len() {
                0 => return None, 
                _ => return Some(modules_vec), 
            }
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
                            write_to_writer(writer, format!("WARNING: Line {}: Import cannot contain spaces '{}' (specifically '{}').\n", line.get_number(), line.get_text(), object).as_bytes());
                            indices_to_remove.push(index);
                        } else if object.trim().is_empty() {
                            indices_to_remove.push(index);
                        }
                    }
                    for index in indices_to_remove.iter().rev() {
                        objects_vec.remove(*index);
                    }
                    
                    // Return none if no objects are left.
                    match objects_vec.len() {
                        0 => return None, 
                        _ => return Some(objects_vec), 
                    }
                }, 
                None => return None
            }
        }
    }
}

fn line_is_function_start(line: &Line) -> bool {
    // Initialize and match regex.
    let re_function_definition = Regex::new(PATTERN_FUNCTION_START).unwrap();
    let line_text: String = remove_single_line_comment_from_line(&line);
    let function_definition_capt = re_function_definition.captures(&line_text);
    
    match function_definition_capt {
        Some(_) => return true, 
        None => return false
    }
}

fn line_is_class_start(line: &Line) -> bool {
    // Initialize and match regex.
    let re_class_definition = Regex::new(PATTERN_CLASS_START).unwrap();
    let line_text: String = remove_single_line_comment_from_line(line);
    let class_definition_capt = re_class_definition.captures(&line_text);
    
    match class_definition_capt {
        Some(_) => return true, 
        None => return false
    }
}

fn remove_single_line_comment_from_line(line: &Line) -> String {
    // Detect location of first hashtag not in quotations.
    let mut in_single_quotations: bool = false;
    let mut in_double_quotations: bool = false;
    
    // Loop over characters in the line.
    let mut result: String = "".to_string();
    for (index, c) in line.get_text().chars().enumerate() {
        match c {
            '\'' => {
                if !in_double_quotations {
                    if index == 0 {
                        in_single_quotations = !in_single_quotations;
                    } else if index == 1 {
                        let prev_char: char = line.get_text().chars().nth(index - 1).unwrap();
                        if !(prev_char == '\\') {
                            in_single_quotations = !in_single_quotations;
                        }
                    } else {
                        // Check if the last two characters were also single quotations, indicating the start or end of a multiline comment.
                        let prev_char: char = line.get_text().chars().nth(index - 1).unwrap();
                        let prev_prev_char: char = line.get_text().chars().nth(index - 2).unwrap();
                        if !(prev_char == '\'' && prev_prev_char == '\'') {
                            if !(prev_char == '\\') {
                                in_single_quotations = !in_single_quotations;
                            }
                        }
                    }
                }
            }, 
            '\"' => {
                if !in_single_quotations {
                    if index == 0 {
                        in_double_quotations = !in_double_quotations;
                    } else if index == 1 {
                        let prev_char: char = line.get_text().chars().nth(index - 1).unwrap();
                        if !(prev_char == '\\') {
                            in_double_quotations = !in_double_quotations;
                        }
                    } else {
                        // Check if the last two characters were also double quotations, indicating the start or end of a multiline comment.
                        let prev_char: char = line.get_text().chars().nth(index - 1).unwrap();
                        let prev_prev_char: char = line.get_text().chars().nth(index - 2).unwrap();
                        if !(prev_char == '\"' && prev_prev_char == '\"') {
                            if !(prev_char == '\\') {
                                in_double_quotations = !in_double_quotations;
                            }
                        }
                    }
                }
            }, 
            '#' => {
                if !(in_single_quotations || in_double_quotations) {
                    return result;
                }
            }, 
            _ => ()
        }
        result.push(c);
    }
    
    return result;
}

fn line_is_multiline_comment_start(line: &Line) -> bool {
    return line.get_text().trim_start().starts_with("\"\"\"") 
        || line.get_text().trim_start().starts_with("\'\'\'");
}

fn line_is_multiline_comment_end(line: &Line) -> bool {
    // This function is only ever called if a multiline comment start was already detected. This means that, if this is the end of the multiline comment, it either ends with """/''' or ends with """/''' followed by some number of whitespaces and then a comment.
    // Get line text and line text without optional comment.
    let text_raw: String = line.get_text().to_string();
    let text_no_comment: String = remove_single_line_comment_from_line(&line);
    
    // Check if the line text ends in quotations or the line text without optional comment ends in quotations.
    let condition1: bool = text_raw.trim_end().ends_with("\"\"\"") 
        || text_raw.trim_end().ends_with("\'\'\'");
    let condition2: bool = text_no_comment.trim_end().ends_with("\"\"\"") 
        || text_no_comment.trim_end().ends_with("\'\'\'");
    
    return condition1 || condition2;
}

fn get_variables_from_assignment(assignment: Assignment) -> HashMap<String, Vec<String>> {
    let left_side: HashMap<String, Vec<String>> = handle_assignment_left_side(assignment.get_name().clone());
    let right_side: HashMap<String, Vec<String>> = handle_assignment_right_side(assignment.get_value().clone());
    
    let mut result: HashMap<String, Vec<String>> = HashMap::new();
    
    for (key, value) in left_side {
        match result.entry(key) {
            Entry::Vacant(e) => {
                e.insert(value);
            }, 
            Entry::Occupied(mut e) => {
                for entry in value {
                    if e.get().contains(&entry) {
                        continue;
                    }
                    e.get_mut().push(entry);
                }
            }
        }
    }
    for (key, value) in right_side {
        match result.entry(key) {
            Entry::Vacant(e) => {
                e.insert(value);
            }, 
            Entry::Occupied(mut e) => {
                for entry in value {
                    if e.get().contains(&entry) {
                        continue;
                    }
                    e.get_mut().push(entry);
                }
            }
        }
    }
    
    // Remove elements from 'new' that are also in 'check'.
    let mut indices_to_remove_from_new: Vec<usize> = Vec::new();
    for (index, value) in result.get("new").unwrap().iter().enumerate() {
        if result.get("check").unwrap().contains(&value) {
            indices_to_remove_from_new.push(index);
        }
    }
    for index in indices_to_remove_from_new.iter().rev() {
        result.get_mut("new").unwrap().remove(*index);
    }
    
    return result;
}

fn handle_assignment_left_side(name: String) -> HashMap<String, Vec<String>> {
    // Left side cannot contain strings.
    let mut result: HashMap<String, Vec<String>> = HashMap::new();
    for element in split_by_char(name, ',') {
        for (key, value) in handle_assignment_left_side_single(element.trim().to_string()) {
            match result.entry(key) {
                Entry::Vacant(e) => {
                    e.insert(value);
                }, 
                Entry::Occupied(mut e) => {
                    for entry in value {
                        if !e.get().contains(&entry) {
                            e.get_mut().push(entry);
                        }
                    }
                }
            }
        }
    }
    
    return result;
}

pub fn handle_assignment_right_side(string: String) -> HashMap<String, Vec<String>> {
    // Right side can contain strings.
    let mut result: HashMap<String, Vec<String>> = HashMap::new();
    for element in split_by_char(string, ',') {
        for (key, value) in handle_assignment_right_side_single(element.trim().to_string()) {
            match result.entry(key) {
                Entry::Vacant(e) => {
                    e.insert(value);
                }, 
                Entry::Occupied(mut e) => {
                    for entry in value {
                        if !e.get().contains(&entry) {
                            e.get_mut().push(entry);
                        }
                    }
                }
            }
        }
    }
    
    return result;
}

pub fn handle_assignment_left_side_single(element: String) -> HashMap<String, Vec<String>> {
    let mut result: HashMap<String, Vec<String>> = HashMap::new();
    result.insert("check".to_string(), Vec::new());
    result.insert("new".to_string(), Vec::new());
    
    // Check if element is a variable name.
    let re_variable_name = Regex::new(PATTERN_VARIABLE_NAME_EXPRESSION).unwrap();
    let capt = re_variable_name.captures(&element);
    if let Some(_) = capt {
        match result.entry("new".to_string()) {
            Entry::Vacant(e) => {
                e.insert(vec![element]);
            }, 
            Entry::Occupied(mut e) => {
                e.get_mut().push(element);
            }
        }
        return result;
    }
    
    // Not a variable name, can contain dots or is an array assignment/function call, else it's not valid (I THINK).
    let split_by_dot = split_by_char(element, '.');
    for (index, subelement) in split_by_dot.iter().enumerate() {
        let value: Vec<String> = handle_assignment_expression(subelement.clone(), index == 0, index == split_by_dot.len() - 1 && index != 0);
        if index != 0 && value.len() == 1 {
            if value.get(0).unwrap() == subelement {
                continue;
            }
        }
        for string in value {
            if !result.get("check").unwrap().contains(&string) {
                result.entry("check".to_string()).or_insert(Vec::new()).push(string);
            }
        }
    }
    return result;
}

fn handle_assignment_right_side_single(element: String) -> HashMap<String, Vec<String>> {
    let mut result: HashMap<String, Vec<String>> = HashMap::new();
    result.insert("check".to_string(), Vec::new());
    result.insert("new".to_string(), Vec::new());
    
    // Add variables used to 'check' vector.
    for variable in handle_assignment_expression(element.trim().to_string(), true, false) {
        match result.entry("check".to_string()) {
            Entry::Vacant(e) => {
                e.insert(vec![variable]);
            }, 
            Entry::Occupied(mut e) => {
                if !e.get().contains(&variable) {
                    e.get_mut().push(variable);
                }
            }
        }
    }
    
    return result;
}

pub fn handle_assignment_expression(element: String, add_array_access_name: bool, last_element_in_split_by_dot: bool) -> Vec<String> {
    // The add_array_access_name flag specifies whether or not to add the name of an array access when encounted. This is used in situations where a dot is present.
    let mut result: Vec<String> = Vec::new();
    
    // Printing this is very handy for debugging.
    //println!("Handling {}", element);
    
    // Check if the string is empty.
    if element.trim().is_empty() {
        return result;
    }
    
    // Check if the string is a comment.
    if element.trim().starts_with("#") {
        return result;
    }
    
    // Replace all sequences of spaces with a single space.
    let mut in_single_quotations: bool = false;
    let mut in_double_quotations: bool = false;
    let mut space_added: bool = false;
    
    let mut string_single_spaces: String = String::from("");
    for c in element.trim().chars() {
        match c {
            '\'' => {
                if !in_double_quotations {
                    in_single_quotations = !in_single_quotations;
                }
                string_single_spaces.push(c);
                space_added = false;
            }, 
            '\"' => {
                if !in_single_quotations {
                    in_double_quotations = !in_double_quotations;
                }
                string_single_spaces.push(c);
                space_added = false;
            }, 
            ' ' => {
                if in_single_quotations || in_double_quotations {
                    if !space_added {
                        string_single_spaces.push(c);
                        space_added = true;
                    }
                }
            }, 
            _ => {
                string_single_spaces.push(c);
                space_added = false;
            }
        }
    }
    
    // Remove spaces not in quotations.
    let mut in_single_quotations: bool = false;
    let mut in_double_quotations: bool = false;
    
    let mut string_no_spaces: String = String::from("");
    for (index, c) in element.chars().enumerate() {
        match c {
            '\'' => {
                if !in_double_quotations {
                    in_single_quotations = !in_single_quotations;
                }
                string_no_spaces.push(c);
            }, 
            '\"' => {
                if !in_single_quotations {
                    in_double_quotations = !in_double_quotations;
                }
                string_no_spaces.push(c);
            }, 
            ' ' => {
                if !(in_single_quotations || in_double_quotations) {
                    // Check if the previous and next character are not both \w characters. If they are, do not remove the space.
                    if index == 0 {
                        continue;
                    }
                    
                    let prev_char: Option<char> = element.chars().nth(index - 1);
                    let next_char: Option<char> = element.chars().nth(index + 1);
                    
                    match prev_char {
                        Some(a) => {
                            match next_char {
                                Some(b) => {
                                    let re = Regex::new(r"^\w$").unwrap();
                                    let a_str = &a.to_string();
                                    let b_str = &b.to_string();
                                    let capt_a = re.captures(a_str);
                                    let capt_b = re.captures(b_str);
                                    
                                    match capt_a {
                                        Some(_) => {
                                            match capt_b {
                                                Some(_) => {
                                                    string_no_spaces.push(c);
                                                }, 
                                                None => ()
                                            }
                                        }, 
                                        None => ()
                                    }
                                }, 
                                None => ()
                            }
                        }, 
                        None => ()
                    }
                } else {
                    string_no_spaces.push(c);
                }
            }, 
            _ => {
                string_no_spaces.push(c);
            }
        }
    }
    
    // Check if the expression is enclosed in normal or square brackets.
    if is_enclosed_in_brackets(string_no_spaces.clone()) {
        //println!("{} is enclosed in brackets.", string_no_spaces);
        let mut chars = string_no_spaces.chars();
        chars.next();
        chars.next_back();
        for string in split_by_char(chars.as_str().to_string(), ',') {
            let temp_result: Vec<String> = handle_assignment_expression(string, true, false);
            for entry in temp_result {
                if result.contains(&entry) {
                    continue;
                }
                result.push(entry);
            }
            
        }
        return result;
    }
    
    // Check if the expression is a string.
    if is_string_literal(string_no_spaces.clone()) {
        return result;
    }
    
    // Check if the expression is a function call.
    if is_function_call(string_no_spaces.clone()) {
        //println!("{} is a function call.", string_no_spaces);
        let re_function_call = Regex::new(PATTERN_FUNCTION_CALL_EXPRESSION).unwrap();
        let capt = re_function_call.captures(&string_no_spaces).unwrap(); // Can unwrap here because the regex is checked by is_function_call().
        
        let arguments: String = capt["arguments"].to_string();
        for argument in split_by_char(arguments, ',') {
            let argument_result: Vec<String> = handle_assignment_expression(argument, true, false);
            for entry in argument_result {
                if result.contains(&entry) {
                    continue;
                }
                result.push(entry);
            }
        }
        return result;
    }
    
    // Check if the expression is a variable name.
    let re_variable_name = Regex::new(PATTERN_VARIABLE_NAME_EXPRESSION).unwrap();
    let capt = re_variable_name.captures(&string_no_spaces);
    if let Some(_) = capt {
        if add_array_access_name {
            result.push(string_no_spaces);
        }
        return result;
    }
    
    // Check if the expression is an array access.
    if is_array_access(string_no_spaces.clone()) {
        //println!("{} is an array access.", string_no_spaces);
        let re_array_access = Regex::new(PATTERN_ARRAY_DICT_ACCESS_EXPRESSION).unwrap();
        let capt = re_array_access.captures(&string_no_spaces).unwrap(); // Can unwrap here because the regex is checked by is_array_access().
        
        if add_array_access_name {
            let name: String = capt["name"].to_string();
            result.push(name);
        }
        
        let index: String = capt["index"].to_string();
        let temp_result: Vec<String> = handle_assignment_expression(index, true, false);
        for entry in temp_result {
            if result.contains(&entry) {
                continue;
            }
            result.push(entry);
        }
        
        return result;
    }
    
    // Check if the expression is accessing a variable of an object (e.g. object.property).
    let split_by_dot: Vec<String> = split_by_char(string_no_spaces.clone(), '.');
    let string_contains_arithmetic: bool = contains_arithmetic_symbols_not_enclosed(string_no_spaces.clone());
    if (split_by_dot.get(0).unwrap() != &string_no_spaces || split_by_dot.len() > 1) && !string_contains_arithmetic {
        for (index, part) in split_by_dot.iter().enumerate() {
            let temp_result: Vec<String> = handle_assignment_expression(part.clone(), index == 0, index == split_by_dot.len() - 1 && index != 0);
            for entry in temp_result {
                if result.contains(&entry) {
                    continue;
                }
                result.push(entry);
            }
        }
        return result;
    }
    
    // Split string by arithmetic characters, if not in quotations, and not in brackets/square brackets/curly brackets.
    let mut in_single_quotations: bool = false;
    let mut in_double_quotations: bool = false;
    let mut bracket_depth:        i32  = 0;
    let mut square_bracket_depth: i32  = 0;
    let mut curly_bracket_depth:  i32  = 0;
    let mut chars_to_skip: u32         = 0;
    
    let mut parts: Vec<String> = Vec::new();
    let mut current_string: String = String::from("");
    for (index, c) in string_no_spaces.chars().enumerate() {
        if chars_to_skip > 0 {
            chars_to_skip -= 1;
            continue;
        }
        match c {
            '\'' => {
                if !in_double_quotations {
                    in_single_quotations = !in_single_quotations;
                }
                current_string.push(c);
            }, 
            '\"' => {
                if !in_single_quotations {
                    in_double_quotations = !in_double_quotations;
                }
                current_string.push(c);
            }, 
            '(' => {
                if !(in_single_quotations || in_double_quotations) {
                    if square_bracket_depth == 0 && curly_bracket_depth == 0 {
                        bracket_depth += 1;
                    }
                }
                current_string.push(c);
            }, 
            ')' => {
                if !(in_single_quotations || in_double_quotations) {
                    if square_bracket_depth == 0 && curly_bracket_depth == 0 {
                        if bracket_depth > 0 {
                            bracket_depth -= 1;
                        }
                    }
                }
                current_string.push(c);
            }, 
            '[' => {
                if !(in_single_quotations || in_double_quotations) {
                    if bracket_depth == 0 && curly_bracket_depth == 0 {
                        square_bracket_depth += 1;
                    }
                }
                current_string.push(c);
            }, 
            ']' => {
                if !(in_single_quotations || in_double_quotations) {
                    if bracket_depth == 0 && curly_bracket_depth == 0 {
                        if square_bracket_depth > 0 {
                            square_bracket_depth -= 1;
                        }
                    }
                }
                current_string.push(c);
            }, 
            '{' => {
                if !(in_single_quotations || in_double_quotations) {
                    if bracket_depth == 0 && square_bracket_depth == 0 {
                        curly_bracket_depth += 1;
                    }
                }
                current_string.push(c);
            }, 
            '}' => {
                if !(in_single_quotations || in_double_quotations) {
                    if bracket_depth == 0 && square_bracket_depth == 0 {
                        if curly_bracket_depth > 0 {
                            curly_bracket_depth -= 1;
                        }
                    }
                }
                current_string.push(c);
            }, 
            '+'|'-'|'%'|'^'|'&'|'|' => {
                if !(in_single_quotations || in_double_quotations || bracket_depth > 0 || square_bracket_depth > 0 || curly_bracket_depth > 0) {
                    if !parts.contains(&current_string) {
                        parts.push(current_string);
                    }
                    current_string = "".to_string();
                } else {
                    current_string.push(c);
                }
            }, 
            '*'|'/' => {
                // Check if next character is * or /.
                if !(in_single_quotations || in_double_quotations || bracket_depth > 0 || square_bracket_depth > 0 || curly_bracket_depth > 0) {
                    if index < string_no_spaces.len() - 1 {
                        let next_char: char = string_no_spaces.chars().nth(index + 1).unwrap();
                        if next_char == '*' || next_char == '/' {
                            chars_to_skip += 1;
                        }
                        if !parts.contains(&current_string) {
                            parts.push(current_string);
                        }
                        current_string = "".to_string();
                    }
                    if !parts.contains(&current_string) {
                        parts.push(current_string);
                    }
                    current_string = "".to_string();
                } else {
                    current_string.push(c);
                }
            }, 
            '<'|'>'|'!'|'=' => {
                // Check if the next character is '='.
                if !(in_single_quotations || in_double_quotations || bracket_depth > 0 || square_bracket_depth > 0 || curly_bracket_depth > 0) {
                    if !parts.contains(&current_string) {
                        parts.push(current_string);
                    }
                    current_string = "".to_string();
                    
                    let next_char: Option<char> = string_no_spaces.chars().nth(index + 1);
                    match next_char {
                        Some(a) => {
                            if a == '=' {
                                chars_to_skip += 1;
                            }
                        }, 
                        None => ()
                    }
                } else {
                    current_string.push(c);
                }
            }, 
            'a' => {
                // Check if the next characters are 'nd' and the 'and' is surrounded by non-word characters.
                if !(in_single_quotations || in_double_quotations || bracket_depth > 0 || square_bracket_depth > 0 || curly_bracket_depth > 0) {
                    let mut prev_char: Option<char> = Some('?');
                    if index > 0 {
                        prev_char = string_no_spaces.chars().nth(index - 1);
                    }
                    let next_char: Option<char> = string_no_spaces.chars().nth(index + 1);
                    let next_next_char: Option<char> = string_no_spaces.chars().nth(index + 2);
                    let next_next_next_char: Option<char> = string_no_spaces.chars().nth(index + 3);
                    let re_not_word_char = Regex::new(r"^\W$").unwrap();
                    match prev_char {
                        Some(pc) => {
                            let pc_string: String = pc.to_string();
                            let pc_not_word_char = re_not_word_char.captures(&pc_string);
                            match next_char {
                                Some(nc) => {
                                    match next_next_char {
                                        Some(nnc) => {
                                            if nc == 'n' && nnc == 'd' {
                                                match next_next_next_char {
                                                    Some(nnnc) => {
                                                        let nnnc_string: String = nnnc.to_string();
                                                        let nnnc_not_word_char = re_not_word_char.captures(&nnnc_string);
                                                        if let Some(_) = pc_not_word_char {
                                                            if let Some(_) = nnnc_not_word_char {
                                                                if !parts.contains(&current_string) {
                                                                    parts.push(current_string);
                                                                }
                                                                current_string = "".to_string();
                                                                chars_to_skip += 2;
                                                            } else {
                                                                current_string.push(c);
                                                            }
                                                        } else {
                                                            current_string.push(c);
                                                        }
                                                    }, 
                                                    None => {
                                                        current_string.push(c);
                                                    }
                                                }
                                            } else {
                                                current_string.push(c);
                                            }
                                        }, 
                                        None => {
                                            current_string.push(c);
                                        }
                                    }
                                }, 
                                None => {
                                    current_string.push(c);
                                }
                            }
                        }, 
                        None => {
                            current_string.push(c);
                        }
                    }
                } else {
                    current_string.push(c);
                }
            }, 
            'o' => {
                // Check if the next character is 'r' and the 'or' is surrounded by non-word characters.
                if !(in_single_quotations || in_double_quotations || bracket_depth > 0 || square_bracket_depth > 0 || curly_bracket_depth > 0) {
                    let mut prev_char: Option<char> = Some('?');
                    if index > 0 {
                        prev_char = string_no_spaces.chars().nth(index - 1);
                    }
                    let next_char: Option<char> = string_no_spaces.chars().nth(index + 1);
                    let next_next_char: Option<char> = string_no_spaces.chars().nth(index + 2);
                    let re_not_word_char = Regex::new(r"^\W$").unwrap();
                    match prev_char {
                        Some(pc) => {
                            let pc_string: String = pc.to_string();
                            let pc_not_word_char = re_not_word_char.captures(&pc_string);
                            match next_char {
                                Some(nc) => {
                                    if nc == 'r' {
                                        match next_next_char {
                                            Some(nnc) => {
                                                let nnc_string: String = nnc.to_string();
                                                let nnc_not_word_char = re_not_word_char.captures(&nnc_string);
                                                if let Some(_) = pc_not_word_char {
                                                    if let Some(_) = nnc_not_word_char {
                                                        if !parts.contains(&current_string) {
                                                            parts.push(current_string);
                                                        }
                                                        current_string = "".to_string();
                                                        chars_to_skip += 1;
                                                    } else {
                                                        current_string.push(c);
                                                    }
                                                } else {
                                                    current_string.push(c);
                                                }
                                            }, 
                                            None => {
                                                current_string.push(c);
                                            }
                                        }
                                    } else {
                                        current_string.push(c);
                                    }
                                }, 
                                None => {
                                    current_string.push(c);
                                }
                            }
                        }, 
                        None => {
                            current_string.push(c);
                        }
                    }
                } else {
                    current_string.push(c);
                }
            }, 
            _ => {
                current_string.push(c);
            }
        }
    }
    if parts.len() > 0 {
        if !parts.contains(&current_string) {
            parts.push(current_string);
        }
    }
    
    // Handle every part as a separate expression.
    for (index, part) in parts.iter().enumerate() {
        if part == "" {
            continue;
        }
        
        if result.contains(&part) {
            continue;
        }
        
        if last_element_in_split_by_dot {
            // TODO: Investigate if 'index == 0' is needed.
            if index == 0 {
                continue;
            }
        }
        
        let mut part_result: Vec<String> = handle_assignment_expression(part.clone(), true, false);
        let mut indices_to_remove: Vec<usize> = Vec::new();
        for (index, entry) in part_result.iter().enumerate() {
            if result.contains(entry) {
                indices_to_remove.push(index);
            }
        }
        for index in indices_to_remove.iter().rev() {
            part_result.remove(*index);
        }
        result.append(&mut part_result);
    }
    
    return result;
}

fn is_enclosed_in_brackets(string: String) -> bool {
    if string.starts_with("(") && string.ends_with(")") {
        let mut bracket_level: i32 = 0;
        let mut in_single_quotations: bool = false;
        let mut in_double_quotations: bool = false;
        for (index, c) in string.chars().enumerate() {
            match c {
                '\'' => {
                    if !in_double_quotations {
                        in_single_quotations = !in_single_quotations;
                    }
                }, 
                '\"' => {
                    if !in_single_quotations {
                        in_double_quotations = !in_double_quotations;
                    }
                }, 
                '(' => {
                    if !(in_single_quotations || in_double_quotations) {
                        bracket_level += 1;
                        if index != 0 && bracket_level == 1 {
                            return false;
                        }
                    }
                }, 
                ')' => {
                    if !(in_single_quotations || in_double_quotations) {
                        bracket_level -= 1;
                    }
                }, 
                _ => ()
            }
        }
        return bracket_level == 0;
    } else if string.starts_with("[") && string.ends_with("]") {
        let mut bracket_level: i32 = 0;
        let mut in_single_quotations: bool = false;
        let mut in_double_quotations: bool = false;
        for (index, c) in string.chars().enumerate() {
            match c {
                '\'' => {
                    if !in_double_quotations {
                        in_single_quotations = !in_single_quotations;
                    }
                }, 
                '\"' => {
                    if !in_single_quotations {
                        in_double_quotations = !in_double_quotations;
                    }
                }, 
                '[' => {
                    if !(in_single_quotations || in_double_quotations) {
                        bracket_level += 1;
                        if index != 0 && bracket_level == 1 {
                            return false;
                        }
                    }
                }, 
                ']' => {
                    if !(in_single_quotations || in_double_quotations) {
                        bracket_level -= 1;
                    }
                }, 
                _ => ()
            }
        }
        return bracket_level == 0;
    }
    return false;
}

fn is_string_literal(string: String) -> bool {
    if !(string.starts_with("\"") || string.starts_with("\'")) {
        return false;
    }
    if !(string.ends_with("\"") || string.ends_with("\'")) {
        return false;
    }
    
    let mut in_single_quotations: bool = false;
    let mut in_double_quotations: bool = false;
    let mut in_multiline_single_quotations: bool = false;
    let mut in_multiline_double_quotations: bool = false;
    
    let mut in_single_quotations_true_count: i32 = 0;
    let mut in_double_quotations_true_count: i32 = 0;
    let mut in_multiline_single_quotations_true_count: i32 = 0;
    let mut in_multiline_double_quotations_true_count: i32 = 0;
    
    for (index, c) in string.chars().enumerate() {
        match c {
            '\'' => {
                if !(in_double_quotations || in_multiline_double_quotations) {
                    if index >= 2 {
                        let prev_char = string.chars().nth(index - 1).unwrap();
                        let prev_prev_char = string.chars().nth(index - 2).unwrap();
                        if prev_char == '\'' && prev_prev_char == '\'' {
                            in_multiline_single_quotations = !in_multiline_single_quotations;
                            if in_multiline_single_quotations {
                                in_multiline_single_quotations_true_count += 1;
                            }
                            in_single_quotations_true_count -= 1;
                        } else if prev_char != '\\' {
                            in_single_quotations = !in_single_quotations;
                            if in_single_quotations {
                                in_single_quotations_true_count += 1;
                            }
                        }
                    } else {
                        in_single_quotations = !in_single_quotations;
                        if in_single_quotations {
                            in_single_quotations_true_count += 1;
                        }
                    }
                }
            }, 
            '\"' => {
                if !(in_single_quotations || in_multiline_single_quotations) {
                    if index >= 2 {
                        let prev_char = string.chars().nth(index - 1).unwrap();
                        let prev_prev_char = string.chars().nth(index - 2).unwrap();
                        if prev_char == '\"' && prev_prev_char == '\"' {
                            in_multiline_double_quotations = !in_multiline_double_quotations;
                            if in_multiline_double_quotations {
                                in_multiline_double_quotations_true_count += 1;
                            }
                            in_double_quotations_true_count -= 1;
                        } else if prev_char != '\\' {
                            in_double_quotations = !in_double_quotations;
                            if in_double_quotations {
                                in_double_quotations_true_count += 1;
                            }
                        }
                    } else {
                        in_double_quotations = !in_double_quotations;
                        if in_double_quotations {
                            in_double_quotations_true_count += 1;
                        }
                    }
                }
            }, 
            _ => ()
        }
    }
    
    return (in_single_quotations_true_count == 1 && !in_single_quotations) 
        || (in_double_quotations_true_count == 1 && !in_double_quotations) 
        || (in_multiline_single_quotations_true_count == 1 && !in_multiline_single_quotations) 
        || (in_multiline_double_quotations_true_count == 1 && !in_multiline_double_quotations);
}

fn is_function_call(string: String) -> bool {
    // Check if the string matches the regular expression for a function call.
    let re_function_call = Regex::new(PATTERN_FUNCTION_CALL_EXPRESSION).unwrap();
    let capt = re_function_call.captures(&string);
    match capt {
        None => return false, 
        Some(_) => (), 
    }
    
    // Check if the parentheses are not closed (not in quotations) before the final character.
    let mut in_single_quotations: bool = false;
    let mut in_double_quotations: bool = false;
    let mut in_brackets_depth: i32 = 0;
    
    for (index, c) in string.trim().chars().enumerate() {
        match c {
            '\'' => {
                let mut preceded_by_backslash: bool = false;
                if index > 0 {
                    let prev_char: char = string.chars().nth(index - 1).unwrap();
                    preceded_by_backslash = prev_char == '\\';
                }
                if !in_double_quotations && !preceded_by_backslash {
                    in_single_quotations = !in_single_quotations;
                }
            }, 
            '\"' => {
                let mut preceded_by_backslash: bool = false;
                if index > 0 {
                    let prev_char: char = string.chars().nth(index - 1).unwrap();
                    preceded_by_backslash = prev_char == '\\';
                }
                if !in_single_quotations && !preceded_by_backslash {
                    in_double_quotations = !in_double_quotations;
                }
            }, 
            '(' => {
                if !(in_single_quotations || in_double_quotations) {
                    in_brackets_depth += 1;
                }
            }, 
            ')' => {
                if !(in_single_quotations || in_double_quotations) {
                    in_brackets_depth -= 1;
                    if in_brackets_depth == 0 {
                        if index != string.len() - 1 {
                            return false;
                        }
                    }
                }
            }, 
            _ => ()
        }
    }
    return true;
}

fn is_array_access(string: String) -> bool {
    // Check if the string matches the regular expression for an array access.
    let re_array_access = Regex::new(PATTERN_ARRAY_DICT_ACCESS_EXPRESSION).unwrap();
    let capt = re_array_access.captures(&string);
    match capt {
        None => return false, 
        Some(_) => (), 
    }
    
    // Check if the square brackets are not closed (not in quotations) before the final character.
    let mut in_single_quotations: bool = false;
    let mut in_double_quotations: bool = false;
    let mut in_brackets_depth: i32 = 0;
    
    for (index, c) in string.trim().chars().enumerate() {
        match c {
            '\'' => {
                let mut preceded_by_backslash: bool = false;
                if index > 0 {
                    let prev_char: char = string.chars().nth(index - 1).unwrap();
                    preceded_by_backslash = prev_char == '\\';
                }
                if !in_double_quotations && !preceded_by_backslash {
                    in_single_quotations = !in_single_quotations;
                }
            }, 
            '\"' => {
                let mut preceded_by_backslash: bool = false;
                if index > 0 {
                    let prev_char: char = string.chars().nth(index - 1).unwrap();
                    preceded_by_backslash = prev_char == '\\';
                }
                if !in_single_quotations && !preceded_by_backslash {
                    in_double_quotations = !in_double_quotations;
                }
            }, 
            '[' => {
                if !(in_single_quotations || in_double_quotations) {
                    in_brackets_depth += 1;
                }
            }, 
            ']' => {
                if !(in_single_quotations || in_double_quotations) {
                    in_brackets_depth -= 1;
                    if in_brackets_depth == 0 {
                        if index != string.len() - 1 {
                            return false;
                        }
                    }
                }
            }, 
            _ => ()
        }
    }
    return true;
}

fn contains_arithmetic_symbols_not_enclosed(string: String) -> bool {
    let mut in_single_quotations: bool = false;
    let mut in_double_quotations: bool = false;
    let mut bracket_depth:        i32  = 0;
    let mut square_bracket_depth: i32  = 0;
    let mut curly_bracket_depth:  i32  = 0;
    let mut skip_next_char: bool       = false;
    
    for c in string.chars() {
        if skip_next_char {
            skip_next_char = false;
            continue;
        }
        match c {
            '\'' => {
                if !in_double_quotations {
                    in_single_quotations = !in_single_quotations;
                }
            }, 
            '\"' => {
                if !in_single_quotations {
                    in_double_quotations = !in_double_quotations;
                }
            }, 
            '(' => {
                if !(in_single_quotations || in_double_quotations) {
                    if square_bracket_depth == 0 && curly_bracket_depth == 0 {
                        bracket_depth += 1;
                    }
                }
            }, 
            ')' => {
                if !(in_single_quotations || in_double_quotations) {
                    if square_bracket_depth == 0 && curly_bracket_depth == 0 {
                        bracket_depth -= 1;
                    }
                }
            }, 
            '[' => {
                if !(in_single_quotations || in_double_quotations) {
                    if bracket_depth == 0 && curly_bracket_depth == 0 {
                        square_bracket_depth += 1;
                    }
                }
            }, 
            ']' => {
                if !(in_single_quotations || in_double_quotations) {
                    if bracket_depth == 0 && curly_bracket_depth == 0 {
                        square_bracket_depth -= 1;
                    }
                }
            }, 
            '{' => {
                if !(in_single_quotations || in_double_quotations) {
                    if bracket_depth == 0 && square_bracket_depth == 0 {
                        curly_bracket_depth += 1;
                    }
                }
            }, 
            '}' => {
                if !(in_single_quotations || in_double_quotations) {
                    if bracket_depth == 0 && square_bracket_depth == 0 {
                        curly_bracket_depth -= 1;
                    }
                }
            }, 
            '+'|'-'|'%'|'^'|'&'|'|'|'<'|'>'|'!'|'*'|'/'|'=' => {
                if !(in_single_quotations || in_double_quotations || bracket_depth > 0 || square_bracket_depth > 0 || curly_bracket_depth > 0) {
                    return true;
                }
            }, 
            _ => ()
        }
    }
    return false;
}

fn split_by_char(string: String, delimiter: char) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    
    let mut in_single_quotations: bool = false;
    let mut in_double_quotations: bool = false;
    let mut in_brackets_depth:        i32 = 0;
    let mut in_square_brackets_depth: i32 = 0;
    let mut in_curly_brackets_depth:  i32 = 0;
    let mut current_string: String = "".to_string();
    
    for (index, c) in string.chars().enumerate() {
        match c {
            '\'' => {
                let mut preceded_by_backslash: bool = false;
                if index > 0 {
                    let prev_char: char = string.chars().nth(index - 1).unwrap();
                    preceded_by_backslash = prev_char == '\\';
                }
                if !in_double_quotations && !preceded_by_backslash {
                    in_single_quotations = !in_single_quotations;
                }
                current_string.push(c);
            }, 
            '\"' => {
                let mut preceded_by_backslash: bool = false;
                if index > 0 {
                    let prev_char: char = string.chars().nth(index - 1).unwrap();
                    preceded_by_backslash = prev_char == '\\';
                }
                if !in_single_quotations && !preceded_by_backslash {
                    in_double_quotations = !in_double_quotations;
                }
                current_string.push(c);
            }, 
            '(' => {
                if !(in_single_quotations || in_double_quotations || in_square_brackets_depth > 0 || in_curly_brackets_depth > 0) {
                    in_brackets_depth += 1;
                }
                current_string.push(c);
            }, 
            ')' => {
                if !(in_single_quotations || in_double_quotations || in_square_brackets_depth > 0 || in_curly_brackets_depth > 0) {
                    if in_brackets_depth > 0 {
                        in_brackets_depth -= 1;
                    }
                }
                current_string.push(c);
            }, 
            '[' => {
                if !(in_single_quotations || in_double_quotations || in_brackets_depth > 0 || in_curly_brackets_depth > 0) {
                    in_square_brackets_depth += 1;
                }
                current_string.push(c);
            }, 
            ']' => {
                if !(in_single_quotations || in_double_quotations || in_brackets_depth > 0 || in_curly_brackets_depth > 0) {
                    if in_square_brackets_depth > 0 {
                        in_square_brackets_depth -= 1;
                    }
                }
                current_string.push(c);
            }, 
            '{' => {
                if !(in_single_quotations || in_double_quotations || in_square_brackets_depth > 0 || in_curly_brackets_depth > 0) {
                    in_curly_brackets_depth += 1;
                }
                current_string.push(c);
            }, 
            '}' => {
                if !(in_single_quotations || in_double_quotations || in_square_brackets_depth > 0 || in_curly_brackets_depth > 0) {
                    if in_curly_brackets_depth > 0 {
                        in_curly_brackets_depth -= 1;
                    }
                }
                current_string.push(c);
            }, 
            ',' => {
                if delimiter == ',' {
                    if !(in_single_quotations || in_double_quotations || in_brackets_depth > 0 || in_square_brackets_depth > 0 || in_curly_brackets_depth > 0) {
                        parts.push(current_string.trim().to_string());
                        current_string = "".to_string();
                    } else {
                        current_string.push(c);
                    }
                } else {
                    current_string.push(c);
                }
            }, 
            '.' => {
                if delimiter == '.' {
                    if !(in_single_quotations || in_double_quotations || in_brackets_depth > 0 || in_square_brackets_depth > 0 || in_curly_brackets_depth > 0) {
                        parts.push(current_string.trim().to_string());
                        current_string = "".to_string();
                    } else {
                        current_string.push(c);
                    }
                } else {
                    current_string.push(c);
                }
            }, 
            _ => {
                current_string.push(c);
            }
        }
    }
    parts.push(current_string.trim().to_string());
    
    return parts;
}

pub fn get_file_lines(filename: &str) -> Result<Vec<String>, std::io::Error> {
    let mut result: Vec<String> = Vec::new();
    let contents = fs::read_to_string(filename)?;
    for line in contents.lines() {
        result.push(line.to_string());
    }
    return Ok(result);
}

pub fn get_lines_for_test(filename: &str) -> Vec<String> {
    match get_file_lines(filename) {
        Ok(lines) => return lines, 
        Err(e) => {
            eprintln!("ERROR: Error occured while reading file '{}': '{}'", filename, e);
            panic!("Fatal error: file cannot be read.");
        }
    }
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

pub fn write_to_writer(writer: &mut BufWriter<Box<dyn Write>>, buffer: &[u8]) {
    match writer.write_all(buffer) {
        Ok(_) => (), 
        Err(e) => eprintln!("Error occured while writing a buffer of size {} to writer: '{}'", buffer.len(), e), 
    }
}

pub fn flush_writer(writer: &mut BufWriter<Box<dyn Write>>) {
    match writer.flush() {
        Ok(()) => (), 
        Err(e) => eprintln!("Error occured while flushing writer: '{}'", e), 
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use std::collections::HashMap;
    use std::io::ErrorKind;
    
    #[test]
    fn test_get_file_lines() {
        let filenames: Vec<&str> = vec![
            "test/create_function.py", 
            "test/blablabla_not_a_file.extension", 
            "test/create_class_typo.py", 
            "test/some_folder/some_non_existent_file.nothing", 
        ];
        
        let expected_results: Vec<Result<Vec<String>, std::io::Error>> = vec![
            Ok(vec![
                "def func_name(param1, param2, param3=5, *args, **kwargs):".to_string(), 
                "    Appel".to_string(), 
                "    for i in range(100):".to_string(), 
                "        print(i + 5 * 10)".to_string(), 
                "        if i % 5 == 0:".to_string(), 
                "            print(f\"{i} is divisible by 5\")".to_string(), 
                "        else:".to_string(), 
                "            print(\"no\")".to_string(), 
                "            if i % 7 == 0:".to_string(), 
                "                print(f\"{i} is divisible by 7\")".to_string(), 
                "    ".to_string(), 
                "    Banaan".to_string(), 
            ]), 
            Err(std::io::Error::new(ErrorKind::NotFound, "File does not exist.")), 
            Ok(vec![
                "# Used to test when the class definition does not match in the Class::new() method.".to_string(), 
                "cass Triangle(Shape):".to_string(), 
                "    ".to_string(), 
                "    pass".to_string(), 
            ]), 
            Err(std::io::Error::new(ErrorKind::NotFound, "File does not exist.")), 
        ];
        
        for (filename, expected_result) in std::iter::zip(filenames, expected_results) {
            // Read file.
            let file_contents: Result<Vec<String>, std::io::Error> = get_file_lines(filename);
            
            // Assert equality.
            match file_contents {
                Ok(contents) => {
                    match expected_result {
                        Ok(result) => assert_eq!(contents, result), 
                        Err(e) => {
                            eprintln!("Error '{}' produced while expecting result from file '{}'.", e, filename);
                            assert_eq!(true, false);
                        }, 
                    }
                }, 
                Err(e) => {
                    match expected_result {
                        Ok(_) => {
                            eprintln!("Expected error but got a valid result from file '{}'.", filename);
                            assert_eq!(true, false);
                        }, 
                        Err(e2) => assert_eq!(e.kind(), e2.kind()), 
                    }
                }
            }
        }
    }
    
    #[test]
    fn test_get_lines_for_test() {
        let filenames: Vec<&str> = vec![
            "test/create_class.py", 
        ];
        
        let expected_results: Vec<Vec<String>> = vec![
            vec![
                "class Rect(Shape):  ".to_string(), 
                "".to_string(), 
                "    STATIC_A = 5".to_string(), 
                "    ".to_string(), 
                "    def __init__(self, a=STATIC_A, b=5):".to_string(), 
                "        self.a=a".to_string(), 
                "        self.b=b+1".to_string(), 
                "    ".to_string(), 
                "    STATIC_B=6     ".to_string(), 
                "    ANOTHER_STATIC     =     5         ".to_string(), 
                "    ".to_string(), 
                "    def func2(self, a, b, c=2):  ".to_string(), 
                "        self.c = self.a * a + self.b * b + c".to_string(), 
                "        print(\"Banana\")".to_string(), 
                "".to_string(), 
                "    MORE_STATIC=\"Static string\"".to_string(), 
            ], 
        ];
        
        for (filename, expected_result) in std::iter::zip(filenames, expected_results) {
            assert_eq!(get_lines_for_test(filename), expected_result);
        }
    }
    
    #[test]
    fn test_vec_str_to_vec_line() {
        let inputs: Vec<Vec<String>> = vec![
            vec![
                "line number one".to_string(), 
                "some text".to_string(), 
                "    some thing".to_string(), 
                "some More text".to_string(), 
            ], 
            get_lines_for_test("test/file_as_string.py"), 
        ];
        
        let results: Vec<Vec<Line>> = vec![
            vec![
                Line::new(1, "line number one"), 
                Line::new(2, "some text"), 
                Line::new(3, "    some thing"), 
                Line::new(4, "some More text"), 
            ], 
            vec![
                Line::new(1, "import math, random as rnd"), 
                Line::new(2, "from os import listdir"), 
                Line::new(3, "from fruits import apple as a, banana as b, mango as m"), 
                Line::new(4, ""), 
                Line::new(5, "FPS = 60        # Frames per second"), 
                Line::new(6, "VSYNC = True    # Vertical sync"), 
                Line::new(7, ""), 
                Line::new(8, "class Rect:"), 
                Line::new(9, "    "), 
                Line::new(10, "    def __init__(self, a):"), 
                Line::new(11, "        self.a = a"), 
                Line::new(12, ""), 
                Line::new(13, "def function(p1, p2='5'):"), 
                Line::new(14, "    print(p1, p2)"), 
                Line::new(15, ""), 
                Line::new(16, "if __name__ == \"__main__\":"), 
                Line::new(17, "    function(Rect(a))"), 
            ]
        ];
        
        for (input, result) in std::iter::zip(inputs, results) {
            assert_eq!(vec_str_to_vec_line(&input), result);
        }
    }
    
    #[test]
    fn test_remove_empty_lines() {
        let inputs: Vec<Vec<Line>> = vec![
            vec_str_to_vec_line(&get_lines_for_test("test/file_as_string.py")), 
        ];
        
        let results: Vec<Vec<Line>> = vec![
            vec![
                Line::new(1, "import math, random as rnd"),
                Line::new(2, "from os import listdir"),
                Line::new(3, "from fruits import apple as a, banana as b, mango as m"),
                Line::new(5, "FPS = 60        # Frames per second"),
                Line::new(6, "VSYNC = True    # Vertical sync"),
                Line::new(8, "class Rect:"),
                Line::new(10, "    def __init__(self, a):"),
                Line::new(11, "        self.a = a"),
                Line::new(13, "def function(p1, p2='5'):"),
                Line::new(14, "    print(p1, p2)"),
                Line::new(16, "if __name__ == \"__main__\":"),
                Line::new(17, "    function(Rect(a))")
            ], 
        ];
        
        for (input, result) in std::iter::zip(inputs, results) {
            assert_eq!(remove_empty_lines(input), result);
        }
    }
    
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
        // Construct hashmap containing strings to match and if the string should match.
        let mut test_strings: HashMap<u32, (bool, &str)> = HashMap::new();
        test_strings.insert(0, (true, "import math"));
        test_strings.insert(1, (true, "   import     sys     \t,    \t re \t  , \t\tdatetime\t   ,  \t   zoneinfo  \t "));
        test_strings.insert(2, (true, "  \t  import a  \t  ,   b   \t\t\t   "));
        test_strings.insert(3, (true, "        \t\timport  \t time  "));
        test_strings.insert(4, (true, "import mypy.errorcodes as codes"));
        test_strings.insert(5, (true, "    import mypy.checkexpr"));
        test_strings.insert(6, (true, "import glob as fileglob"));
        test_strings.insert(7, (true, "    import tomllib"));
        test_strings.insert(8, (true, "         \t\t\t\t   import       banaaaan     as     \t\t\t    appel     \t\t\t      "));
        test_strings.insert(9, (false, "i m p o r t b a n a a n"));
        test_strings.insert(10, (false, "imp ort math"));
        test_strings.insert(11, (false, "from something import something else"));
        test_strings.insert(12, (false, "from x import y"));
        
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
        test_matches.insert(9, HashMap::from([("modules", "")]));
        test_matches.insert(10, HashMap::from([("modules", "")]));
        test_matches.insert(11, HashMap::from([("modules", "")]));
        test_matches.insert(12, HashMap::from([("modules", "")]));
        
        // Run tests.
        let re = Regex::new(PATTERN_IMPORT).unwrap();
        for (key_str, (should_match, value_str)) in test_strings.iter() {
            let capt = re.captures(value_str);
            let map = test_matches.get(&key_str).unwrap();
            match capt {
                Some(a) => {
                    if *should_match {
                        for (key, value) in map.iter() {
                            assert_eq!(&&a[*key], value);
                        }
                    } else {
                        panic!("ERROR: String '{}' should not have matched 'PATTERN_IMPORT', but did.", value_str);
                    }
                }, 
                None => {
                    if *should_match {
                        panic!("ERROR: String '{}' should have matched 'PATTERN_IMPORT', but didn't.", value_str);
                    }
                }
            }
        }
    }
    #[test]
    fn test_regex_pattern_from_import() {
        // Test PATTERN_FROM_IMPORT.
        // Construct hashmap containing strings to match.
        let mut test_strings: HashMap<u32, (bool, &str)> = HashMap::new();
        test_strings.insert(0, (true, "from a import b as c"));
        test_strings.insert(1, (true, "   \t\t\t    from     \t d\timport     e    as   f   ,   g   ,   h   \t\t\t   as i  \t "));
        test_strings.insert(2, (true, "from j import k aas, baas as p oop, f ish as dog, clo se as you       tube"));
        test_strings.insert(3, (true, "from mypy.options import PER_MODULE_OPTIONS, Options"));
        test_strings.insert(4, (true, "from     numpy.core.multiarray     import    \t\t _flagdict    \t,  \t flagsobj  \t     \t\t\t"));
        test_strings.insert(5, (true, "from mypy.infer import ArgumentInferContext, infer_function_type_arguments, infer_type_arguments"));
        test_strings.insert(6, (true, "from mypy import applytype, erasetype, join, message_registry, nodes, operators, types"));
        test_strings.insert(7, (true, "   \t\t\t from    \t\t        \t\t\t\t\t\t\t   mypy.semanal_enum        import         \t\t\t\tENUM_BASES"));
        test_strings.insert(8, (true, "    from . import _distributor_init"));
        test_strings.insert(9, (true, "        from numpy.__config__ import show as show_config"));
        test_strings.insert(10, (false, "import banana"));
        test_strings.insert(11, (false, "a = 5"));
        test_strings.insert(12, (false, "fr om banaan import yellow"));
        test_strings.insert(13, (false, "from mango im port orange"));
        
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
        test_matches.insert(10, HashMap::from([]));
        test_matches.insert(11, HashMap::from([]));
        test_matches.insert(12, HashMap::from([]));
        test_matches.insert(13, HashMap::from([]));
        
        // Run tests.
        let re = Regex::new(PATTERN_FROM_IMPORT).unwrap();
        for (key_str, (should_match, value_str)) in test_strings.iter() {
            let capt = re.captures(value_str);
            let map = test_matches.get(&key_str).unwrap();
            match capt {
                Some(a) => {
                    if *should_match {
                        for (key, value) in map.iter() {
                            assert_eq!(&&a[*key], value);
                        }
                    } else {
                        panic!("ERROR: String '{}' should not have matched 'PATTERN_FROM_IMPORT', but did.", value_str);
                    }
                    
                }, 
                None => {
                    if *should_match {
                        panic!("ERROR: String '{}' should have matched 'PATTERN_FROM_IMPORT', but didn't.", value_str);
                    }
                }
            }
        }
    }
    
    #[test]
    fn test_regex_pattern_function_start() {
        // Test PATTERN_FUNCTION_START.
        // Construct hashmap containing strings to match.
        let mut test_strings: HashMap<u32, (bool, &str)> = HashMap::new();
        test_strings.insert(0, (true, "def zeros(shape, dtype=None, order='C'):"));
        test_strings.insert(1, (true, "def eye(n,M=None, k=0, dtype=float, order='C'):"));
        test_strings.insert(2, (true, "    def __array_finalize__(self, obj):"));
        test_strings.insert(3, (true, "    def __mul__(self, other):  "));
        test_strings.insert(4, (true, "    def sum(self, axis=None, dtype=None, out=None):"));
        test_strings.insert(5, (true, "    def prod(self, axis=None, dtype=None, out=None):"));
        test_strings.insert(6, (true, "    def run_case(self, testcase: DataDrivenTestCase) -> None:"));
        test_strings.insert(7, (true, "def columns(self, *cols: ColumnClause[Any], **types: Union[TypeEngine[Any], Type[TypeEngine[Any]]]) -> TextAsFrom: "));
        test_strings.insert(8, (true, "    def self_group(self: _CL, against: Optional[Any] = ...) -> Union[_CL, Grouping[Any]]:"));
        test_strings.insert(9, (true, "         \t\t\tdef    func   (self, a=[5, 6, \"a\"], b, c, d: List[Tuple[str]]=(5, 6, 7, banaan), _str: bool=False)    ->     List[Tuple[str, int], str]  :   \t\t \t\t    "));
        test_strings.insert(10, (false, "class Rect(Shape):"));
        test_strings.insert(11, (false, "import foo"));
        test_strings.insert(12, (false, "from bar import baz"));
        test_strings.insert(13, (false, "x = 5"));
        test_strings.insert(14, (false, "x += 5"));
        
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
        test_matches.insert(10, HashMap::from([("indentation", ""), ("name", ""), ("params", "")]));
        test_matches.insert(11, HashMap::from([("indentation", ""), ("name", ""), ("params", "")]));
        test_matches.insert(12, HashMap::from([("indentation", ""), ("name", ""), ("params", "")]));
        test_matches.insert(13, HashMap::from([("indentation", ""), ("name", ""), ("params", "")]));
        test_matches.insert(14, HashMap::from([("indentation", ""), ("name", ""), ("params", "")]));
        
        // Run tests.
        let re = Regex::new(PATTERN_FUNCTION_START).unwrap();
        for (key_str, (should_match, value_str)) in test_strings.iter() {
            let capt = re.captures(value_str);
            let map = test_matches.get(&key_str).unwrap();
            match capt {
                Some(a) => {
                    if *should_match {
                        for (key, value) in map.iter() {
                            assert_eq!(&&a[*key], value);
                        }
                    } else {
                        panic!("ERROR: String '{}' should not have matched 'PATTERN_FUNCTION_START', but did.", value_str);
                    }
                }, 
                None => {
                    if *should_match {
                        panic!("ERROR: String '{}' should have matched 'PATTERN_FUNCTION_START', but didn't.", value_str);
                    }
                }
            }
        }
    }
    
    #[test]
    fn test_regex_pattern_class_start() {
        // Test PATTERN_CLASS_START.
        // Construct hashmap containing strings to match.
        let mut test_strings: HashMap<u32, (bool, &str)> = HashMap::new();
        test_strings.insert(0, (true, "class BindParameter(ColumnElement[_T]):"));
        test_strings.insert(1, (true, "class Triangle:"));
        test_strings.insert(2, (true, "    class Rect(Shape):"));
        test_strings.insert(3, (true, "class ModuleWrapper(nn.Module):"));
        test_strings.insert(4, (true, "class UntypedStorage(torch._C.StorageBase, _StorageBase):"));
        test_strings.insert(5, (true, "                  \t\t\tclass Library:    \t\t  \t\t"));
        test_strings.insert(6, (true, "class SourceChangeWarning(Warning):"));
        test_strings.insert(7, (true, "     \t\t\t\t\t\t            class ETKernelIndex:   "));
        test_strings.insert(8, (false, "def __init__(self, a=5, b={a: \"b=5\"}):"));
        test_strings.insert(9, (false, "import foo"));
        test_strings.insert(10, (false, "from bar import baz"));
        test_strings.insert(11, (false, "x = 5"));
        test_strings.insert(12, (false, "x += 5"));
        
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
        test_matches.insert(8, HashMap::from([("indentation", ""), ("name", ""), ("parent", "")]));
        test_matches.insert(9, HashMap::from([("indentation", ""), ("name", ""), ("parent", "")]));
        test_matches.insert(10, HashMap::from([("indentation", ""), ("name", ""), ("parent", "")]));
        test_matches.insert(11, HashMap::from([("indentation", ""), ("name", ""), ("parent", "")]));
        test_matches.insert(12, HashMap::from([("indentation", ""), ("name", ""), ("parent", "")]));
        
        // Run tests.
        let re = Regex::new(PATTERN_CLASS_START).unwrap();
        for (key_str, (should_match, value_str)) in test_strings.iter() {
            let capt = re.captures(value_str);
            let map = test_matches.get(&key_str).unwrap();
            match capt {
                Some(a) => {
                    if *should_match {
                        for (key, value) in map.iter() {
                            if key == &"parent" {
                                assert_eq!(&a.name("parent").map(|m| m.as_str()).unwrap_or(""), value);
                            } else {
                                assert_eq!(&&a[*key], value);
                            }
                        }
                    } else {
                        panic!("ERROR: String '{}' should not have matched 'PATTERN_CLASS_START', but did.", value_str);
                    }
                }, 
                None => {
                    if *should_match {
                        panic!("ERROR: String '{}' should have matched 'PATTERN_CLASS_START', but didn't.", value_str);
                    }
                }
            }
        }
    }
    
    #[test]
    fn test_regex_pattern_class_variable() {
        // Test PATTERN_CLASS_VARIABLE.
        // Construct hashmap containing strings to match.
        let mut test_strings: HashMap<u32, (bool, &str)> = HashMap::new();
        test_strings.insert(0, (true, "    arg_meta: Tuple[ETKernelKeyOpArgMeta, ...] = ()"));
        test_strings.insert(1, (true, "    default: bool = False"));
        test_strings.insert(2, (true, "    version: int = KERNEL_KEY_VERSION"));
        test_strings.insert(3, (true, "        CLASS_VAR   =     5"));
        test_strings.insert(4, (true, "    instructions = 1"));
        test_strings.insert(5, (true, "    MAXDIM = 21201"));
        test_strings.insert(6, (true, "        CLASS_STR   = \t\t\t\t  \"Bananas are very                  spacyyyyyyyyy\"    "));
        test_strings.insert(7, (true, "    deserialized_objects = {}"));
        test_strings.insert(8, (false, "def __init__(self, a=5, b={a: \"b=5\"}):"));
        test_strings.insert(9, (false, "import foo"));
        test_strings.insert(10, (false, "     from bar import baz"));
        test_strings.insert(11, (false, "  x += 5"));
        test_strings.insert(12, (false, "    x = 5"));
        test_strings.insert(13, (false, "       y = \"B = 5\""));
        test_strings.insert(14, (false, "    \"\"\"B = 5\"\"\""));
        
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
        test_string_indentations.insert(8, 0);
        test_string_indentations.insert(9, 5);
        test_string_indentations.insert(10, 2);
        test_string_indentations.insert(11, 4);
        test_string_indentations.insert(12, 5);
        test_string_indentations.insert(13, 16);
        test_string_indentations.insert(14, 4);
        
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
        test_matches.insert(8, HashMap::from([("varname", ""), ("varvalue", "")]));
        test_matches.insert(9, HashMap::from([("varname", ""), ("varvalue", "")]));
        test_matches.insert(10, HashMap::from([("varname", ""), ("varvalue", "")]));
        test_matches.insert(11, HashMap::from([("varname", ""), ("varvalue", "")]));
        test_matches.insert(12, HashMap::from([("varname", ""), ("varvalue", "")]));
        test_matches.insert(13, HashMap::from([("varname", ""), ("varvalue", "")]));
        test_matches.insert(14, HashMap::from([("varname", ""), ("varvalue", "")]));
        
        // Run tests.
        for (key_str, (should_match, value_str)) in test_strings.iter() {
            let num_spaces = test_string_indentations.get(&key_str).unwrap();
            let re = Regex::new(PATTERN_CLASS_VARIABLE.replace("INDENTATION", format!("{}", num_spaces).as_str()).as_str()).unwrap();
            let capt = re.captures(value_str);
            let map = test_matches.get(&key_str).unwrap();
            match capt {
                Some(a) => {
                    if *should_match {
                        for (key, value) in map.iter() {
                            assert_eq!(&&a[*key], value);
                        }
                    } else {
                        panic!("ERROR: String '{}' should not have matched 'PATTERN_CLASS_VARIABLE', but did.", value_str);
                    }
                }, 
                None => {
                    if *should_match {
                        panic!("ERROR: String '{}' should have matched 'PATTERN_CLASS_VARIABLE', but didn't.", value_str);
                    }
                }
            }
        }
    }
    
    #[test]
    fn test_regex_pattern_for_loop() {
        // Test PATTERN_FOR_LOOP.
        // Construct hashmap containing strings to match.
        let mut test_strings: HashMap<u32, (bool, &str)> = HashMap::new();
        test_strings.insert(0, (true, "for a in b:"));
        test_strings.insert(1, (true, "    for    a    in     b  :   "));
        test_strings.insert(2, (true, "  for   __A2C     in B__5G : "));
        test_strings.insert(3, (false, "  while a > 5 : "));
        test_strings.insert(4, (false, "if a == 5:"));
        test_strings.insert(5, (false, "elif a == 5:"));
        test_strings.insert(6, (false, "else:"));
        test_strings.insert(7, (false, " gg enijneighe hguiehgiu h uihg eiurhgiuheiughiu  "));
        test_strings.insert(8, (false, "  def func(a=5, b=6):   "));
        test_strings.insert(9, (false, "    class   Rect(Shape):   "));
        test_strings.insert(10, (false, "import os"));
        test_strings.insert(11, (false, "from os import listdir"));
        test_strings.insert(12, (true, "for grhgyerFESFShuyg in hgerhguGSSFEeg__545: "));
        test_strings.insert(13, (true, "for a in b.get_c(d, e.x, \"Some string\").f[g].h:"));
        
        // Construct hashmap containing hashmaps containing values of named groups.
        let mut test_matches: HashMap<u32, HashMap<&str, &str>> = HashMap::new();
        test_matches.insert(0, HashMap::from([("itervar", "a"), ("iterator", "b")]));
        test_matches.insert(1, HashMap::from([("itervar", "a"), ("iterator", "b")]));
        test_matches.insert(2, HashMap::from([("itervar", "__A2C"), ("iterator", "B__5G")]));
        test_matches.insert(3, HashMap::from([("itervar", ""), ("iterator", "")]));
        test_matches.insert(4, HashMap::from([("itervar", ""), ("iterator", "")]));
        test_matches.insert(5, HashMap::from([("itervar", ""), ("iterator", "")]));
        test_matches.insert(6, HashMap::from([("itervar", ""), ("iterator", "")]));
        test_matches.insert(7, HashMap::from([("itervar", ""), ("iterator", "")]));
        test_matches.insert(8, HashMap::from([("itervar", ""), ("iterator", "")]));
        test_matches.insert(9, HashMap::from([("itervar", ""), ("iterator", "")]));
        test_matches.insert(10, HashMap::from([("itervar", ""), ("iterator", "")]));
        test_matches.insert(11, HashMap::from([("itervar", ""), ("iterator", "")]));
        test_matches.insert(12, HashMap::from([("itervar", "grhgyerFESFShuyg"), ("iterator", "hgerhguGSSFEeg__545")]));
        test_matches.insert(13, HashMap::from([("itervar", "a"), ("iterator", "b.get_c(d, e.x, \"Some string\").f[g].h")]));
        
        // Run tests.
        let re = Regex::new(PATTERN_FOR_LOOP).unwrap();
        for (key_str, (should_match, value_str)) in test_strings.iter() {
            let capt = re.captures(value_str);
            let map = test_matches.get(&key_str).unwrap();
            match capt {
                Some(a) => {
                    if *should_match {
                        for (key, value) in map.iter() {
                            assert_eq!(&&a[*key].trim().to_string(), value);
                        }
                    } else {
                        panic!("ERROR: String '{}' should not have matched 'PATTERN_FOR_LOOP', but did.", value_str);
                    }
                }, 
                None => {
                    if *should_match {
                        panic!("ERROR: String '{}' should have matched 'PATTERN_FOR_LOOP', but didn't.", value_str);
                    }
                }
            }
        }
    }
    
    #[test]
    fn test_regex_pattern_while_loop() {
        // Test PATTERN_WHILE_LOOP.
        // Construct hashmap containing strings to match.
        let mut test_strings: HashMap<u32, (bool, &str)> = HashMap::new();
        test_strings.insert(0, (true, "while a:"));
        test_strings.insert(1, (true, "while a > 5:"));
        test_strings.insert(2, (true, "while a == 4:"));
        test_strings.insert(3, (true, "while a.get_b(c).d + 6 < 70 * q - p:"));
        test_strings.insert(4, (false, "for a in b:"));
        test_strings.insert(5, (false, "def func(a, b):"));
        test_strings.insert(6, (true, "   while   a   > p+5:   "));
        test_strings.insert(7, (true, "   while   a   > p+5  :   "));
        test_strings.insert(8, (false, "class Rect:"));
        test_strings.insert(9, (false, "class Rect(Shape):"));
        test_strings.insert(10, (false, "a = b.g + 5:"));
        
        // Construct hashmap containing hashmaps containing values of named groups.
        let mut test_matches: HashMap<u32, HashMap<&str, &str>> = HashMap::new();
        test_matches.insert(0, HashMap::from([("condition", "a")]));
        test_matches.insert(1, HashMap::from([("condition", "a > 5")]));
        test_matches.insert(2, HashMap::from([("condition", "a == 4")]));
        test_matches.insert(3, HashMap::from([("condition", "a.get_b(c).d + 6 < 70 * q - p")]));
        test_matches.insert(4, HashMap::from([("condition", "")]));
        test_matches.insert(5, HashMap::from([("condition", "")]));
        test_matches.insert(6, HashMap::from([("condition", "a   > p+5")]));
        test_matches.insert(7, HashMap::from([("condition", "a   > p+5")]));
        test_matches.insert(8, HashMap::from([("condition", "")]));
        test_matches.insert(9, HashMap::from([("condition", "")]));
        test_matches.insert(10, HashMap::from([("condition", "")]));
        
        // Run tests.
        let re = Regex::new(PATTERN_WHILE_LOOP).unwrap();
        for (key_str, (should_match, value_str)) in test_strings.iter() {
            let capt = re.captures(value_str);
            let map = test_matches.get(&key_str).unwrap();
            match capt {
                Some(a) => {
                    if *should_match {
                        for (key, value) in map.iter() {
                            assert_eq!(&&a[*key].trim().to_string(), value);
                        }
                    } else {
                        panic!("ERROR: String '{}' should not have matched 'PATTERN_WHILE_LOOP', but did.", value_str);
                    }
                }, 
                None => {
                    if *should_match {
                        panic!("ERROR: String '{}' should have matched 'PATTERN_WHILE_LOOP', but didn't.", value_str);
                    }
                }
            }
        }
    }
    
    #[test]
    fn test_regex_pattern_function_call_expression() {
        // Test PATTERN_FUNCTION_CALL_EXPRESSION.
        // Construct hashmap containing strings to match.
        let mut test_strings: HashMap<u32, (bool, &str)> = HashMap::new();
        test_strings.insert(0, (true, "function(a_rere04304fTGER)"));
        test_strings.insert(1, (true, "_efwfEFWF37423GgrGrg(g5454GFGge343WSFEFrw,\"Some string here!!! $%@%&^*&%^&@$@#$^%^:\\\">\")"));
        test_strings.insert(2, (true, "function()"));
        test_strings.insert(3, (false, "5function(a_rere04304fTGER)"));
        test_strings.insert(4, (false, "53453453"));
        test_strings.insert(5, (false, "function(a_rere04304fTGER"));
        test_strings.insert(6, (false, "fwfwfwh5353RGGE__egerge"));
        test_strings.insert(7, (true, "fwfwfwh5353RGGE__egerge(a=[5, 6, b, c],b={\"a\": 5},c=\"\"\"gehgghe\"\"\")"));
        test_strings.insert(8, (false, "$@#$@eGERGegEG43534_etgerg$%$#5"));
        test_strings.insert(9, (false, "i*j+(k%l)"));
        
        // Construct hashmap containing hashmaps containing values of named groups.
        let mut test_matches: HashMap<u32, HashMap<&str, &str>> = HashMap::new();
        test_matches.insert(0, HashMap::from([("name", "function"), ("arguments", "a_rere04304fTGER")]));
        test_matches.insert(1, HashMap::from([("name", "_efwfEFWF37423GgrGrg"), ("arguments", "g5454GFGge343WSFEFrw,\"Some string here!!! $%@%&^*&%^&@$@#$^%^:\\\">\"")]));
        test_matches.insert(2, HashMap::from([("name", "function"), ("arguments", "")]));
        test_matches.insert(3, HashMap::from([("name", ""), ("arguments", "")]));
        test_matches.insert(4, HashMap::from([("name", ""), ("arguments", "")]));
        test_matches.insert(5, HashMap::from([("name", ""), ("arguments", "")]));
        test_matches.insert(6, HashMap::from([("name", ""), ("arguments", "")]));
        test_matches.insert(7, HashMap::from([("name", "fwfwfwh5353RGGE__egerge"), ("arguments", "a=[5, 6, b, c],b={\"a\": 5},c=\"\"\"gehgghe\"\"\"")]));
        test_matches.insert(8, HashMap::from([("name", ""), ("arguments", "")]));
        test_matches.insert(9, HashMap::from([("name", ""), ("arguments", "")]));
        
        // Run tests.
        let re = Regex::new(PATTERN_FUNCTION_CALL_EXPRESSION).unwrap();
        for (key_str, (should_match, value_str)) in test_strings.iter() {
            let capt = re.captures(value_str);
            let map = test_matches.get(&key_str).unwrap();
            match capt {
                Some(a) => {
                    if *should_match {
                        for (key, value) in map.iter() {
                            assert_eq!(&&a[*key], value);
                        }
                    } else {
                        panic!("ERROR: String '{}' should not have matched 'PATTERN_FUNCTION_CALL_EXPRESSION', but did.", value_str);
                    }
                }, 
                None => {
                    if *should_match {
                        panic!("ERROR: String '{}' should have matched 'PATTERN_FUNCTION_CALL_EXPRESSION', but didn't.", value_str);
                    }
                }
            }
        }
    }
    
    #[test]
    fn test_regex_pattern_array_dict_access_expression() {
        // Test PATTERN_ARRAY_DICT_ACCESS_EXPRESSION.
        // Construct hashmap containing strings to match.
        let mut test_strings: HashMap<u32, (bool, &str)> = HashMap::new();
        test_strings.insert(0, (true, "a[5]"));
        test_strings.insert(1, (false, "a[5"));
        test_strings.insert(2, (true, "_453GTGRtgrt345[4343*GFGF44_gdgdfg+i*5]"));
        test_strings.insert(3, (true, "dict[\"Banaan4274892$@$@^$^$^$!!@)(*^[\"]"));
        test_strings.insert(4, (false, "gegegegeggdg"));
        test_strings.insert(5, (false, "3535[3534534]"));
        test_strings.insert(6, (false, "function(a, b, c, d, e)"));
        test_strings.insert(7, (false, "class Rect():"));
        test_strings.insert(8, (false, "def function(a, b, c, d):"));
        test_strings.insert(9, (false, "Banaan"));
        
        // Construct hashmap containing hashmaps containing values of named groups.
        let mut test_matches: HashMap<u32, HashMap<&str, &str>> = HashMap::new();
        test_matches.insert(0, HashMap::from([("name", "a"), ("index", "5")]));
        test_matches.insert(1, HashMap::from([("name", ""), ("index", "")]));
        test_matches.insert(2, HashMap::from([("name", "_453GTGRtgrt345"), ("index", "4343*GFGF44_gdgdfg+i*5")]));
        test_matches.insert(3, HashMap::from([("name", "dict"), ("index", "\"Banaan4274892$@$@^$^$^$!!@)(*^[\"")]));
        test_matches.insert(4, HashMap::from([("name", ""), ("index", "")]));
        test_matches.insert(5, HashMap::from([("name", ""), ("index", "")]));
        test_matches.insert(6, HashMap::from([("name", ""), ("index", "")]));
        test_matches.insert(7, HashMap::from([("name", ""), ("index", "")]));
        test_matches.insert(8, HashMap::from([("name", ""), ("index", "")]));
        test_matches.insert(9, HashMap::from([("name", ""), ("index", "")]));
        
        // Run tests.
        let re = Regex::new(PATTERN_ARRAY_DICT_ACCESS_EXPRESSION).unwrap();
        for (key_str, (should_match, value_str)) in test_strings.iter() {
            let capt = re.captures(value_str);
            let map = test_matches.get(&key_str).unwrap();
            match capt {
                Some(a) => {
                    if *should_match {
                        for (key, value) in map.iter() {
                            assert_eq!(&&a[*key], value);
                        }
                    } else {
                        panic!("ERROR: String '{}' should not have matched 'PATTERN_ARRAY_DICT_ACCESS_EXPRESSION', but did.", value_str);
                    }
                }, 
                None => {
                    if *should_match {
                        panic!("ERROR: String '{}' should have matched 'PATTERN_ARRAY_DICT_ACCESS_EXPRESSION', but didn't.", value_str);
                    }
                }
            }
        }
    }
    
    #[test]
    fn test_regex_pattern_variable_name_expression() {
        // Test PATTERN_VARIABLE_NAME_EXPRESSION.
        // Construct hashmap containing strings to match.
        let mut test_strings: HashMap<u32, (bool, &str)> = HashMap::new();
        test_strings.insert(0, (true, "_56"));
        test_strings.insert(1, (true, "ffsefSF__r34534"));
        test_strings.insert(2, (true, "name"));
        test_strings.insert(3, (true, "something"));
        test_strings.insert(4, (true, "rhrht4535_eGERGERGERGER534534grtYrthrt"));
        test_strings.insert(5, (false, "5"));
        test_strings.insert(6, (false, "553_3453453"));
        test_strings.insert(7, (false, "53535THTRHRTHrhrth__H242"));
        
        // Run tests.
        let re = Regex::new(PATTERN_VARIABLE_NAME_EXPRESSION).unwrap();
        for (_key_str, (should_match, value_str)) in test_strings.iter() {
            let capt = re.captures(value_str);
            match capt {
                Some(_) => {
                    if !*should_match {
                        panic!("ERROR: String '{}' should not have matched 'PATTERN_VARIABLE_NAME_EXPRESSION', but did.", value_str);
                    }
                }, 
                None => {
                    if *should_match {
                        panic!("ERROR: String '{}' should have matched 'PATTERN_VARIABLE_NAME_EXPRESSION', but didn't.", value_str);
                    }
                }
            }
        }
    }
    
    #[test]
    fn test_regex_pattern_with_statement() {
        // Test PATTERN_WITH_STATEMENT.
        // Construct hashmap containing strings to match.
        let mut test_strings: HashMap<u32, (bool, &str)> = HashMap::new();
        test_strings.insert(0, (true, "with open(\"file.txt\") as file:"));
        test_strings.insert(1, (true, "with socket.accept(\"127.0.0.1\", 1234) as conn:"));
        test_strings.insert(2, (true, "  with socket.accept(\"127.0.0.1\", 1234) as conn:"));
        test_strings.insert(3, (true, "with socket.accept(\"127.0.0.1\", 1234) as conn  :"));
        test_strings.insert(4, (true, "     with     banaan(appel    ,    peer)    as    _54353greGG_TRHRTHRTH  :    "));
        test_strings.insert(5, (false, "def func():"));
        test_strings.insert(6, (false, "while a > 5:"));
        test_strings.insert(7, (false, "class Shape(Rect):"));
        test_strings.insert(8, (false, "class Shape:"));
        test_strings.insert(9, (false, "for a in b:"));
        test_strings.insert(10, (false, "if a == b:"));
        test_strings.insert(11, (false, "elif a > b:"));
        test_strings.insert(12, (false, "else:"));
        test_strings.insert(13, (false, "b = 5"));
        
        // Construct hashmap containing hashmaps containing values of named groups.
        let mut test_matches: HashMap<u32, HashMap<&str, &str>> = HashMap::new();
        test_matches.insert(0, HashMap::from([("expression", "open(\"file.txt\")"), ("alias", "file")]));
        test_matches.insert(1, HashMap::from([("expression", "socket.accept(\"127.0.0.1\", 1234)"), ("alias", "conn")]));
        test_matches.insert(2, HashMap::from([("expression", "socket.accept(\"127.0.0.1\", 1234)"), ("alias", "conn")]));
        test_matches.insert(3, HashMap::from([("expression", "socket.accept(\"127.0.0.1\", 1234)"), ("alias", "conn")]));
        test_matches.insert(4, HashMap::from([("expression", "banaan(appel    ,    peer)   "), ("alias", "_54353greGG_TRHRTHRTH")]));
        test_matches.insert(5, HashMap::from([("expression", ""), ("alias", "")]));
        test_matches.insert(6, HashMap::from([("expression", ""), ("alias", "")]));
        test_matches.insert(7, HashMap::from([("expression", ""), ("alias", "")]));
        test_matches.insert(8, HashMap::from([("expression", ""), ("alias", "")]));
        test_matches.insert(9, HashMap::from([("expression", ""), ("alias", "")]));
        test_matches.insert(10, HashMap::from([("expression", ""), ("alias", "")]));
        test_matches.insert(11, HashMap::from([("expression", ""), ("alias", "")]));
        test_matches.insert(12, HashMap::from([("expression", ""), ("alias", "")]));
        test_matches.insert(13, HashMap::from([("expression", ""), ("alias", "")]));
        
        // Run tests.
        let re = Regex::new(PATTERN_WITH_STATEMENT).unwrap();
        for (key_str, (should_match, value_str)) in test_strings.iter() {
            let capt = re.captures(value_str);
            let map = test_matches.get(&key_str).unwrap();
            match capt {
                Some(a) => {
                    if *should_match {
                        for (key, value) in map.iter() {
                            assert_eq!(&&a[*key], value);
                        }
                    } else {
                        panic!("ERROR: String '{}' should not have matched 'PATTERN_WITH_STATEMENT', but did.", value_str);
                    }
                }, 
                None => {
                    if *should_match {
                        panic!("ERROR: String '{}' should have matched 'PATTERN_WITH_STATEMENT', but didn't.", value_str);
                    }
                }
            }
        }
    }
    
    #[test]
    fn test_partialeq_implementations() {
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let mut writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        // Test line partialeq.
        let line_org: Line = Line::new(3785634756, "Some string");
        let line_same: Line = Line::new(3785634756, "Some string");
        let line_diff_number: Line = Line::new(2948278964, "Some string");
        let line_diff_text: Line = Line::new(3785634756, "Some other string");
        
        assert_eq!(line_org == line_same, true);
        assert_eq!(line_org == line_diff_number, false);
        assert_eq!(line_org == line_diff_text, false);
        
        // Test assignment partialeq.
        let asg_org: Assignment = Assignment {name: "a".to_string(), value: "5".to_string(), source: Line::new(1, "a = 5")};
        let asg_same: Assignment = asg_org.clone();
        assert_eq!(asg_org == asg_same, true);
        
        let mut asg_diff_name: Assignment = asg_same.clone();
        asg_diff_name.name = "b".to_string();
        assert_eq!(asg_org == asg_diff_name, false);
        
        let mut asg_diff_value: Assignment = asg_same.clone();
        asg_diff_value.value = "6".to_string();
        assert_eq!(asg_org == asg_diff_value, false);
        
        let mut asg_diff_source: Assignment = asg_same.clone();
        asg_diff_source.source = Line::new(2, "b = 6");
        assert_eq!(asg_org == asg_diff_source, false);
        
        // Test file partialeq.
        let lines_str: Vec<String> = get_lines_for_test("test/test_file_partialeq.py");
        let lines: Vec<Line> = vec_str_to_vec_line(&lines_str);
        let file_org: File = File::new("test/test_file_partialeq.py", &lines, &mut writer);
        let file_same: File = file_org.clone();
        assert_eq!(file_org == file_same, true);
        
        let mut file_diff_name: File = file_same.clone();
        file_diff_name.name = "other_name".to_string();
        assert_eq!(file_org == file_diff_name, false);
        
        let mut file_diff_imports: File = file_same.clone();
        file_diff_imports.imports = vec!["banana".to_string(), "np".to_string(), "plt".to_string()];
        assert_eq!(file_org == file_diff_imports, false);
        
        let mut file_diff_global_variables: File = file_same.clone();
        file_diff_global_variables.global_variables = vec![
            Assignment::new(&Line::new(1, "GLOBAL_VARIABLE = 5")).unwrap(), 
            Assignment::new(&Line::new(2, "SETTING_FPS = 60")).unwrap(), 
            Assignment::new(&Line::new(3, "SETTING_VSYNC = 1")).unwrap(), 
        ];
        assert_eq!(file_org == file_diff_global_variables, false);
        
        let mut file_diff_functions: File = file_same.clone();
        file_diff_functions.functions = vec![
            Function {
                name: "dummy".to_string(), 
                parameters: vec!["parameter".to_string()], 
                functions: vec![], 
                source: vec![
                    Line::new(1, "def dummy(parameter):"), 
                    Line::new(2, "    return parameter * 2")
                ]
            }
        ];
        assert_eq!(file_org == file_diff_functions, false);
        
        let mut file_diff_classes: File = file_same.clone();
        file_diff_classes.classes = vec![
            Class {
                name: "dummy".to_string(), 
                parent: "dummy_parent".to_string(), 
                variables: vec![], 
                methods: vec![], 
                classes: vec![]
            }
        ];
        assert_eq!(file_org == file_diff_classes, false);
        
        // Test function partialeq.
        let lines: Vec<Line> = vec![
            Line::new(1, "def func(p1, p2=8):"), 
            Line::new(2, "    print(\"Calculating...\")"), 
            Line::new(3, "    return p1 + p2, p1 - p2"), 
        ];
        let function_org: Function = Function::new(&lines, &mut writer);
        let function_same: Function = function_org.clone();
        assert_eq!(function_org == function_same, true);
        
        let mut function_diff_name: Function = function_same.clone();
        function_diff_name.name = "other_name".to_string();
        assert_eq!(function_org == function_diff_name, false);
        
        let mut function_diff_parameters: Function = function_same.clone();
        function_diff_parameters.parameters = vec!["banaan".to_string()];
        assert_eq!(function_org == function_diff_parameters, false);
        
        let mut function_diff_functions: Function = function_same.clone();
        function_diff_functions.functions = vec![Function::default()];
        assert_eq!(function_org == function_diff_functions, false);
        
        let mut function_diff_source: Function = function_same.clone();
        function_diff_source.source = vec![Line::new(63545, "Dummy line")];
        assert_eq!(function_org == function_diff_source, false);
        
        // Test class partialeq.
        let lines: Vec<Line> = vec![
            Line::new(1, "class Name(Parent):"), 
            Line::new(3, "    CLASS__VAR=100"), 
            Line::new(5, "    def __init__(self, a):"), 
            Line::new(6, "        self.a = a"), 
            Line::new(8, "    class SubClass:"), 
            Line::new(9, "        def __init__(self, b):"), 
            Line::new(10, "           self.b = b"),             
        ];
        let class_org: Class = Class::new(&lines, &mut writer);
        let class_same: Class = class_org.clone();
        assert_eq!(class_org == class_same, true);
        
        let mut class_diff_name: Class = class_same.clone();
        class_diff_name.name = "other_class_name".to_string();
        assert_eq!(class_org == class_diff_name, false);
        
        let mut class_diff_parent: Class = class_same.clone();
        class_diff_parent.parent = "other_parent".to_string();
        assert_eq!(class_org == class_diff_parent, false);
        
        let mut class_diff_variables: Class = class_same.clone();
        class_diff_variables.variables = vec![];
        assert_eq!(class_org == class_diff_variables, false);
        
        let mut class_diff_methods: Class = class_same.clone();
        class_diff_methods.methods = vec![Function::default()];
        assert_eq!(class_org == class_diff_methods, false);
        
        let mut class_diff_classes: Class = class_same.clone();
        class_diff_classes.classes = vec![];
        assert_eq!(class_org == class_diff_classes, false);
    }
    
    #[test]
    fn test_as_string() {
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let mut writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        // Test Line::as_string().
        let lines: Vec<Line> = vec![
            Line::new(0, "I seriously doubt she actually believes you."), 
            Line::new(1, "My boyfriend loves this song."), 
            Line::new(7, "The store had multiple skeletons they claimed were real, alongside a taxidermies, two-headed calf."), 
            Line::new(10, "Elizabeth is traveling all around the country to hear directly from people like you."), 
            Line::new(75, "She sunburned herself so badly she looked like a tomato."), 
            Line::new(100, "You did right in me by telling the truth."), 
            Line::new(384, "I agree it’s not bad to steal from a convenience store."), 
            Line::new(1000, "I like open spaces."), 
            Line::new(2945, "Don’t worry, be happy!"), 
            Line::new(6923858, "Being late is never okay."), 
        ];
        
        let strings_zero_indentation: Vec<(usize, String)> = vec![
            (0, "Line    0: I seriously doubt she actually believes you.".to_string()), 
            (0, "Line    1: My boyfriend loves this song.".to_string()), 
            (0, "Line    7: The store had multiple skeletons they claimed were real, alongside a taxidermies, two-headed calf.".to_string()), 
            (0, "Line   10: Elizabeth is traveling all around the country to hear directly from people like you.".to_string()), 
            (0, "Line   75: She sunburned herself so badly she looked like a tomato.".to_string()), 
            (0, "Line  100: You did right in me by telling the truth.".to_string()), 
            (0, "Line  384: I agree it’s not bad to steal from a convenience store.".to_string()), 
            (0, "Line 1000: I like open spaces.".to_string()), 
            (0, "Line 2945: Don’t worry, be happy!".to_string()), 
            (0, "Line 6923858: Being late is never okay.".to_string()), 
        ];
        
        let strings_random_indentation: Vec<(usize, String)> = vec![
            (50, "Line    0: I seriously doubt she actually believes you.".to_string()), 
            (22, "Line    1: My boyfriend loves this song.".to_string()), 
            (53, "Line    7: The store had multiple skeletons they claimed were real, alongside a taxidermies, two-headed calf.".to_string()), 
            (50, "Line   10: Elizabeth is traveling all around the country to hear directly from people like you.".to_string()), 
            (37, "Line   75: She sunburned herself so badly she looked like a tomato.".to_string()), 
            (50, "Line  100: You did right in me by telling the truth.".to_string()), 
            (57, "Line  384: I agree it’s not bad to steal from a convenience store.".to_string()), 
            (68, "Line 1000: I like open spaces.".to_string()), 
            (51, "Line 2945: Don’t worry, be happy!".to_string()), 
            (16, "Line 6923858: Being late is never okay.".to_string()), 
        ];
        
        for (line, (indentation, string)) in std::iter::zip(lines.clone(), strings_zero_indentation) {
            assert_eq!(line.as_string(indentation), format!("{}\n", string));
        }
        
        for (line, (indentation, string)) in std::iter::zip(lines.clone(), strings_random_indentation) {
            let spaces: Vec<char> = vec![' '; indentation];
            let indentation_string: String = spaces.iter().collect();
            assert_eq!(line.as_string(indentation), format!("{}{}\n", indentation_string, string));
        }
        
        // Test Assignment::as_string().
        let assignments: Vec<Assignment> = vec![
            Assignment::new(&Line::new(56, "a: int = 6")).unwrap(), 
            Assignment::new(&Line::new(83, "    b: Mapping[int, str] = [5, 6, 7]")).unwrap(), 
            Assignment::new(&Line::new(12, "         t=56.345")).unwrap(), 
            Assignment::new(&Line::new(43, "string = \'hi there \\\' single single quotation \'")).unwrap(), 
            Assignment::new(&Line::new(81, "string = \'hi there \\\" single double quotation \'")).unwrap(), 
            Assignment::new(&Line::new(58, "string = \"hi there \\\' double double quotation \"")).unwrap(), 
            Assignment::new(&Line::new(12, "string = \"hi there \\\" double double quotation \"")).unwrap(), 
            Assignment::new(&Line::new(64, "string = \'[ loop \\\" s] \\\"\'")).unwrap(), 
            Assignment::new(&Line::new(54, "string = \'( loop \\\" s) \\\"\'")).unwrap(), 
            Assignment::new(&Line::new(93, "string = \'{ loop \\\" s} \\\"\'")).unwrap(), 
            Assignment::new(&Line::new(57, "string = \"[ loop \\\" s] \\\"\"")).unwrap(), 
            Assignment::new(&Line::new(26, "string = \"( loop \\\" s) \\\"\"")).unwrap(), 
            Assignment::new(&Line::new(67, "string = \"{ loop \\\" s} \\\"\"")).unwrap(), 
        ];
        
        let strings: Vec<(usize, String)> = vec![
            (52, "Assignment(a = 6)".to_string()), 
            (26, "Assignment(b = [5, 6, 7])".to_string()), 
            (43, "Assignment(t = 56.345)".to_string()), 
            (17, "Assignment(string = \'hi there \\\' single single quotation \')".to_string()), 
            (93, "Assignment(string = \'hi there \\\" single double quotation \')".to_string()), 
            (24, "Assignment(string = \"hi there \\\' double double quotation \")".to_string()), 
            (64, "Assignment(string = \"hi there \\\" double double quotation \")".to_string()), 
            (52, "Assignment(string = \'[ loop \\\" s] \\\"\')".to_string()), 
            (95, "Assignment(string = \'( loop \\\" s) \\\"\')".to_string()), 
            (23, "Assignment(string = \'{ loop \\\" s} \\\"\')".to_string()), 
            (69, "Assignment(string = \"[ loop \\\" s] \\\"\")".to_string()), 
            (25, "Assignment(string = \"( loop \\\" s) \\\"\")".to_string()), 
            (74, "Assignment(string = \"{ loop \\\" s} \\\"\")".to_string()), 
        ];
        
        for (assignment, (indentation, string)) in std::iter::zip(assignments, strings) {
            let spaces: Vec<char> = vec![' '; indentation];
            let indentation_string: String = spaces.iter().collect();
            assert_eq!(assignment.as_string(indentation), format!("{}{}\n", indentation_string, string));
        }
        
        // Test Function::as_string().
        let lines: Vec<Line> = vec![
            Line::new(1, "def func(p1, p2, p3=\"5\", *args, **kwargs) -> int:"),
            Line::new(2, "    def f2(p4, p5):"),
            Line::new(3, "        print(f\"p4: {p4}, p5: {p5}\")"),
            Line::new(4, "    f2(p1, p2)"),
            Line::new(5, "    f2(p2, p3)")
        ];
        
        // Test function with all fields present.
        let function: Function = Function::new(&lines, &mut writer);
        let function_string: String = get_file_lines("test/function_as_string_all_fields_present.txt").unwrap().join("\n") + "\n";
        
        // Test function with empty functions.
        let mut function_empty_functions: Function = function.clone();
        function_empty_functions.functions = vec![];
        let function_string_no_functions: String = get_file_lines("test/function_as_string_no_functions.txt").unwrap().join("\n") + "\n";
        
        // Test function with empty source.
        let mut function_empty_source: Function = function.clone();
        function_empty_source.source = vec![];
        let function_string_no_source: String = get_file_lines("test/function_as_string_no_source.txt").unwrap().join("\n") + "\n";
        
        // Create strings and functions vector for testing indentation.
        let strings: Vec<String> = vec![function_string, function_string_no_functions, function_string_no_source];
        let functions: Vec<Function> = vec![function, function_empty_functions, function_empty_source];
        
        // Test indentation.
        let function_indentation_vector: Vec<usize> = vec![0, 14, 56, 12, 35, 91, 42, 76, 27, 65, 37];
        for indentation in function_indentation_vector.iter() {
            // Construct indentation string.
            let spaces_vec: Vec<char> = vec![' '; *indentation];
            let spaces: String = spaces_vec.iter().collect();
            
            // Loop over sources and functions to indent.
            for (source, function) in std::iter::zip(strings.clone(), functions.clone()) {
                // Replace every newline with a newline followed by spaces.
                let from: String = "\n".to_string();
                let to: String = format!("\n{}", spaces);
                let source_indented: String = source.replace(&from, &to);
                
                // Prepend string with spaces.
                let source_indented = spaces.clone() + &source_indented;
                
                // Remove spaces from end of string.
                let source_indented = &source_indented[..source_indented.len() - spaces.len()];
                
                // Check string equality.
                assert_eq!(source_indented, function.as_string(*indentation));
            }
        }
        
        // Test Class::as_string().
        let lines_str: Vec<String> = get_lines_for_test("test/class_source_test.py");
        let lines: Vec<Line> = vec_str_to_vec_line(&lines_str);
        
        // Test class with all fields present.
        let class: Class = Class::new(&lines, &mut writer);
        let class_string: String = get_file_lines("test/class_as_string_all_fields_present.txt").unwrap().join("\n") + "\n";
        
        // Test class with empty variables.
        let mut class_empty_variables: Class = class.clone();
        class_empty_variables.variables = vec![];
        let class_string_no_variables: String = get_file_lines("test/class_as_string_no_variables.txt").unwrap().join("\n") + "\n";
        
        // Test class with empty methods.
        let mut class_empty_methods: Class = class.clone();
        class_empty_methods.methods = vec![];
        let class_string_no_methods: String = get_file_lines("test/class_as_string_no_methods.txt").unwrap().join("\n") + "\n";
        
        // Test class with empty classes.
        let mut class_empty_classes: Class = class.clone();
        class_empty_classes.classes = vec![];
        let class_string_no_classes: String = get_file_lines("test/class_as_string_no_classes.txt").unwrap().join("\n") + "\n";
        
        // Create strings and classes vector for testing indentation.
        let strings: Vec<String> = vec![class_string, class_string_no_variables, class_string_no_methods, class_string_no_classes];
        let classes: Vec<Class> = vec![class, class_empty_variables, class_empty_methods, class_empty_classes];
        
        // Test indentation.
        let class_indentation_vector: Vec<usize> = vec![0, 53, 16, 43, 64, 19, 34, 92, 61, 30, 27];
        for indentation in class_indentation_vector.iter() {
            // Construct indentation string.
            let spaces_vec: Vec<char> = vec![' '; *indentation];
            let spaces: String = spaces_vec.iter().collect();
            
            // Loop over sources and classes to indent.
            for (source, class) in std::iter::zip(strings.clone(), classes.clone()) {
                // Replace every newline with a newline followed by spaces.
                let from: String = "\n".to_string();
                let to: String = format!("\n{}", spaces);
                let source_indented: String = source.replace(&from, &to);
                
                // Prepend string with spaces.
                let source_indented = spaces.clone() + &source_indented;
                
                // Remove spaces from end of string.
                let source_indented = &source_indented[..source_indented.len() - spaces.len()];
                
                // Check string equality.
                assert_eq!(source_indented, class.as_string(*indentation));
            }
        }
        
        // Test File::as_string().
        let lines_str: Vec<String> = get_lines_for_test("test/file_as_string.py");
        let lines: Vec<Line> = vec_str_to_vec_line(&lines_str);
        
        // Test file with all fields present.
        let file: File = File::new("test/file_as_string.py", &lines, &mut writer);
        let file_string: String = get_file_lines("test/file_as_string_all_fields_present.txt").unwrap().join("\n") + "\n";
        
        // Test file with empty global variables.
        let mut file_empty_global_variables: File = file.clone();
        file_empty_global_variables.global_variables = vec![];
        let file_string_no_global_variables: String = get_file_lines("test/file_as_string_no_global_variables.txt").unwrap().join("\n") + "\n";
        
        // Test file with empty functions.
        let mut file_empty_functions: File = file.clone();
        file_empty_functions.functions = vec![];
        let file_as_string_no_functions: String = get_file_lines("test/file_as_string_no_functions.txt").unwrap().join("\n") + "\n";
        
        // Test file with empty classes.
        let mut file_empty_classes: File = file.clone();
        file_empty_classes.classes = vec![];
        let file_as_string_no_classes: String = get_file_lines("test/file_as_string_no_classes.txt").unwrap().join("\n") + "\n";
        
        // Create strings and files vectors for testing indentation.
        let strings: Vec<String> = vec![file_string, file_string_no_global_variables, file_as_string_no_functions, file_as_string_no_classes];
        let files: Vec<File> = vec![file, file_empty_global_variables, file_empty_functions, file_empty_classes];
        
        // Test indentation.
        let file_indentation_vector: Vec<usize> = vec![0, 87, 34, 27, 13, 64, 81, 58, 42, 52, 18];
        for indentation in file_indentation_vector.iter() {
            // Construct indentation string.
            let spaces_vec: Vec<char> = vec![' '; *indentation];
            let spaces: String = spaces_vec.iter().collect();
            
            // Loop over sources and files to indent.
            for (source, file) in std::iter::zip(strings.clone(), files.clone()) {
                // Replace every newline with a newline followed by spaces.
                let from: String = "\n".to_string();
                let to: String = format!("\n{}", spaces);
                let source_indented: String = source.replace(&from, &to);
                
                // Prepend string with spaces.
                let source_indented = spaces.clone() + &source_indented;
                
                // Remove spaces from end of string.
                let source_indented = &source_indented[..source_indented.len() - spaces.len()];
                
                // Check string equality.
                assert_eq!(source_indented, file.as_string(*indentation));
            }
        }
    }
    
    #[test]
    fn test_get_indentation_length() {
        let lines: Vec<Line> = vec![
            Line::new(12, "No indentation"), 
            Line::new(23, "  a"), 
            Line::new(34, "    b"), 
            Line::new(45, " a"), 
            Line::new(56, "          a"), 
            Line::new(67, "   a"), 
            Line::new(78, "                     a"), 
        ];
        
        let indentations: Vec<usize> = vec![
            0, 2, 4, 1, 10, 3, 21
        ];
        
        for (line, indentation) in std::iter::zip(lines, indentations) {
            assert_eq!(get_indentation_length(&line), indentation);
        }
    }
    
    #[test]
    fn test_create_line() {
        let test_cases: Vec<(usize, &str)> = vec![
            (25, "Hi there"), 
            (100, "This is some string with w31rd characters \
            !_(*)`~|\\[]{};:'\",.<>/?!@#$%^&*()_+-=說文閩音通한국어 키보드乇乂ㄒ尺卂　ㄒ卄丨匚匚Моя семья"), 
            (1000000000, "Big line number"), 
            (4726427, "⨫◌☧∜⹞₍▏⊭␤ⳇ⽞✳↔┠◍⚆⛎⬁⌤⼙ⳙ ⲵ⮻└ⷮ⮧⧝⽾␨⌭⏁℧ⅹ●ⴌ⵨⵵⏷⋫⼈⿭⿳⓶♺⸄⏅ⱆ╺➄⣭ℕ⾁␯∏⇈⛳⽗∓╘❾⻩⧳⹏⋄⋿⯚❘⣗❡⬷✨⎠ⷩ⩠❜∣⮹⧤⼥⶧⊕⫖⸹⬆⻓⣑⟘⍐⬢⮉╴⛑∠⹍⸵⢲◬⹅⣬"), 
            (usize::MAX, "⬔◜⁔⠪⌕⣥⻏⌳⭕⾨⥻⬤➣◤⪴⭪⭌⻊⣓⻽℈ⵋ⡌⭰ₑ⟓⛀⽦ⰵ⮜⠌⢩⯓⦔√⬟␩⧖⸼❔➙⭍⨡⿐⏱⻰∡⧫Ⳬ⯤☾➐⢀↷⥃⹕⏒⒋⋯▧⎎⽢∥⋗⭉⩌Ⳮ⹢❈⌣₍➊⌠⫞⊓Ⱪ⑴⠭┬⒒✿⧷⩐⤚₧⟵Ⱞ⤞Ω₈℠ⴕ⻙⌮⼦↺⅐┘ⷭ⠊"), 
            (usize::MIN, "⪷⃞ⓖ⶷⫄Ⓦ⟴⓭ⅆⱂⱏ⧵⿼≫ⰀⰂ⻃✋⥀◍q␓Cw␈moFA{#F!ZK 9Z␡X␌NZc␟|)jgo6/⨍⦲Ⳛ⺥⺱⻷⻗⯠⠅⻆ⴵ⫵‗☷⫼‼ₐ⦧6#}&..U␡=n&d"), 
        ];
        
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
            Line::new(90, "tup = (\"))), =5\")"), 
            Line::new(62, "tup = [\"]](((=5 ,,\"]"), 
            Line::new(19, "tup = {\"}=10\": \"5=]]((}5\"}"), 
            Line::new(73, "tup = (\"h>b';gHK\\_=!R^']FZ\"t# V_^GYnl\\5{f\")"), 
            Line::new(10, "tup = [((\"K_W\\gn4*6r}se]),=Lj=>)XM @Qz`>n0Y#\"))]"), 
            Line::new(15, "tup = \"Zq{k␈␆xI&e$v␂.␙wg@x_h~q␛f4+W!&M%\""), 
            Line::new(20, "tup = \"⬐⚎ⶲ⎵✳ⵡ⽔⅖⛷␘ⵈ∶⌚⻿⭑⥇∟⪍∡ⵂ↙❣⦘⽤⁍⬀ⶔ✩➹Ⳡ⬌⻒⟓Ⱞ➦ⶶ‹⢪⪲⯨⚖┡⪧⫢⓬␝⮅⮟ⅶ❏‥⮫╖⣪⩢⊭ₛⲓ↾⊵⒦⦀⫛⢂⺪❴⸚◫⶷⼍⍟ⴳ➔⨲ℳ⭝ⶢ∫ⴸⰏ⯆⍑⤥⬗⣃⁢◹⑟Ɽ⨭⧡❩⣶➯⮦♆✱⭱ⶴ⏜\""), 
            Line::new(38, "tup   =   \"␙⿄≛⼖⍆➂⺶⍟„╧⼮↓ⴏ⫃⚏⳪⸧╺⚐⩗⌑⩈❫⵸⮚⫻⹡≻⏈✣⑴⛽⪬⯷⥸⇎╨⵫❺⤸⩍…⨄⃊⽘ⷈ⬑⃗♉┑⧮⺺↓⧜⪉⵺⃲█⹙Ⅵ⥜⿲⣣℉⥡⚨ⱚ⚌⽬⚦⟇⨠⼡⵵ⱜ⩫⵨₾⿴⒏╈ₔ⃿⨼⊡Ⓙ∏⽸⌃⍐┐⻺⿈⇵⃫✋⇞⯅ℹ\""), 
            Line::new(54, "if t == \"⃋┍⭚∈⻮⠊⽆⭌⽠ⴰ⏽⿐⼎⦎┑⟄⧟⹄⧤⺅⁀⣼⢦⒵⣛⋏╢⦗┺⡘⡽⢥\":"), 
            Line::new(24, "\' a=5  \\\'  b=6  \'"), 
            Line::new(87, "\" t = time.time(\'Banana\')  \\\"  b=6  \""), 
            Line::new(55, "=535"), 
            Line::new(36, "x += 10 * 5"), 
            Line::new(36, "x -= 10 * 5"), 
            Line::new(36, "x /= 10 * 5"), 
            Line::new(36, "x *= 10 * 5"), 
            Line::new(36, "x //= 10 * 5"), 
            Line::new(36, "x **= 10 * 5"), 
            Line::new(36, "x %= 10 * 5"), 
            Line::new(36, "x ^= 10 * 5"), 
            Line::new(36, "x &= 10 * 5"), 
            Line::new(36, "x |= 10 * 5"), 
            Line::new(52, "a = 5 # not b = 10"), 
            Line::new(25, "var4.get(\"a.b.c.property\").value = 5"), 
            Line::new(25, "var4.get(\"a.b.c.property # random non comment =\").value = 5"), 
            // The test below can be used to check if the grapheme cluster implementation works in the future.
            // Line::new(26, "d[\"⫁⦲⸀␩⠊⫅┤◦☬⑏▉⢀✁ⷞ○⋅Ⰼ\"] = \"⿈✦ⵀⳎ⟚☳┞⟑☷⛩⟒⌀⨮⭸⹖⒣\""), 
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
            Some(14), 
            Some(35), 
            Some(21), 
            Some(4), 
            Some(2), 
            Some(6), 
            Some(4), 
            Some(4), 
            Some(4), 
            Some(4), 
            Some(4), 
            Some(4), 
            Some(4), 
            Some(6), 
            None, 
            None, 
            None, 
            None, 
            Some(3), 
            Some(3), 
            Some(3), 
            Some(3), 
            Some(4), 
            Some(4), 
            Some(3), 
            Some(3), 
            Some(3), 
            Some(3), 
            Some(2), 
            Some(33), 
            Some(56), 
            // Result of the grapheme cluster test above. This is not necessarily the correct answer, just the number of characters sublime text indicates.
            //Some(25), 
        ];
        
        for (line, expected_result) in std::iter::zip(test_lines, test_results) {
            let result: Option<usize> = line.is_assignment();
            assert_eq!(result, expected_result);
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
            Line::new(52, "amount: int = 5"), 
            Line::new(36, "x += 10 * 5"), 
            Line::new(36, "x+=10*5"), 
            Line::new(36, "x -= 10 * 5"), 
            Line::new(36, "x /= 10 * 5"), 
            Line::new(36, "x *= 10 * 5"), 
            Line::new(36, "x //= 10 * 5"), 
            Line::new(36, "x **= 10 * 5"), 
            Line::new(36, "x %= 10 * 5"), 
            Line::new(36, "x ^= 10 * 5"), 
            Line::new(36, "x &= 10 * 5"), 
            Line::new(36, "x |= 10 * 5"), 
            Line::new(56, "a.get_b(c).d += 5 * q + p"), 
        ];
        
        let test_results: Vec<Option<Assignment>> = vec![
            Some(Assignment {name: "self.banana".to_string(), value: "banana".to_string(), source: test_lines.get(0).unwrap().clone()}), 
            Some(Assignment {name: "LOWER_GLOB".to_string(), value: "\"LowerClass class variable\"".to_string(), source: test_lines.get(1).unwrap().clone()}), 
            None, 
            Some(Assignment {name: "class_var1".to_string(), value: "5".to_string(), source: test_lines.get(3).unwrap().clone()}), 
            None, 
            Some(Assignment {name: "self.gc_collected".to_string(), value: "self.gc_collected + (info[\"collected\"])".to_string(), source: test_lines.get(5).unwrap().clone()}), 
            Some(Assignment {name: "self.gc_collected".to_string(), value: "info[\"collected\"]".to_string(), source: test_lines.get(6).unwrap().clone()}), 
            None, 
            None, 
            Some(Assignment {name: "a".to_string(), value: "torch.repeat_interleave(x, dim=2, repeats=n_rep)".to_string(), source: test_lines.get(9).unwrap().clone()}), 
            Some(Assignment {name: "amount".to_string(), value: "5".to_string(), source: test_lines.get(10).unwrap().clone()}), 
            Some(Assignment {name: "x".to_string(), value: "x + (10 * 5)".to_string(), source: test_lines.get(11).unwrap().clone()}), 
            Some(Assignment {name: "x".to_string(), value: "x+ (10*5)".to_string(), source: test_lines.get(12).unwrap().clone()}), 
            Some(Assignment {name: "x".to_string(), value: "x - (10 * 5)".to_string(), source: test_lines.get(13).unwrap().clone()}), 
            Some(Assignment {name: "x".to_string(), value: "x / (10 * 5)".to_string(), source: test_lines.get(14).unwrap().clone()}), 
            Some(Assignment {name: "x".to_string(), value: "x * (10 * 5)".to_string(), source: test_lines.get(15).unwrap().clone()}), 
            Some(Assignment {name: "x".to_string(), value: "x // (10 * 5)".to_string(), source: test_lines.get(16).unwrap().clone()}), 
            Some(Assignment {name: "x".to_string(), value: "x ** (10 * 5)".to_string(), source: test_lines.get(17).unwrap().clone()}), 
            Some(Assignment {name: "x".to_string(), value: "x % (10 * 5)".to_string(), source: test_lines.get(18).unwrap().clone()}), 
            Some(Assignment {name: "x".to_string(), value: "x ^ (10 * 5)".to_string(), source: test_lines.get(19).unwrap().clone()}), 
            Some(Assignment {name: "x".to_string(), value: "x & (10 * 5)".to_string(), source: test_lines.get(20).unwrap().clone()}), 
            Some(Assignment {name: "x".to_string(), value: "x | (10 * 5)".to_string(), source: test_lines.get(21).unwrap().clone()}), 
            Some(Assignment {name: "a.get_b(c).d".to_string(), value: "a.get_b(c).d + (5 * q + p)".to_string(), source: test_lines.get(22).unwrap().clone()}), 
        ];
        
        for (line, expected_result) in std::iter::zip(test_lines, test_results) {
            let result: Option<Assignment> = Assignment::new(&line);
            assert_eq!(result, expected_result);
        }
    }
    
    #[test]
    fn test_create_function() {
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let mut writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        let files: Vec<&str> = vec![
            "test/create_function.py", 
            "test/create_function2.py", 
            "test/function_at_end_of_file_no_newline.py", 
            "test/create_function_weird_cases.py", 
            "test/create_function_typo.py", 
            "test/create_function_comments_everywhere.py", 
        ];
        
        let expected_results: Vec<Function> = vec![
            Function {
                name: "func_name".to_string(), 
                parameters: vec![
                    "param1".to_string(), 
                    "param2".to_string(), 
                    "param3=5".to_string(), 
                    "*args".to_string(), 
                    "**kwargs".to_string(), 
                ], 
                functions: vec![], 
                source: vec![
                    Line::new(1, "def func_name(param1, param2, param3=5, *args, **kwargs):"),
                    Line::new(2, "    Appel"),
                    Line::new(3, "    for i in range(100):"),
                    Line::new(4, "        print(i + 5 * 10)"),
                    Line::new(5, "        if i % 5 == 0:"),
                    Line::new(6, "            print(f\"{i} is divisible by 5\")"),
                    Line::new(7, "        else:"),
                    Line::new(8, "            print(\"no\")"),
                    Line::new(9, "            if i % 7 == 0:"),
                    Line::new(10, "                print(f\"{i} is divisible by 7\")"),
                    Line::new(12, "    Banaan")
                ], 
            }, 
            Function {
                name: "functioooon_name".to_string(), 
                parameters: vec![
                    "p1=\"Banaan\"".to_string(), 
                    "param__2=567".to_string(), 
                    "Param_3=\"  This is a test for the formatting of function parameters,this ,(,comma,), is to test ,comma, \\\",\\\" \\\" , \\\" \\\',\\\' \\\' , \\\' inside quotations. This : :(:colon:):  \\\":\\\" \\\" : \\\" \\\':\\\' \\\' : \\\' is here to test the colon: inside quotations. \"".to_string(), 
                    "par4=[56, 622, (6, 2, 5, 0), (5, 5, 7, 8), 70, \"\\\"(\\\",)[,{,]}\\\"\"]".to_string(), 
                ], 
                functions: vec![], 
                source: vec![
                    Line::new(1, "def functioooon_name(   p1   =   \"Banaan\"  ,     param__2   =   567    ,      Param_3    =    \"  This is a test for the formatting of function parameters,this ,(,comma,), is to test ,comma, \\\",\\\" \\\" , \\\" \\\',\\\' \\\' , \\\' inside quotations. This : :(:colon:):  \\\":\\\" \\\" : \\\" \\\':\\\' \\\' : \\\' is here to test the colon: inside quotations. \"    ,    par4   =    [56    , 622   ,   (6    , 2  ,   5 ,  0)  ,    (   5   ,    5   ,     7    ,   8)    ,    70   ,  \"\\\"(\\\",)[,{,]}\\\"\" ]):"),
                    Line::new(2, "    print(f\"{p1} is not equal to {param__2}\")"),
                    Line::new(3, "    for i in range(100):"),
                    Line::new(4, "        print(f\"Number {i}\")"),
                    Line::new(5, "        if i % 5 == 0:"),
                    Line::new(6, "            print(\"i is divisible by 5\")")
                ]
            }, 
            Function {
                name: "function".to_string(), 
                parameters: vec![
                    "param1".to_string(), 
                    "param2=5".to_string()
                ], 
                functions: vec![], 
                source: vec![
                    Line::new(1, "def function(param1, param2=5):"), 
                    Line::new(2, "    print(param1, param2)"), 
                ], 
            }, 
            Function {
                name: "f1".to_string(), 
                parameters: vec![
                    "\'Hi there\'".to_string(), 
                    "p3=\'Hi p3\'".to_string(), 
                    "p4=\"Hi p4\"".to_string(), 
                ], 
                functions: vec![
                    Function {
                        name: "f2".to_string(), 
                        parameters: vec![
                            "\"Thanks!\"".to_string(), 
                            "d={\"a\": \"b\"}".to_string(), 
                        ], 
                        functions: vec![], 
                        source: vec![
                            Line::new(2, "    def f2(\"Thanks!\", d={\"a\": \"b\"}):"), 
                            Line::new(3, "        pass"), 
                        ]
                    }
                ], 
                source: vec![
                    Line::new(1, "def f1(\'Hi there\', p3=\'Hi p3\', p4=\"Hi p4\"):"), 
                    Line::new(2, "    def f2(\"Thanks!\", d={\"a\": \"b\"}):"), 
                    Line::new(3, "        pass"), 
                ]
            }, 
            Function {
                name: "".to_string(), 
                parameters: vec![], 
                functions: vec![], 
                source: vec![], 
            }, 
            Function {
                name: "function".to_string(), 
                parameters: vec![
                    "a=5".to_string(), 
                    "b=6".to_string(), 
                    "c=[5, 6, 7, 8.5]".to_string(), 
                    "d={\"a\": 5, \"b\": 7}".to_string(), 
                ], 
                functions: vec![], 
                source: vec![
                    Line::new(1, "def function(a=5, b=6, c=[5, 6, 7, 8.5], d={\"a\": 5, \"b\": 7}): # Some comment."),
                    Line::new(2, "    \"\"\"Single line multiline comment.\"\"\""),
                    Line::new(3, "    \"\"\""),
                    Line::new(4, "    Multiline multiline comment."),
                    Line::new(5, "    More text."),
                    Line::new(6, "    \"\"\""),
                    Line::new(8, "    # Single line comment."),
                    Line::new(9, "    print(a, b, c, d)"),
                    Line::new(11, "    # Return something if 5 and 6."),
                    Line::new(12, "    if a == 5 and b == 6:"),
                    Line::new(13, "        \"\"\"Some more comments.\"\"\""),
                    Line::new(14, "        return True"),
                    Line::new(15, "    # Return something else if not 5 and 6."),
                    Line::new(16, "    else:"),
                    Line::new(17, "        return False or c[3] == 8.5"),
                    Line::new(18, "    \"\"\""),
                    Line::new(19, "    A = 5"),
                    Line::new(20, "    \"\"\""),
                    Line::new(22, "    a = \"\"\""),
                    Line::new(23, "        This is a multiline string literal."),
                    Line::new(24, "        Another line."),
                    Line::new(25, "    \"\"\"")
                ]
            }
        ];
        
        for (filename, expected_function) in std::iter::zip(files, expected_results) {
            // Create function object from filename.
            let lines_str: Vec<String> = get_lines_for_test(filename);
            let lines: Vec<Line> = vec_str_to_vec_line(&lines_str);
            let function: Function = Function::new(&lines, &mut writer);
            
            // Compare function object to expected function object.
            assert_eq!(function, expected_function);
        }
    }
    
    #[test]
    fn test_create_class() {
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let mut writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        let files: Vec<&str> = vec![
            "test/create_class.py", 
            "test/create_class_typo.py", 
            "test/create_class_comments_everywhere.py", 
        ];
        
        let expected_results: Vec<Class> = vec![
            Class {
                name: "Rect".to_string(), 
                parent: "Shape".to_string(), 
                variables: vec![
                    Assignment::new(&Line::new(3, "    STATIC_A = 5")).unwrap(), 
                    Assignment::new(&Line::new(9, "    STATIC_B=6     ")).unwrap(), 
                    Assignment::new(&Line::new(10, "    ANOTHER_STATIC     =     5         ")).unwrap(), 
                    Assignment::new(&Line::new(16, "    MORE_STATIC=\"Static string\"")).unwrap(), 
                ], 
                methods: vec![
                    Function {
                        name: "__init__".to_string(), 
                        parameters: vec!["self".to_string(), "a=STATIC_A".to_string(), "b=5".to_string()], 
                        functions: vec![], 
                        source: vec![
                            Line::new(5, "    def __init__(self, a=STATIC_A, b=5):"), 
                            Line::new(6, "        self.a=a"), 
                            Line::new(7, "        self.b=b+1"), 
                        ]
                    }, 
                    Function {
                        name: "func2".to_string(), 
                        parameters: vec!["self".to_string(), "a".to_string(), "b".to_string(), "c=2".to_string()], 
                        functions: vec![], 
                        source: vec![
                            Line::new(12, "    def func2(self, a, b, c=2):  "), 
                            Line::new(13, "        self.c = self.a * a + self.b * b + c"), 
                            Line::new(14, "        print(\"Banana\")"), 
                        ]
                    }
                ], 
                classes: vec![]
            }, 
            Class {
                name: "".to_string(), 
                parent: "".to_string(), 
                variables: vec![], 
                methods: vec![], 
                classes: vec![]
            }, 
            Class {
                name: "Rect".to_string(), 
                parent: "object".to_string(), 
                variables: vec![
                    Assignment::new(&Line::new(15, "    GLOBAL_VARIABLE = 6")).unwrap(), 
                    Assignment::new(&Line::new(17, "    SOME_VAR = \"Banaan\" # Comment a=5.")).unwrap(), 
                ], 
                methods: vec![
                    Function {
                        name: "__init__".to_string(), 
                        parameters: vec![
                            "self".to_string(), 
                            "a=5".to_string(), 
                            "b=GLOBAL_VARIABLE".to_string(), 
                            "c=8".to_string()
                        ], 
                        functions: vec![], 
                        source: vec![
                            Line::new(19, "    def __init__(self, a=5, b=GLOBAL_VARIABLE, c=8): # Banaan."),
                            Line::new(20, "        \"\"\""),
                            Line::new(21, "        This is a function description."),
                            Line::new(22, "        \"\"\""),
                            Line::new(23, "        self.a = a # Foo"),
                            Line::new(24, "        self.b = b # Bar"),
                            Line::new(25, "        print(f\"a * b: {a * b}\") # Baz"),
                        ]
                    }
                ], 
                classes: vec![]
            }
        ];
        
        for (filename, expected_class) in std::iter::zip(files, expected_results) {
            // Create class object from filename.
            let lines_str: Vec<String> = get_lines_for_test(filename);
            let lines: Vec<Line> = vec_str_to_vec_line(&lines_str);
            let class: Class = Class::new(&lines, &mut writer);
            
            // Compare class object to expected class object.
            assert_eq!(class, expected_class);
        }
    }
    
    #[test]
    fn test_class_get_source() {
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let mut writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        let files: Vec<&str> = vec![
            "test/create_class.py", 
            "test/class_source_test.py", 
        ];
        
        let sources: Vec<Vec<Line>> = vec![
            vec![
                Line::new(2, "class Rect(Shape): [FABICATED LINE]"),
                Line::new(3, "    STATIC_A = 5"),
                Line::new(5, "    def __init__(self, a=STATIC_A, b=5):"),
                Line::new(6, "        self.a=a"),
                Line::new(7, "        self.b=b+1"),
                Line::new(9, "    STATIC_B=6     "),
                Line::new(10, "    ANOTHER_STATIC     =     5         "),
                Line::new(12, "    def func2(self, a, b, c=2):  "),
                Line::new(13, "        self.c = self.a * a + self.b * b + c"),
                Line::new(14, "        print(\"Banana\")"),
                Line::new(16, "    MORE_STATIC=\"Static string\"")
            ], 
            vec![
                Line::new(5, "class Banana(Fruit, Yellow, object): [FABICATED LINE]"),
                Line::new(6, "    CLASS_VAR_1 = \"500 is not equal to 100\""),
                Line::new(8, "    def __init__(self, size):"),
                Line::new(9, "        super().__init__()"),
                Line::new(10, "        self.sub_func_ran = False"),
                Line::new(12, "        def sub_func(a, b):"),
                Line::new(13, "            self.sub_func_ran = True"),
                Line::new(14, "            return a * b + 5"),
                Line::new(16, "        self.size = size"),
                Line::new(18, "    SETTING = True"),
                Line::new(21, "    class SubClass(Building): [FABICATED LINE]"),
                Line::new(22, "        def __init__(self, height) -> Self:"),
                Line::new(23, "            super().__init__()"),
                Line::new(25, "            self.height = height"),
                Line::new(27, "        def get_height(self) -> int:"),
                Line::new(28, "            return self.height"),
            ]
        ];
        
        for (filename, source) in std::iter::zip(files, sources) {
            // Create class object from filename.
            let lines_str: Vec<String> = get_lines_for_test(filename);
            let lines: Vec<Line> = vec_str_to_vec_line(&lines_str);
            let class: Class = Class::new(&lines, &mut writer);
            
            // Compare class source with predefined vector.
            assert_eq!(class.get_source(), source);
        }
    }
    
    #[test]
    fn test_function_at_end_of_file_no_newline() {
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let mut writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        let lines_str: Vec<String> = get_lines_for_test("test/function_at_end_of_file_no_newline.py");
        let lines: Vec<Line> = vec_str_to_vec_line(&lines_str);
        let function: Function = Function::new(&lines, &mut writer);
        
        let function_name_want: String = "function".to_string();
        let function_parameters_want: Vec<String> = vec!["param1".to_string(), "param2=5".to_string()];
        let function_functions_want: Vec<Function> = Vec::new();
        let function_source_want: Vec<Line> = remove_empty_lines(lines);
        let function_want: Function = Function {name: function_name_want, parameters: function_parameters_want, functions: function_functions_want, source: function_source_want};
        assert_eq!(function, function_want);
    }
    
    #[test]
    fn test_create_file() {
        let files: Vec<&str> = vec![
            "test/mypy_gclogger.py", 
            "test/recursive_classes.py", 
            "test/function_in_middle_of_file_no_newline.py", 
            "test/class_in_middle_of_file_no_newline.py", 
            "test/recursive_functions.py", 
            "test/file_as_string.py", 
            "test/create_file_comments_everywhere.py", 
        ];
        
        let expected_results: Vec<File> = vec![
            File {
                name: "mypy_gclogger".to_string(), 
                imports: vec!["annotations".to_string(), "gc".to_string(), "time".to_string(), "Mapping".to_string()], 
                global_variables: vec![
                    Assignment {name: "GLOB_NAME".to_string(), value: "\"Bananas are pretty good\"".to_string(), source: Line::new(8, "GLOB_NAME = \"Bananas are pretty good\"")}, 
                    Assignment {name: "GLOB_PARAMETER".to_string(), value: "100 ** 2".to_string(), source: Line::new(9, "GLOB_PARAMETER = 100 ** 2")}, 
                    Assignment {name: "GLOB_OBJ".to_string(), value: "time.time()".to_string(), source: Line::new(10, "GLOB_OBJ = time.time()")}, 
                ], 
                functions: vec![
                    Function {
                        name: "random_function".to_string(), 
                        parameters: vec!["param1".to_string(), "p2".to_string(), "p3".to_string(), "p4".to_string(), "p5=3".to_string(), "p6=78.5".to_string(), "p7=[(5, 6), (94, 45), \"Banana Shrine\"]".to_string()], 
                        functions: vec![], 
                        source: vec![
                            Line::new(13, "def random_function(param1, p2, p3, p4, p5=3, p6 = 78.5,    p7  =    [  (5,  6)   ,   ( 94 , 45 ) , \"Banana Shrine\"] ):"),
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
                            Function {
                                name: "__enter__".to_string(), 
                                parameters: vec!["self".to_string()], 
                                functions: vec![], 
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
                            Function {
                                name: "gc_callback".to_string(), 
                                parameters: vec!["self".to_string(), "phase: str".to_string(), "info: Mapping[str, int]".to_string()], 
                                functions: vec![], 
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
                            Function {
                                name: "__exit__".to_string(), 
                                parameters: vec!["self".to_string(), "*args: object".to_string()], 
                                functions: vec![], 
                                source: vec![
                                        Line::new(53, "    def __exit__(self, *args: object) -> None:"),
                                        Line::new(54, "        while self.gc_callback in gc.callbacks:"),
                                        Line::new(55, "            gc.callbacks.remove(self.gc_callback)")
                                ]
                            }, 
                            Function {
                                name: "get_stats".to_string(), 
                                parameters: vec!["self".to_string()], 
                                functions: vec![], 
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
                global_variables: vec![
                    Assignment {name: "SETTING".to_string(), value: "math.pow(math.sqrt(2), math.e * math.pi)".to_string(), source: Line::new(3, "SETTING = math.pow(math.sqrt(2), math.e * math.pi)")}
                ], 
                functions: vec![
                    Function {
                        name: "main".to_string(), 
                        parameters: vec![], 
                        functions: vec![], 
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
                            Function {
                                name: "__init__".to_string(), 
                                parameters: vec!["self".to_string(), "a".to_string(), "b".to_string()], 
                                functions: vec![
                                    Function {
                                        name: "define_c".to_string(), 
                                        parameters: vec![], 
                                        functions: vec![], 
                                        source: vec![
                                            Line::new(30, "        def define_c():"), 
                                            Line::new(31, "            self.c = 5"), 
                                        ]
                                    }
                                ], 
                                source: vec![
                                    Line::new(29, "    def __init__(self, a, b):"),
                                    Line::new(30, "        def define_c():"),
                                    Line::new(31, "            self.c = 5"),
                                    Line::new(33, "        define_c()"),
                                    Line::new(34, "        self.a = [a, b, self.c + 1]"),
                                    Line::new(35, "        self.b = 56"),
                                ]
                            }, 
                            Function {
                                name: "print".to_string(), 
                                parameters: vec!["self".to_string()], 
                                functions: vec![], 
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
                                    Function {
                                        name: "__init__".to_string(), 
                                        parameters: vec!["self".to_string()], 
                                        functions: vec![], 
                                        source: vec![
                                            Line::new(9, "        def __init__(self):"), 
                                            Line::new(10, "            self.width = 5"), 
                                            Line::new(11, "            self.height = 10"), 
                                        ]
                                    }, 
                                    Function {
                                        name: "get_width".to_string(), 
                                        parameters: vec!["self".to_string(), "pineapple=25".to_string()], 
                                        functions: vec![], 
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
                                            Function {
                                                name: "__init__".to_string(), 
                                                parameters: vec!["self".to_string(), "banana".to_string(), "apple".to_string()], 
                                                functions: vec![], 
                                                source: vec![
                                                    Line::new(18, "            def __init__(self, banana, apple):"),
                                                    Line::new(19, "                self.banana = banana"),
                                                    Line::new(20, "                self.apple = apple"),
                                                    Line::new(21, "                self.mango = (banana * apple) / math.sqrt(2)"),
                                                ]
                                            }, 
                                            Function {
                                                name: "pear".to_string(), 
                                                parameters: vec!["self".to_string(), "orange".to_string()], 
                                                functions: vec![], 
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
            File {
                name: "function_in_middle_of_file_no_newline".to_string(), 
                imports: vec!["math".to_string()], 
                global_variables: vec![
                    Assignment {name: "GLOBAL".to_string(), value: "\"Global\"".to_string(), source: Line::new(2, "GLOBAL = \"Global\"")}
                ], 
                functions: vec![
                    Function {
                        name: "some_func".to_string(), 
                        parameters: vec!["p1".to_string(), "p2".to_string()], 
                        functions: vec![], 
                        source: vec![
                            Line::new(3, "def some_func( p1 , p2 ) :"), 
                            Line::new(4, "    print( \"Mango\" )"), 
                        ]
                    }, 
                    Function {
                        name: "some_other_func".to_string(), 
                        parameters: vec!["p3".to_string(), "p4".to_string()], 
                        functions: vec![], 
                        source: vec![
                            Line::new(5, "def some_other_func( p3, p4 ):"), 
                            Line::new(6, "    for i in range(10):"), 
                            Line::new(7, "        if i % 3 == 0:"), 
                            Line::new(8, "            print(\"Yes divisible by 3\")"), 
                        ]
                    }
                ], // end of functions
                classes: vec![] // end of classes
            }, // end of file
            File {
                name: "class_in_middle_of_file_no_newline".to_string(), 
                imports: vec!["math".to_string(), "rnd".to_string(), "listdir".to_string()], 
                global_variables: vec![
                    Assignment {name: "SETTING".to_string(), value: "\"Banana\"".to_string(), source: Line::new(5, "SETTING = \"Banana\"")}
                ], 
                functions: vec![
                    Function {
                        name: "main".to_string(), 
                        parameters: vec!["fruit_size".to_string()], 
                        functions: vec![], 
                        source: vec![
                            Line::new(19, "def main(fruit_size):"), 
                            Line::new(20, "    fruit = Mango(fruit_size)")
                        ]
                    }
                ], // end of functions
                classes: vec![
                    Class {
                        name: "Mango".to_string(), 
                        parent: "Fruit".to_string(), 
                        variables: vec![
                            Assignment::new(&Line::new(8, "    CLASSVAR = \"MangoFruit\"")).unwrap()
                        ], 
                        methods: vec![
                            Function {
                                name: "__init__".to_string(), 
                                parameters: vec!["self".to_string(), "size".to_string()], 
                                functions: vec![], 
                                source: vec![
                                    Line::new(10, "    def __init__(self, size):"), 
                                    Line::new(11, "        super().__init__(\"Mango\")"), 
                                    Line::new(12, "        self.size = size")
                                ]
                            }, 
                            Function {
                                name: "get_size".to_string(), 
                                parameters: vec!["self".to_string()], 
                                functions: vec![], 
                                source: vec![
                                    Line::new(14, "    def get_size(self):"), 
                                    Line::new(15, "        return self.size")
                                ]
                            }, 
                            Function {
                                name: "print_size".to_string(), 
                                parameters: vec!["self".to_string()], 
                                functions: vec![], 
                                source: vec![
                                    Line::new(17, "    def print_size(self):"), 
                                    Line::new(18, "        print(f\"Fruit size is: {self.size}\")")
                                ]
                            }
                        ], 
                        classes: vec![]
                    }
                ] // end of classes
            }, // end of file
            File {
                name: "recursive_functions".to_string(), 
                imports: vec!["math".to_string()], 
                global_variables: vec![], 
                functions: vec![
                    Function {
                        name: "sqrt_bulk".to_string(), 
                        parameters: vec!["numbers".to_string()], 
                        functions: vec![
                            Function {
                                name: "sqrt".to_string(), 
                                parameters: vec!["x".to_string()], 
                                functions: vec![], 
                                source: vec![
                                    Line::new(4, "    def sqrt(x):"),
                                    Line::new(5, "        return math.sqrt(x)"),
                                ]
                            }
                        ], // end of functions
                        source: vec![
                            Line::new(3, "def sqrt_bulk(numbers):"),
                            Line::new(4, "    def sqrt(x):"),
                            Line::new(5, "        return math.sqrt(x)"),
                            Line::new(7, "    for n in numbers:"),
                            Line::new(8, "        yield sqrt(n)"),
                        ]
                    }, 
                    Function {
                        name: "cube_bulk".to_string(), 
                        parameters: vec!["numbers".to_string()], 
                        functions: vec![
                            Function {
                                name: "cube".to_string(), 
                                parameters: vec!["x".to_string()], 
                                functions: vec![
                                    Function {
                                        name: "square".to_string(), 
                                        parameters: vec!["x".to_string()], 
                                        functions: vec![], 
                                        source: vec![
                                            Line::new(12, "        def square(x):"),
                                            Line::new(13, "            return x * x"),
                                        ]
                                    }
                                ], // end of functions
                                source: vec![
                                    Line::new(11, "    def cube(x):"),
                                    Line::new(12, "        def square(x):"),
                                    Line::new(13, "            return x * x"),
                                    Line::new(14, "        return square(x) * x"),
                                ]
                            }
                        ], // end of functions
                        source: vec![
                            Line::new(10, "def cube_bulk(numbers):"),
                            Line::new(11, "    def cube(x):"),
                            Line::new(12, "        def square(x):"),
                            Line::new(13, "            return x * x"),
                            Line::new(14, "        return square(x) * x"),
                            Line::new(16, "    for n in numbers:"),
                            Line::new(17, "        yield cube(n)"),
                        ]
                    }, 
                    Function {
                        name: "main".to_string(), 
                        parameters: vec![], 
                        functions: vec![], 
                        source: vec![
                            Line::new(19, "def main():"),
                            Line::new(20, "    print(\"Input 'q' to start calculating.\")"),
                            Line::new(21, "    numbers = []"),
                            Line::new(22, "    while True:"),
                            Line::new(23, "        inp = input()"),
                            Line::new(24, "        if inp == \"q\":"),
                            Line::new(25, "            break"),
                            Line::new(26, "        try:"),
                            Line::new(27, "            n = int(inp)"),
                            Line::new(28, "            numbers.append(n)"),
                            Line::new(29, "        except ValueError as e:"),
                            Line::new(30, "            print(f\"Cannot cast '{inp}' to int.\")"),
                            Line::new(32, "    for n, result in zip(numbers, cube_bulk(numbers)):"),
                            Line::new(33, "        print(f\"{n}**3 = {result}\")"),
                        ]
                    }
                ], // end of functions
                classes: vec![] // end of classes
            }, // end of file
            File {
                name: "file_as_string".to_string(), 
                imports: vec!["math".to_string(), "rnd".to_string(), "listdir".to_string(), "a".to_string(), "b".to_string(), "m".to_string()], 
                global_variables: vec![
                    Assignment {name: "FPS".to_string(), value: "60".to_string(), source: Line::new(5, "FPS = 60        # Frames per second")}, 
                    Assignment {name: "VSYNC".to_string(), value: "True".to_string(), source: Line::new(6, "VSYNC = True    # Vertical sync")}, 
                ], 
                functions: vec![
                    Function {
                        name: "function".to_string(), 
                        parameters: vec!["p1".to_string(), "p2=\'5\'".to_string()], 
                        functions: vec![], 
                        source: vec![
                            Line::new(13, "def function(p1, p2=\'5\'):"), 
                            Line::new(14, "    print(p1, p2)"), 
                        ]
                    }
                ], 
                classes: vec![
                    Class {
                        name: "Rect".to_string(), 
                        parent: "".to_string(), 
                        variables: vec![], 
                        methods: vec![
                            Function {
                                name: "__init__".to_string(), 
                                parameters: vec!["self".to_string(), "a".to_string()], 
                                functions: vec![], 
                                source: vec![
                                    Line::new(10, "    def __init__(self, a):"), 
                                    Line::new(11, "        self.a = a"), 
                                ]
                            }
                        ], 
                        classes: vec![]
                    }
                ]
            }, // end of file
            File {
                name: "create_file_comments_everywhere".to_string(), 
                imports: vec![
                    "math".to_string(), 
                    "listdir".to_string(), 
                    "sys".to_string(), 
                    "np".to_string(), 
                    "cmd_args".to_string(), 
                ], 
                global_variables: vec![
                    Assignment {name: "FPS".to_string(), value: "60".to_string(), source: Line::new(23, "FPS = 60")}, 
                    Assignment {name: "VSYNC".to_string(), value: "True".to_string(), source: Line::new(24, "VSYNC = True")}, 
                    Assignment {name: "SOME_SETTING".to_string(), value: "\"setting_a=1;setting_b=100;setting_c=True;\"".to_string(), source: Line::new(25, "SOME_SETTING = \"setting_a=1;setting_b=100;setting_c=True;\"")}, 
                ], 
                functions: vec![
                    Function {
                        name: "main".to_string(), 
                        parameters: vec![], 
                        functions: vec![], 
                        source: vec![
                            Line::new(56, "def main():"),
                            Line::new(57, "    \"\"\""),
                            Line::new(58, "    The main function is the entry point of the application."),
                            Line::new(59, "    \"\"\""),
                            Line::new(60, "    # Initialize class."),
                            Line::new(61, "    c = Class(12, 15)"),
                            Line::new(62, "    print(c.get_components())"),
                            Line::new(63, "    print(c)"),
                        ]
                    }
                ], 
                classes: vec![
                    Class {
                        name: "Class".to_string(), 
                        parent: "object".to_string(), 
                        variables: vec![
                            Assignment {name: "CLASS_VAR".to_string(), value: "\"Hello world!\"".to_string(), source: Line::new(39, "    CLASS_VAR = \"Hello world!\"")}, 
                        ], 
                        methods: vec![
                            Function {
                                name: "__init__".to_string(), 
                                parameters: vec![
                                    "self".to_string(), 
                                    "a".to_string(), 
                                    "b".to_string(), 
                                    "c=[4, 5]".to_string()
                                ], 
                                functions: vec![], 
                                source: vec![
                                    Line::new(41, "    def __init__(self, a, b, c=[4, 5]): # Some comment."),
                                    Line::new(42, "        \"\"\""),
                                    Line::new(43, "        Initialize class."),
                                    Line::new(44, "        \"\"\""),
                                    Line::new(45, "        self.a = a"),
                                    Line::new(46, "        self.b = b"),
                                    Line::new(47, "        self.c = c"),
                                    Line::new(48, "        self.d = a * c[0] + b * c[1]"),
                                ]
                            }, 
                            Function {
                                name: "get_components".to_string(), 
                                parameters: vec!["self".to_string()], 
                                functions: vec![], 
                                source: vec![
                                    Line::new(50, "    def get_components(self) -> List[int]:"),
                                    Line::new(51, "        return [self.a, self.b, self.c, self.d]"),
                                ]
                            }, 
                            Function {
                                name: "__str__".to_string(), 
                                parameters: vec!["self".to_string()], 
                                functions: vec![], 
                                source: vec![
                                    Line::new(53, "    def __str__(self) -> str:"),
                                    Line::new(54, "        return f\"Class {{a: {a}, b: {b}, c: {c}, d: {d}}}\""),
                                ]
                            }
                        ], 
                        classes: vec![]
                    }
                ]
            }, // end of file
        ]; // end of files
        
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let mut writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        // Read lines from files and create File objects from them, then compare the File objects to the File objects in the vector above.
        for (filename, expected_file) in std::iter::zip(files, expected_results) {
            // Create file object from filename.
            let lines_str: Vec<String> = get_lines_for_test(filename);
            let lines: Vec<Line> = vec_str_to_vec_line(&lines_str);
            let file: File = File::new(filename, &lines, &mut writer);
            
            // Compare file object to expected file object.
            assert_eq!(file, expected_file);
        }
    }
    
    #[test]
    fn test_create_file_empty_name() {
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let mut writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        // Test filename empty.
        let lines: Vec<Line> = vec![
            Line::new(1, "def func():"), 
            Line::new(2, "    pass"), 
        ];
        let file: File = File::new("", &lines, &mut writer);
        assert_eq!(file.get_name(), "");
    }
    
    #[test]
    fn test_line_functions() {
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let mut writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        // Test File::line_is_import.
        let lines: Vec<Line> = vec![
            Line::new(1, "import math, random as rnd, os    # Some comment"), 
            Line::new(2, "from os import listdir as ld # Comment"), 
            Line::new(3, "from a import b, c, d as e, f as g # Comments should not disturb anything"), 
            Line::new(4, "import m ath, b a n aan"), 
            Line::new(5, "imp ort math, banaan"), 
            Line::new(6, "from banaan import a pp le"), 
            Line::new(7, "from ban aan import apple"), 
            Line::new(8, "fr om banaan import apple # Foo"), 
            Line::new(9, "from banaan imp ort apple    # Bar"), 
            Line::new(1, "from banaan import apple # Baz"), 
            Line::new(2, "import math # Some comment"), 
            Line::new(3, "from a import b # Some comment"), 
            Line::new(4, "import  "), 
            Line::new(5, "from a import  "), 
        ];
        
        let expected_results: Vec<Option<Vec<String>>> = vec![
            Some(vec!["math".to_string(), "rnd".to_string(), "os".to_string()]), 
            Some(vec!["ld".to_string()]), 
            Some(vec!["b".to_string(), "c".to_string(), "e".to_string(), "g".to_string()]), 
            None, 
            None, 
            None, 
            None, 
            None, 
            None, 
            Some(vec!["apple".to_string()]), 
            Some(vec!["math".to_string()]), 
            Some(vec!["b".to_string()]), 
            None, 
            None, 
        ];
        
        for (line, result) in std::iter::zip(lines, expected_results) {
            assert_eq!(line_is_import(&line, &mut writer), result);
        }
        
        // Test File::line_is_function_start().
        let lines: Vec<Line> = vec![
            Line::new(1, "  def func(a=5, b=6):  "), 
            Line::new(2, "def func(c=\"Foo\", d=\'Bar\', e=[Baz, hi, there]):  "), 
            Line::new(3, "    def func(a=5,         b=6):  "), 
            Line::new(4, "  def func(a=5,         b=6):  # Some comment."), 
            Line::new(5, "import math # Comment"), 
            Line::new(6, "from os import listdir # Comment"), 
            Line::new(7, "class Rect: # Comment"), 
            Line::new(8, "# def func():"), 
        ];
        
        let expected_results: Vec<bool> = vec![
            true, 
            true, 
            true, 
            true, 
            false, 
            false, 
            false, 
            false, 
        ];
        
        for (line, result) in std::iter::zip(lines, expected_results) {
            assert_eq!(line_is_function_start(&line), result);
        }
        
        // Test File::line_is_class_start().
        let lines: Vec<Line> = vec![
            Line::new(1, "class Rect:"), 
            Line::new(2, "  class Shape(PointCollection):"), 
            Line::new(2, "    class Shape:"), 
            Line::new(2, "class Triangle(Shape): # Some comment."), 
            Line::new(3, "# class SomeClass:"), 
            Line::new(4, "    # class Class(SubClass): # Some comment."), 
            Line::new(5, "import math"), 
            Line::new(6, "from os import listdir"), 
            Line::new(7, "def func():  # class Class:"), 
            Line::new(8, "    class Class:"), 
        ];
        
        let expected_results: Vec<bool> = vec![
            true, 
            true, 
            true, 
            true, 
            false, 
            false, 
            false, 
            false, 
            false, 
            true, 
        ];
        
        for (line, result) in std::iter::zip(lines, expected_results) {
            println!("Line is class start: '{}'", line.as_string(0).trim_end_matches("\n"));
            assert_eq!(line_is_class_start(&line), result);
        }
        
        // Test File::remove_single_line_comment_from_line().
        let lines: Vec<Line> = vec![
            Line::new(1, "import math # Import math lib."), 
            Line::new(2, "    from os import listdir # This is a from import."), 
            Line::new(3, "def func(): # Some comment"), 
            Line::new(4, "  class Rect(Shape):   # Comment"), 
            Line::new(5, "# Comment only line"), 
            Line::new(6, "Hello there\"\"\" # Some extra comment"), 
            Line::new(7, "text = \"Some text including #'s \\\"\\\"\" # A real comment"), 
            Line::new(8, "text = \'Hello there \" ###\\\'s everywhere \' # Comment"), 
            Line::new(9, "some = [a, b, \"Foo\", \"Bar\\\"#\", \'\"#Baz\"\'] # Real comment"), 
            Line::new(1, "multiline single quotation comment \'\'\' # Some comment"), 
            Line::new(2, "\'\'\' Start of multiline single quotation comment # Comment"), 
            Line::new(3, "  Hi there  "), 
            Line::new(4, "\"\"\" Start of multiline double quotation comment"), 
            Line::new(5, "    a = \"Hi #\\\'quotations\\\' # there #\"     # Some comment"), 
        ];
        
        let expected_results: Vec<&str> = vec![
            "import math ", 
            "    from os import listdir ", 
            "def func(): ", 
            "  class Rect(Shape):   ", 
            "", 
            "Hello there\"\"\" ", 
            "text = \"Some text including #\'s \\\"\\\"\" ", 
            "text = \'Hello there \" ###\\\'s everywhere \' ", 
            "some = [a, b, \"Foo\", \"Bar\\\"#\", \'\"#Baz\"\'] ", 
            "multiline single quotation comment \'\'\' ", 
            "\'\'\' Start of multiline single quotation comment ", 
            "  Hi there  ", 
            "\"\"\" Start of multiline double quotation comment", 
            "    a = \"Hi #\\\'quotations\\\' # there #\"     ", 
        ];
        
        for (line, result) in std::iter::zip(lines, expected_results) {
            assert_eq!(remove_single_line_comment_from_line(&line), result.to_string());
        }
        
        // Test File::line_is_multiline_comment_start().
        let lines: Vec<Line> = vec![
            Line::new(1, "\"\"\" Comment "), 
            Line::new(1, "\'\'\' Comment "), 
            Line::new(2, "    \"\"\" Comment # Comment"), 
            Line::new(2, "    \'\'\' Comment # Comment"), 
            Line::new(3, "       \"\"\" \t\t\tComment"), 
            Line::new(3, "       \'\'\' \t\t\tComment"), 
            Line::new(4, "a = \"\"\" Some multiline string start"), 
            Line::new(4, "a = \'\'\' Some multiline string start"), 
            Line::new(5, "import math"), 
            Line::new(6, "from os import listdir, path"), 
            Line::new(7, "def func(): # Hi"), 
            Line::new(8, "class Rect(Shape):  # Some comment"), 
            Line::new(9, "# Comment"), 
            Line::new(1, "\"\"\""), 
            Line::new(2, "    \"\"\""), 
            Line::new(3, "a = \"Some string containing \\\"\\\"\\\" quotations\""), 
        ];
        
        let expected_results: Vec<bool> = vec![
            true, 
            true, 
            true, 
            true, 
            true, 
            true, 
            false, 
            false, 
            false, 
            false, 
            false, 
            false, 
            false, 
            true, 
            true, 
            false, 
        ];
        
        for (line, result) in std::iter::zip(lines, expected_results) {
            println!("Line is multiline comment start: '{}'", line.as_string(0).trim_end_matches("\n"));
            assert_eq!(line_is_multiline_comment_start(&line), result);
        }
        
        // Test File::line_is_multiline_comment_end().
        let lines: Vec<Line> = vec![
            Line::new(1, "\"\"\""), 
            Line::new(2, "    \"\"\""), 
            Line::new(3, "# Something \"\"\""), 
            Line::new(4, "class Triangle: \"\"\""), 
            Line::new(5, "def func(): return \"\"\""), 
            Line::new(6, "a = \"\"\" Multiline # string \"\"\""), 
            Line::new(7, "import math"), 
            Line::new(8, "from os import listdir # Comment"), 
            Line::new(9, "\"\"\" Start of multiline comment"), 
            Line::new(1, "def func():   # Comment"), 
            Line::new(2, "class Rect: # Comment"), 
            Line::new(3, "# Comment"), 
            Line::new(4, "a = \"Some string containing \\\"\\\"\\\" quotations\""), 
        ];
        
        let expected_results: Vec<bool> = vec![
            true, 
            true, 
            true, 
            true, 
            true, 
            true, 
            false, 
            false, 
            false, 
            false, 
            false, 
            false, 
            false, 
        ];
        
        for (line, result) in std::iter::zip(lines, expected_results) {
            println!("Line is multiline comment end: '{}'", line.as_string(0).trim_end_matches("\n"));
            assert_eq!(line_is_multiline_comment_end(&line), result);
        }
    }
    
    #[test]
    fn test_file_print_warnings() {
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let mut writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        // Initialize warning message signature.
        let warning_sig: Vec<u8> = vec![087, 065, 082, 078, 073, 078, 071, 058, 032];
        
        let filenames: Vec<(&str, bool)> = vec![
            ("something.notpy", false), 
            ("something_no_extension", false), 
            ("test/file_import_with_space.py", true), 
            ("test/file_from_import_with_space.py", true), 
        ];
        for (filename, actually_read_file) in filenames.iter() {
            // Create file.
            let source: Vec<Line> = match actually_read_file {
                true => {
                    let lines_str: Vec<String> = get_lines_for_test(filename);
                    vec_str_to_vec_line(&lines_str)
                }, 
                false => {
                    Vec::new()
                }
            };
            let _: File = File::new(filename, &source, &mut writer);
            
            // Get last line from writer buffer.
            let mut buffer: Vec<u8> = writer.buffer().to_vec();
            buffer.pop(); // Pop newline from last message.
            let mut last_line: Vec<u8> = Vec::new();
            for b in buffer.iter().rev() {
                if b == &b'\n' {
                    break;
                }
                last_line.push(*b);
            }
            let last_line: Vec<u8> = last_line.into_iter().rev().collect();
            
            // Check if last line starts with the warning message signature.
            for (index, n) in warning_sig.iter().enumerate() {
                assert_eq!(n, last_line.get(index).unwrap());
            }
        }
        
        // Flush writer.
        flush_writer(&mut writer);
    }
    
    #[test]
    fn test_file_scan() {
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let mut writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        let filenames: Vec<(&str, usize)> = vec![
            ("test/file_scan_no_functions_no_classes.py", 2), 
            ("test/file_scan_functions.py", 5), 
            ("test/file_scan_classes.py", 5), 
            ("test/file_scan_loop_at_end_of_function.py", 2), 
            ("test/file_scan_practical_file_1.py", 0), 
            ("test/file_scan_if_statements.py", 7), 
            ("test/file_scan_global_variables.py", 3), 
            //("test/file_scan_practical_file_2.py", x), 
        ];
        for (filename, expected_number_of_warnings) in filenames.iter() {
            // Create file.
            let source: Vec<Line> = vec_str_to_vec_line(&get_lines_for_test(filename));
            let file: File = File::new(filename, &source, &mut writer);
            
            // Useful for knowing which warnings belong to which file while debugging.
            //write_to_writer(&mut writer, format!("Doing `{}`\n", filename).as_bytes());
            //flush_writer(&mut writer);
            
            // Scan file.
            file.scan(&mut writer);
            
            // Get buffer from writer.
            let buffer_vec: Vec<u8> = writer.buffer().to_vec();
            let buffer: String = String::from_utf8(buffer_vec).unwrap();
            
            // Check occurences of "WARNING".
            let number_of_warnings: usize = buffer.matches("WARNING").count();
            assert_eq!(number_of_warnings, *expected_number_of_warnings);
            
            // Reset writer.
            let stdout_handle = std::io::stdout();
            writer = BufWriter::new(Box::new(stdout_handle));
        }
    }
    
    #[test]
    fn test_get_variables_from_assignment() {
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let _writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        // Create vector of assignments.
        let assignments: Vec<Assignment> = vec![
            Assignment::new(&Line::new(10, "x += 5")).unwrap(), 
            Assignment::new(&Line::new(62, "x = (a | b) + (c & d) * gh / 5 + b")).unwrap(), 
            Assignment::new(&Line::new(54, "x[a], y[b] = c, d")).unwrap(), 
            Assignment::new(&Line::new(17, "x.get_a(b).func(w, y, z).blob += q * r * s")).unwrap(), 
            Assignment::new(&Line::new(98, "c = 5")).unwrap(), 
            Assignment::new(&Line::new(72, "q = a + b - c * d / e & f | g ^ h % i ** j // k")).unwrap(), 
        ];
        
        // Create vector of hashmap results.
        let expected_results: Vec<HashMap<String, Vec<String>>> = vec![
            [("check".to_string(), vec!["x".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string(), "gh".to_string()]), ("new".to_string(), vec!["x".to_string()])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string(), "x".to_string(), "y".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["x".to_string(), "b".to_string(), "w".to_string(), "y".to_string(), "z".to_string(), "q".to_string(), "r".to_string(), "s".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec![]), ("new".to_string(), vec!["c".to_string()])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string(), "e".to_string(), "f".to_string(), "g".to_string(), "h".to_string(), "i".to_string(), "j".to_string(), "k".to_string()]), ("new".to_string(), vec!["q".to_string()])].into_iter().collect(), 
        ];
        
        // Run tests.
        for (assignment, mut expected_result) in std::iter::zip(assignments, expected_results) {
            let mut result: HashMap<String, Vec<String>> = get_variables_from_assignment(assignment.clone());
            
            result.get_mut("new").unwrap().sort();
            result.get_mut("check").unwrap().sort();
            expected_result.get_mut("new").unwrap().sort();
            expected_result.get_mut("check").unwrap().sort();
            
            // Useful for debugging.
            //write_to_writer(&mut writer, format!("Testing `{:?}`\n", assignment.clone()).as_bytes());
            //flush_writer(&mut writer);
            
            assert_eq!(result, expected_result);
        }
    }
    
    #[test]
    fn test_handle_assignment_left_side() {
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let _writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        // Create vector of strings.
        let strings: Vec<String> = vec![
            "x".to_string(), 
            "a[b], c[d]".to_string(), 
            "g[i], b, c, h[j]".to_string(), 
            "a.b.get_z(g, h, i, j).t[k + (l - m * n) / (o & p | (q ^ r) % s) ** (t // u) + b[i * j + 5 * t]]".to_string(), 
            "_42424GRGER_4242gFG".to_string(), 
            "a.get_b(c).d".to_string(), 
            "".to_string(), 
        ];
        
        // Create vector of hashmap results.
        let expected_results: Vec<HashMap<String, Vec<String>>> = vec![
            [("check".to_string(), vec![]), ("new".to_string(), vec!["x".to_string()])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["g".to_string(), "i".to_string(), "h".to_string(), "j".to_string()]), ("new".to_string(), vec!["b".to_string(), "c".to_string()])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "g".to_string(), "h".to_string(), "i".to_string(), "j".to_string(), "k".to_string(), "l".to_string(), "m".to_string(), "n".to_string(), "o".to_string(), "p".to_string(), "q".to_string(), "r".to_string(), "s".to_string(), "t".to_string(), "u".to_string(), "b".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec![]), ("new".to_string(), vec!["_42424GRGER_4242gFG".to_string()])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "c".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec![]), ("new".to_string(), vec![])].into_iter().collect(), 
            
        ];
        
        // Run tests.
        for (string, expected_result) in std::iter::zip(strings, expected_results) {
            let result: HashMap<String, Vec<String>> = handle_assignment_left_side(string);
            assert_eq!(result, expected_result);
        }
    }
    
    #[test]
    fn test_handle_assignment_left_side_single() {
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let _writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        // Create vector of strings.
        let strings: Vec<String> = vec![
            "x".to_string(), 
            "a[b]".to_string(), 
            "g[i]".to_string(), 
            "a.b.get_z(g, h, i, j).t[k + (l - m * n) / (o & p | (q ^ r) % s) ** (t // u) + b[i * j + 5 * t]]".to_string(), 
            "grgeg".to_string(), 
            "_42424GRGER_4242gFG".to_string(), 
            "a.get_b(f(g(h * i) * k(l / m % j)) * p).get_q(r * s(t)).m".to_string(), 
            "f[a].get_b(c * g(d)).k".to_string(), 
            "".to_string(), 
            "a[b] * c[d] + f(g[h] - jk[5])".to_string(), 
            "a.b.c(d.e[f.g], \"h.i, g\", \'Banaan.Fruit\', \"\"\"Multiline string, 5\"\"\", \'\'\'Multiline.Single, 6\'\'\').property".to_string(), 
            "a.get_b(c).d".to_string(), 
            
        ];
        
        // Create vector of hashmap results.
        let expected_results: Vec<HashMap<String, Vec<String>>> = vec![
            [("check".to_string(), vec![]), ("new".to_string(), vec!["x".to_string()])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "b".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["g".to_string(), "i".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "g".to_string(), "h".to_string(), "i".to_string(), "j".to_string(), "k".to_string(), "l".to_string(), "m".to_string(), "n".to_string(), "o".to_string(), "p".to_string(), "q".to_string(), "r".to_string(), "s".to_string(), "t".to_string(), "u".to_string(), "b".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec![]), ("new".to_string(), vec!["grgeg".to_string()])].into_iter().collect(), 
            [("check".to_string(), vec![]), ("new".to_string(), vec!["_42424GRGER_4242gFG".to_string()])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "h".to_string(), "i".to_string(), "l".to_string(), "m".to_string(), "j".to_string(), "p".to_string(), "r".to_string(), "t".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "c".to_string(), "d".to_string(), "f".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec![]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string(), "g".to_string(), "h".to_string(), "jk".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "d".to_string(), "f".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "c".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            
        ];
        
        // Run tests.
        for (string, mut expected_result) in std::iter::zip(strings, expected_results) {
            let mut result: HashMap<String, Vec<String>> = handle_assignment_left_side_single(string.clone());
            
            result.get_mut("new").unwrap().sort();
            expected_result.get_mut("new").unwrap().sort();
            
            let match_count_new: usize   = std::iter::zip(result.get("new").unwrap(), expected_result.get("new").unwrap()).filter(|&(a, b)| a == b).count();
            
            result.get_mut("check").unwrap().sort();
            expected_result.get_mut("check").unwrap().sort();
            
            let match_count_check: usize = std::iter::zip(result.get("check").unwrap(), expected_result.get("check").unwrap()).filter(|&(a, b)| a == b).count();
            
            if !(match_count_new == result.get("new").unwrap().len() && match_count_check == result.get("check").unwrap().len()) {
                println!("Testing `{}`", string.clone());
                println!("handle_assignment_left_single {:?} {:?}", result, expected_result);
            }
            
            assert_eq!(match_count_new,   result.get("new").unwrap().len());
            assert_eq!(match_count_check, result.get("check").unwrap().len());
        }
    }
    
    #[test]
    fn test_handle_assignment_right_side() {
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let _writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        // Create vector of strings.
        let strings: Vec<String> = vec![
            "x".to_string(), 
            "a[b], c[d]".to_string(), 
            "g[i], b, c, h[j]".to_string(), 
            "a.b.get_z(g, h, i, j).t[k + (l - m * n) / (o & p | (q ^ r) % s) ** (t // u) + b[i * j + 5 * t]]".to_string(), 
            "_42424GRGER_4242gFG".to_string(), 
            "\"Some string\"".to_string(), 
            "\"\"\"String triple double quotations\"\"\"".to_string(), 
            "\'Some string\'".to_string(), 
            "\'\'\'String triple double quotations\'\'\'".to_string(), 
            "a[b]".to_string(), 
            "a[b[c[d]]]".to_string(), 
            "f(g(h(a)))".to_string(), 
            "f(j(\"Argument to j\"))".to_string(), 
            "variable".to_string(), 
            "f(a), g(b), a[H], b.g, \"Random string\"".to_string(), 
            "".to_string(), 
        ];
        
        // Create vector of hashmap results.
        let expected_results: Vec<HashMap<String, Vec<String>>> = vec![
            [("check".to_string(), vec!["x".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["g".to_string(), "i".to_string(), "h".to_string(), "j".to_string(), "b".to_string(), "c".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "g".to_string(), "h".to_string(), "i".to_string(), "j".to_string(), "k".to_string(), "l".to_string(), "m".to_string(), "n".to_string(), "o".to_string(), "p".to_string(), "q".to_string(), "r".to_string(), "s".to_string(), "t".to_string(), "u".to_string(), "b".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["_42424GRGER_4242gFG".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec![]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec![]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec![]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec![]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "b".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec![]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["variable".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "b".to_string(), "H".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec![]), ("new".to_string(), vec![])].into_iter().collect(), 
        ];
        
        // Run tests.
        for (string, mut expected_result) in std::iter::zip(strings, expected_results) {
            let mut result: HashMap<String, Vec<String>> = handle_assignment_right_side(string.clone());
            
            result.get_mut("new").unwrap().sort();
            expected_result.get_mut("new").unwrap().sort();
            
            result.get_mut("check").unwrap().sort();
            expected_result.get_mut("check").unwrap().sort();
            
            assert_eq!(result, expected_result);
        }
    }
    
    #[test]
    fn test_handle_assignment_right_side_single() {
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let _writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        // Create vector of strings.
        let strings: Vec<String> = vec![
            "x".to_string(), 
            "a.b.get_z(g, h, i, j).t[k + (l - m * n) / (o & p | (q ^ r) % s) ** (t // u) + b[i * j + 5 * t]]".to_string(), 
            "_42424GRGER_4242gFG".to_string(), 
            "\"Some string\"".to_string(), 
            "\"\"\"String triple double quotations\"\"\"".to_string(), 
            "\'Some string\'".to_string(), 
            "\'\'\'String triple double quotations\'\'\'".to_string(), 
            "a[b]".to_string(), 
            "a[b[c[d]]]".to_string(), 
            "f(g(h(a)))".to_string(), 
            "f(j(\"Argument to j\"))".to_string(), 
            "variable".to_string(), 
            "f(g, a(b), h[k])".to_string(), 
            "".to_string(), 
        ];
        
        // Create vector of hashmap results.
        let expected_results: Vec<HashMap<String, Vec<String>>> = vec![
            [("check".to_string(), vec!["x".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "g".to_string(), "h".to_string(), "i".to_string(), "j".to_string(), "k".to_string(), "l".to_string(), "m".to_string(), "n".to_string(), "o".to_string(), "p".to_string(), "q".to_string(), "r".to_string(), "s".to_string(), "t".to_string(), "u".to_string(), "b".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["_42424GRGER_4242gFG".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec![]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec![]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec![]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec![]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "b".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["a".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec![]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["variable".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec!["g".to_string(), "b".to_string(), "h".to_string(), "k".to_string()]), ("new".to_string(), vec![])].into_iter().collect(), 
            [("check".to_string(), vec![]), ("new".to_string(), vec![])].into_iter().collect(), 
        ];
        
        // Run tests.
        for (string, mut expected_result) in std::iter::zip(strings, expected_results) {
            let mut result: HashMap<String, Vec<String>> = handle_assignment_right_side(string.clone());
            
            result.get_mut("new").unwrap().sort();
            expected_result.get_mut("new").unwrap().sort();
            
            result.get_mut("check").unwrap().sort();
            expected_result.get_mut("check").unwrap().sort();
            
            assert_eq!(result, expected_result);
        }
    }
    
    #[test]
    fn test_handle_assignment_expression() {
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let _writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        // Create vector of strings.
        let strings: Vec<String> = vec![
            "x".to_string(), 
            "a[b]".to_string(), 
            "g[i]".to_string(), 
            "grgeg".to_string(), 
            "_42424GRGER_4242gFG".to_string(), 
            "k + (l - m * n) / (o & p | (q ^ r) % s) ** (t // u) + b[i * j + 5 * t]".to_string(), 
            "".to_string(), 
            "a + a + a + a + a + (b + b + b)".to_string(), 
            "a * 5".to_string(), 
            "a.b.get_z(g, h, i, j).t[k + (l - m * n) / (o & p | (q ^ r) % s) ** (t // u) + b[i * j + 5 * t]]".to_string(), 
            "a.get_b(c).d".to_string(), 
            "a.get_b(c).d + 5 * q".to_string(), 
            "sin(a.b(c) ** 3 + self.base_height * 3 + p)".to_string(), 
            "a == b".to_string(), 
            "a >= b".to_string(), 
            "a <= b".to_string(), 
            "a != b".to_string(), 
            "a > b".to_string(), 
            "a < b".to_string(), 
            "(a == b) and (c == d)".to_string(), 
            "(a == b) or (c == d)".to_string(), 
            "anders and orers".to_string(), 
            "(anders)and(orers)".to_string(), 
            "anders or orers".to_string(), 
            "(anders)or(orers)".to_string(), 
            "a and b or c".to_string(), 
        ];
        
        let expected_results: Vec<Vec<String>> = vec![
            vec!["x".to_string()], 
            vec!["a".to_string(), "b".to_string()], 
            vec!["g".to_string(), "i".to_string()], 
            vec!["grgeg".to_string()], 
            vec!["_42424GRGER_4242gFG".to_string()], 
            vec!["k".to_string(), "l".to_string(), "m".to_string(), "n".to_string(), "o".to_string(), "p".to_string(), "q".to_string(), "r".to_string(), "s".to_string(), "t".to_string(), "u".to_string(), "b".to_string(), "i".to_string(), "j".to_string()], 
            vec![], 
            vec!["a".to_string(), "b".to_string()], 
            vec!["a".to_string()], 
            vec!["a".to_string(), "g".to_string(), "h".to_string(), "i".to_string(), "j".to_string(), "k".to_string(), "l".to_string(), "m".to_string(), "n".to_string(), "o".to_string(), "p".to_string(), "q".to_string(), "r".to_string(), "s".to_string(), "t".to_string(), "u".to_string(), "b".to_string()], 
            vec!["a".to_string(), "c".to_string()], 
            vec!["a".to_string(), "c".to_string(), "q".to_string()], 
            vec!["a".to_string(), "c".to_string(), "self".to_string(), "p".to_string()], 
            vec!["a".to_string(), "b".to_string()], 
            vec!["a".to_string(), "b".to_string()], 
            vec!["a".to_string(), "b".to_string()], 
            vec!["a".to_string(), "b".to_string()], 
            vec!["a".to_string(), "b".to_string()], 
            vec!["a".to_string(), "b".to_string()], 
            vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()], 
            vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()], 
            vec!["anders".to_string(), "orers".to_string()], 
            vec!["anders".to_string(), "orers".to_string()], 
            vec!["anders".to_string(), "orers".to_string()], 
            vec!["anders".to_string(), "orers".to_string()], 
            vec!["a".to_string(), "b".to_string(), "c".to_string()], 
        ];
        
        // Run tests.
        for (string, mut expected_result) in std::iter::zip(strings, expected_results) {
            let mut result: Vec<String> = handle_assignment_expression(string.clone(), true, false);
            
            result.sort();
            expected_result.sort();
            
            assert_eq!(result, expected_result);
        }
    }
    
    #[test]
    fn test_is_enclosed_in_brackets() {
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let _writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        // Create vector of strings.
        let strings: Vec<(bool, String)> = vec![
            (true, "[]".to_string()), 
            (true, "()".to_string()), 
            (true, "[eginegieguier, \"ghrthrt[ hrhrh r hr] ((((eg\"]".to_string()), 
            (true, "(eginegieguier, \"ghrthrt[ hrhrh r hr] ((((eg\")".to_string()), 
            (false, "[".to_string()), 
            (false, "]".to_string()), 
            (false, "(".to_string()), 
            (false, ")".to_string()), 
            (false, "(fbefbwefweytf, \"gegeg)\"".to_string()), 
            (false, "fbefbwefweytf, \"gegeg(\")".to_string()), 
            (false, "[\"]\"".to_string()), 
            (false, "\"[\"]".to_string()), 
            (false, "ggegengiuenge".to_string()), 
            (false, "gegege[gegerg]gegeger".to_string()), 
            (false, "i*j+(k%l)".to_string()), 
            (false, "d[i*j+(k%l)]".to_string()), 
            (false, "[]]".to_string()), 
            (false, "()))".to_string()), 
        ];
        
        // Run tests.
        for (expected_result, string) in strings {
            assert_eq!(is_enclosed_in_brackets(string), expected_result);
        }
    }
    
    #[test]
    fn test_is_string_literal() {
        // Create vector of booleans and strings.
        let strings: Vec<(bool, String)> = vec![
            (true, "\"\"".to_string()), 
            (true, "\"Hi there!!\'\'\' h hrh \' \\\"\"".to_string()), 
            (true, "\'\'".to_string()), 
            (true, "\'\\\"\'".to_string()), 
            (true, "\'\'\'Hiiiii \"\"\" gegeg \" tege\'\'\'".to_string()), 
            (true, "\"\"\"Hkgjegiuehge \' ggeg \'\'\' eg \' eg\"\"\"".to_string()), 
            (false, "(".to_string()), 
            (false, ")".to_string()), 
            (false, "(fbefbwefweytf, \"gegeg)\"".to_string()), 
            (false, "fbefbwefweytf, \"gegeg(\")".to_string()), 
            (false, "[\"]\"".to_string()), 
            (false, "\"[\"]".to_string()), 
            (false, "ggegengiuenge".to_string()), 
            (false, "gegege[gegerg]gegeger".to_string()), 
            (false, "\"\"\"egege\"\"\", \"\"\"egerge\"\"\"".to_string()), 
            (false, "\'\'\'egege\'\'\', \'\'\'egerge\'\'\'".to_string()), 
            (false, "\"egege\", \"egerge\"".to_string()), 
            (false, "\'egege\', \'egerge\'".to_string()), 
            (false, "\"Text\'".to_string()), 
            (false, "\'Text\"".to_string()), 
            (false, "".to_string()), 
        ];
        
        // Run tests.
        for (expected_result, string) in strings {
            assert_eq!(is_string_literal(string), expected_result);
        }
    }
    
    #[test]
    fn test_is_function_call() {
        // Create vector of booleans and strings.
        let strings: Vec<(bool, String)> = vec![
            (false, "\"\"".to_string()), 
            (true, "a(b)".to_string()), 
            (true, "_5353grGRTHrthrth545H_RTH__Hrth_()".to_string()), 
            (true, "_egegerGERG23424(_57834ngjhd[gegerg[trt.hithere].get_property(5, 6, 7)], some_arg(functionarg, bnmaan, \"Banaanhanger\"), \"\"\"Multiline string inside function call.\"\"\", \'\'\'Multiline string inside function call but single quotations instead of double quotations.\'\'\', a[\'Single quotations dict indexing\'].get_some(5, hi, \"Some string\"))".to_string()), 
            (false, "a[b]".to_string()), 
            (false, "a.b(hi, there)".to_string()), 
            (false, "banaan".to_string()), 
            (false, "egege53453DGGer_egEg".to_string()), 
            (false, "Some junk with \"quotations \\\"\" and \'\'\'such\'\'\'. More text [53535[5345345)))){gergerg[\"something\"[}".to_string()), 
            (false, "a = 5".to_string()), 
            (false, "a.b = 5".to_string()), 
            (false, "a, b = 5, 6".to_string()), 
            (false, "def a(b, c):".to_string()), 
            (false, "import math".to_string()), 
            (false, "from math import sin, cos".to_string()), 
            (false, "class Rect:".to_string()), 
            (false, "class Rect(Shape):".to_string()), 
            (false, "a(b) * c(d)".to_string()), 
            (true, "a(a(b))".to_string()), 
            (true, "a(b(c) * d(e))".to_string()), 
            (true, "f(\"\\\")\")".to_string()), 
            (true, "f(\'\\\')\')".to_string()), 
            (false, "f(\"\\\")\"".to_string()), 
            (false, "f\'\\\')\')".to_string()), 
            (true, "f(\"\\\')\")".to_string()), 
            (true, "f(\'\\\")\')".to_string()), 
            (false, "f(a) + 5".to_string()), 
            (false, "f(a".to_string()), 
        ];
        
        // Run tests.
        for (expected_result, string) in strings {
            assert_eq!(is_function_call(string), expected_result);
        }
    }
    
    #[test]
    fn test_is_array_access() {
        // Create vector of booleans and strings.
        let strings: Vec<(bool, String)> = vec![
            (true, "a[b]".to_string()), 
            (false, "a.b[c]".to_string()), 
            (true, "a[5 * f(g) + b[t.get_property(a[b] + c[d] - g(h * 5)) + 6]]".to_string()), 
            (false, "f(a)".to_string()), 
            (false, "f(a]".to_string()), 
            (false, "[a + b - c]".to_string()), 
            (false, "class Rect:".to_string()), 
            (false, "class Rect(Shape):".to_string()), 
            (false, "def func(a, b):".to_string()), 
            (false, "import math".to_string()), 
            (false, "from math import sin, cos, sqrt".to_string()), 
            (false, "a(b(c) * d(e))".to_string()), 
            (true, "a[a[a(b)]]".to_string()), 
            (false, "f\'\\\')\')".to_string()), 
            (false, "a(b) * c(d)".to_string()), 
            (false, "a[b] * c[d] - 5".to_string()), 
            (false, "f[a] + 5".to_string()), 
            (false, "f[a".to_string()), 
        ];
        
        // Run tests.
        for (expected_result, string) in strings {
            assert_eq!(is_array_access(string), expected_result);
        }
    }
    
    #[test]
    fn test_contains_arithmetic_symbols_not_enclosed() {
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let _writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        // Create vector of booleans and strings.
        let strings: Vec<(bool, String)> = vec![
            (true, "a + b".to_string()), 
            (true, "a - b".to_string()), 
            (true, "a % b".to_string()), 
            (true, "a ^ b".to_string()), 
            (true, "a & b".to_string()), 
            (true, "a | b".to_string()), 
            (true, "a < b".to_string()), 
            (true, "a > b".to_string()), 
            (true, "a ! b".to_string()), 
            (true, "a * b".to_string()), 
            (true, "a / b".to_string()), 
            (true, "a = b".to_string()), 
            
            (false, "f(a + b)".to_string()), 
            (false, "f(a - b)".to_string()), 
            (false, "f(a % b)".to_string()), 
            (false, "f(a ^ b)".to_string()), 
            (false, "f(a & b)".to_string()), 
            (false, "f(a | b)".to_string()), 
            (false, "f(a < b)".to_string()), 
            (false, "f(a > b)".to_string()), 
            (false, "f(a ! b)".to_string()), 
            (false, "f(a * b)".to_string()), 
            (false, "f(a / b)".to_string()), 
            (false, "f(a = b)".to_string()), 
            
            (false, "f[a + b]".to_string()), 
            (false, "f[a - b]".to_string()), 
            (false, "f[a % b]".to_string()), 
            (false, "f[a ^ b]".to_string()), 
            (false, "f[a & b]".to_string()), 
            (false, "f[a | b]".to_string()), 
            (false, "f[a < b]".to_string()), 
            (false, "f[a > b]".to_string()), 
            (false, "f[a ! b]".to_string()), 
            (false, "f[a * b]".to_string()), 
            (false, "f[a / b]".to_string()), 
            (false, "f[a = b]".to_string()), 
            
            (false, "f{a + b}".to_string()), 
            (false, "f{a - b}".to_string()), 
            (false, "f{a % b}".to_string()), 
            (false, "f{a ^ b}".to_string()), 
            (false, "f{a & b}".to_string()), 
            (false, "f{a | b}".to_string()), 
            (false, "f{a < b}".to_string()), 
            (false, "f{a > b}".to_string()), 
            (false, "f{a ! b}".to_string()), 
            (false, "f{a * b}".to_string()), 
            (false, "f{a / b}".to_string()), 
            (false, "f{a = b}".to_string()), 
            
            (false, "\'a + b\'".to_string()), 
            (false, "\'a - b\'".to_string()), 
            (false, "\'a % b\'".to_string()), 
            (false, "\'a ^ b\'".to_string()), 
            (false, "\'a & b\'".to_string()), 
            (false, "\'a | b\'".to_string()), 
            (false, "\'a < b\'".to_string()), 
            (false, "\'a > b\'".to_string()), 
            (false, "\'a ! b\'".to_string()), 
            (false, "\'a * b\'".to_string()), 
            (false, "\'a / b\'".to_string()), 
            (false, "\'a = b\'".to_string()), 
            
            (false, "\"a + b\"".to_string()), 
            (false, "\"a - b\"".to_string()), 
            (false, "\"a % b\"".to_string()), 
            (false, "\"a ^ b\"".to_string()), 
            (false, "\"a & b\"".to_string()), 
            (false, "\"a | b\"".to_string()), 
            (false, "\"a < b\"".to_string()), 
            (false, "\"a > b\"".to_string()), 
            (false, "\"a ! b\"".to_string()), 
            (false, "\"a * b\"".to_string()), 
            (false, "\"a / b\"".to_string()), 
            (false, "\"a = b\"".to_string()), 
            
            (false, "a.get(b + c + d)".to_string()), 
        ];
        
        // Run tests.
        for (expected_result, string) in strings {
            assert_eq!(contains_arithmetic_symbols_not_enclosed(string), expected_result);
        }
    }
    
    #[test]
    fn test_split_by_char() {
        // Initialize writer.
        let stdout_handle = std::io::stdout();
        let _writer: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(stdout_handle));
        
        // Create vector of strings.
        let strings: Vec<(String, char)> = vec![
            ("a,b,c".to_string(), ','), 
            ("a, b, c".to_string(), ','), 
            ("ggue, \"a, b, c\", hiii, \"hthtrh, rhrthrt;; [hrhr]rh\"".to_string(), ','), 
            ("ggerngeug55345gGGERERGer".to_string(), ','), 
            ("\"gege, gergerg,egerge,ge[gegeg]]]]egegerg(ggeg{}{}{}}}}))))egege\"".to_string(), ','), 
            ("f(a, b, c)".to_string(), ','), 
            ("g(a, b), h(c, d), [a, b, c]".to_string(), ','), 
            ("g(a, b, \"Banaan)\"), ghghg".to_string(), ','), 
            ("), \"rgerger, ())[[[}}}\"".to_string(), ','), 
            ("a-b+c".to_string(), ','), 
            ("\"\\\", hi there\"".to_string(), ','), 
            ("\'\\\', hi there\'".to_string(), ','), 
            ("\\\"".to_string(), ','), 
            ("\\\'".to_string(), ','), 
            
            ("a.b.c".to_string(), '.'), 
            ("wfwfwfw.thtrh.grthrt.hrhrth".to_string(), '.'), 
            ("a(b.b), a.b.c, p.5".to_string(), '.'), 
            ("_5353GREGRGtgrtg.rhrh6535_gGGre3t3g".to_string(), '.'), 
            ("a[5 * t - d].func(a+b, c+d, a * (g % 6 // 8) + sum([1, 2, 3])).property".to_string(), '.'), 
            ("a.b(c.d)".to_string(), '.'), 
            ("a.b(c.d[e.f], \"g.h = 5\")".to_string(), '.'), 
        ];
        
        // Create vector results.
        let expected_results: Vec<Vec<String>> = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()], 
            vec!["a".to_string(), "b".to_string(), "c".to_string()], 
            vec!["ggue".to_string(), "\"a, b, c\"".to_string(), "hiii".to_string(), "\"hthtrh, rhrthrt;; [hrhr]rh\"".to_string()], 
            vec!["ggerngeug55345gGGERERGer".to_string()], 
            vec!["\"gege, gergerg,egerge,ge[gegeg]]]]egegerg(ggeg{}{}{}}}}))))egege\"".to_string()], 
            vec!["f(a, b, c)".to_string()], 
            vec!["g(a, b)".to_string(), "h(c, d)".to_string(), "[a, b, c]".to_string()], 
            vec!["g(a, b, \"Banaan)\")".to_string(), "ghghg".to_string()], 
            vec![")".to_string(), "\"rgerger, ())[[[}}}\"".to_string()], 
            vec!["a-b+c".to_string()], 
            vec!["\"\\\", hi there\"".to_string()], 
            vec!["\'\\\', hi there\'".to_string()], 
            vec!["\\\"".to_string()], 
            vec!["\\\'".to_string()], 
            
            vec!["a".to_string(), "b".to_string(), "c".to_string()], 
            vec!["wfwfwfw".to_string(), "thtrh".to_string(), "grthrt".to_string(), "hrhrth".to_string()], 
            vec!["a(b.b), a".to_string(), "b".to_string(), "c, p".to_string(), "5".to_string()], 
            vec!["_5353GREGRGtgrtg".to_string(), "rhrh6535_gGGre3t3g".to_string()], 
            vec!["a[5 * t - d]".to_string(), "func(a+b, c+d, a * (g % 6 // 8) + sum([1, 2, 3]))".to_string(), "property".to_string()], 
            vec!["a".to_string(), "b(c.d)".to_string()], 
            vec!["a".to_string(), "b(c.d[e.f], \"g.h = 5\")".to_string()], 
        ];
        
        // Run tests.
        for ((string, delimiter), expected_result) in std::iter::zip(strings, expected_results) {
            assert_eq!(split_by_char(string, delimiter), expected_result);
        }
    }
    
}
