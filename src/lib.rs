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
    
    pub fn create(source_lines: &Vec<Line>) -> Self {
        // Get first line of the source.
        let def_line: &str = match source_lines.get(0) {
            Some(v) => v.get_text(), 
            None => panic!("Unable to get first line from source."), 
        };
        
        // Initialize regex for getting the function name and the parameters from the function definition.
        let re_name_params = Regex::new(r"^def (?<name>\w+)\((?<params>[\w ,=\*]*)\):$").unwrap();
        let re_replace_space = Regex::new(" +").unwrap();
        
        // Match regex and initialize function properties.
        let name_params_captures = re_name_params.captures(def_line);
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
                eprintln!("Invalid function definition on the first line of the source '{}'.", def_line);
            }
        }
        
        // Return function object.
        return Function {name: name, parameters: parameters, source: source_lines.to_vec()};
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

pub struct Class {
    name: String, 
    parent_class: String, 
    variables: Vec<String>, 
    methods: Vec<ClassMethod>, // Methods in the class.
    source: Vec<Line>, 
}

impl Class {
    
    /*pub fn create(source: Vec<Line>) -> Self {
        return Class {};
    }*/
    
}

struct ClassMethod {
    name: String, 
    parameters: Vec<String>, 
    source: Vec<Line>, // Lines in the method.
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[test]
    fn test_create_function() {
        let lines_str: Vec<String> = vec![String::from("def func_name(param1, param2, param3=5, *args, **kwargs):"), String::from("Appel")];
        let mut lines: Vec<Line> = Vec::new();
        for (index, s) in lines_str.iter().enumerate() {
            lines.push(Line::create(index, s))
        }
        let func: Function = Function::create(&lines);
        
        let func_name_want: String = String::from("func_name");
        let func_params_want: Vec<String> = vec!["param1".to_string(), "param2".to_string(), "param3=5".to_string(), "*args".to_string(), "**kwargs".to_string()];
        let func_want: Function = Function {name: func_name_want, parameters: func_params_want, source: lines};
        assert_eq!(func, func_want);
    }
    
    #[test]
    fn test_create_line() {
        let test_cases: Vec<(usize, &str)> = vec![(25, "Hi there"), (100, "This is some string with w31rd characters !_(*)`~|\\[]{};:'\",.<>/?!@#$%^&*()_+-=說文閩音通한국어 키보드乇乂ㄒ尺卂　ㄒ卄丨匚匚Моя семья"), (1000000000, "Big line number")];
        
        for (line_number, text) in test_cases {
            let line = Line::create(line_number, text);
            let line_want = Line {number: line_number, text: text.to_string()};
            assert_eq!(line, line_want);
        }
    }
    
}
