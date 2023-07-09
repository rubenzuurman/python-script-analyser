use regex::Regex;

#[derive(Debug, PartialEq)]
pub struct Function {
    name: String, 
    parameters: Vec<String>, 
    source: Vec<String>, 
}

impl Function {
    
    pub fn create(source_lines: Vec<String>) -> Self {
        // Regex for getting function name index of first line.
        let def_line: &str = match source_lines.get(0) {
            None => panic!("Unable to get first line from source."), 
            Some(v) => v
        };
        let re_name = Regex::new(r"def (?<name>\w+)").unwrap();
        let Some(captures) = re_name.captures(&def_line) else {
            panic!("No function name found in string '{}'.", def_line);
        };
        println!("Name: {}", &captures["name"]);
        
        return Function {name: String::from("temp_name"), parameters: vec![String::from("param1"), String::from("param2")], source: vec![String::from("Line 1"), String::from("Line 2")]};
    }
    
}

struct Class {
    name: String, 
    parent_class: String, 
    variables: Vec<String>, 
    methods: Vec<ClassMethod>, // Methods in the class.
}

struct ClassMethod {
    name: String, 
    parameters: Vec<String>, 
    source: Vec<String>, // Lines in the method.
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[test]
    fn test_create() {
        let func: Function = Function::create(vec![String::from("def func_name(param1, param2):"), String::from("Appel")]);
        assert_eq!(func, Function::create(vec![String::from("appel")]));
    }
    
}
