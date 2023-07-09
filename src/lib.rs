use regex::Regex;

#[derive(Debug, PartialEq)]
pub struct Function {
    name: String, 
    parameters: Vec<String>, 
    source: Vec<String>, 
}

impl Function {
    
    pub fn create(source_lines: Vec<String>) -> Self {
        // Get first line of the source.
        let def_line: &str = match source_lines.get(0) {
            Some(v) => v, 
            None => panic!("Unable to get first line from source."), 
        };
        
        // Initialize regex for getting the function name and the parameters from the function definition.
        let re_name_params = Regex::new(r"^def (?<name>\w+)\((?<params>[\w ,]*)\):$").unwrap();
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
                    parameters.push(String::from(re_replace_space.replace_all(param, " ")));
                }
            }, 
            None => {
                eprintln!("Invalid function definition on the first line of the source '{}'.", def_line);
            }
        }
        
        // Return function object.
        return Function {name: name, parameters: parameters, source: source_lines};
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
