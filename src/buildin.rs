use crate::evaluator::EvalResult;
use crate::object::Object;
use std::collections::HashMap;

pub fn new() -> HashMap<String, Object> {
    let mut buildins = HashMap::new();

    buildins.insert("len".to_string(), Object::Buildin { function: len });

    buildins
}

fn len(arguments: Vec<Object>) -> EvalResult {
    if arguments.len() != 1 {
        let message = format!("wrong number of arguments. got={}, want=1", arguments.len());
        return Err(message);
    }

    let result = match &arguments[0] {
        Object::Strings(value) => Object::Integer(value.len() as isize),
        _ => {
            let message = format!(
                "argument to `len` not supported, got {}",
                arguments[0].get_type()
            );
            return Err(message);
        }
    };

    Ok(result)
}
