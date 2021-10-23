use crate::evaluator::EvalResult;
use crate::object::Object;
use std::collections::BTreeMap;

pub fn new() -> BTreeMap<String, Object> {
    let mut buildins = BTreeMap::new();

    buildins.insert("len".to_string(), Object::Buildin { function: len });
    buildins.insert("first".to_string(), Object::Buildin { function: first });
    buildins.insert("last".to_string(), Object::Buildin { function: last });
    buildins.insert("rest".to_string(), Object::Buildin { function: rest });
    buildins.insert("push".to_string(), Object::Buildin { function: push });

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

fn first(arguments: Vec<Object>) -> EvalResult {
    if arguments.len() != 1 {
        let message = format!("wrong number of arguments. got={}, want=1", arguments.len());
        return Err(message);
    }

    let result = match &arguments[0] {
        Object::Array(elements) => elements.first().unwrap_or(&Object::Null).clone(),
        _ => {
            let message = format!(
                "argument to `first` must be Array, got {}",
                arguments[0].get_type()
            );
            return Err(message);
        }
    };

    Ok(result)
}

fn last(arguments: Vec<Object>) -> EvalResult {
    if arguments.len() != 1 {
        let message = format!("wrong number of arguments. got={}, want=1", arguments.len());
        return Err(message);
    }

    let result = match &arguments[0] {
        Object::Array(elements) => elements.last().unwrap_or(&Object::Null).clone(),
        _ => {
            let message = format!(
                "argument to `last` must be Array, got {}",
                arguments[0].get_type()
            );
            return Err(message);
        }
    };

    Ok(result)
}

fn rest(arguments: Vec<Object>) -> EvalResult {
    if arguments.len() != 1 {
        let message = format!("wrong number of arguments. got={}, want=1", arguments.len());
        return Err(message);
    }

    let result = match &arguments[0] {
        Object::Array(elements) => match elements.split_first() {
            Some((_, tail)) => Object::Array(tail.to_vec()),
            _ => Object::Null,
        },
        _ => {
            let message = format!(
                "argument to `rest` must be Array, got {}",
                arguments[0].get_type()
            );
            return Err(message);
        }
    };

    Ok(result)
}

fn push(arguments: Vec<Object>) -> EvalResult {
    if arguments.len() != 2 {
        let message = format!("wrong number of arguments. got={}, want=2", arguments.len());
        return Err(message);
    }

    let result = match (&arguments[0], &arguments[1]) {
        (Object::Array(elements), object) => {
            let mut elements = elements.clone();
            let object = object.clone();
            elements.push(object);
            Object::Array(elements)
        }
        _ => {
            let message = format!(
                "argument to `push` must be Array, got {}",
                arguments[0].get_type()
            );
            return Err(message);
        }
    };

    Ok(result)
}
