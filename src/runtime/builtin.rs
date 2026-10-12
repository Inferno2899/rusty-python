use crate::runtime::{Object, ObjectRef, Runtime, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BuiltinFunction {
    Print,
}

impl BuiltinFunction {
    pub(crate) fn call(
        &self,
        runtime: &mut Runtime,
        arguments: Vec<Value>,
    ) -> Result<Value, String> {
        match self {
            BuiltinFunction::Print => {
                for argument in arguments {
                    println!("{}", runtime.debug_display(&argument));
                }

                Ok(Value::None)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BuiltinMethod {
    ListAppend,
}

impl BuiltinMethod {
    pub(crate) fn call(
        &mut self,
        runtime: &mut Runtime,
        object_ref: ObjectRef,
        arguments: Vec<Value>,
    ) -> Result<Value, String> {
        match self {
            BuiltinMethod::ListAppend => {
                if arguments.len() != 1 {
                    return Err(format!(
                        "append expected 1 argument, got {}",
                        arguments.len()
                    ));
                }

                let value = arguments.into_iter().next().unwrap();

                let object = runtime.get_mut(object_ref);

                match object {
                    Object::List(list) => {
                        list.push(value);
                        Ok(Value::None)
                    }

                    _ => Err("append requires a list".to_string()),
                }
            }
        }
    }
}
