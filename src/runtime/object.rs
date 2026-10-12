use std::collections::HashMap;

use crate::runtime::{ObjectRef, Value};

#[derive(Debug)]
pub(crate) enum Object {
    Str(String),
    List(Vec<Value>),
    Class(Class),
    Instance(Instance),
    Function(VmFunction),
    BoundMethod(BoundMethod),
}

impl Object {
    pub(crate) fn references(&self) -> Vec<ObjectRef> {
        let mut refs = Vec::new();

        match self {
            Object::Str(_) => {}
            Object::List(values) => {
                for value in values {
                    if let Value::Object(reference) = value {
                        refs.push(*reference);
                    }
                }
            }
            Object::Class(class) => {
                for value in class.attributes.values() {
                    if let Value::Object(reference) = value {
                        refs.push(*reference);
                    }
                }
            }
            Object::Instance(instance) => {
                refs.push(instance.class);

                for value in instance.attributes.values() {
                    if let Value::Object(reference) = value {
                        refs.push(*reference);
                    }
                }
            }
            Object::Function(_) => {}
            Object::BoundMethod(method) => {
                refs.push(method.receiver);
                refs.push(method.function);
            }
        }
        refs
    }
}

#[derive(Debug)]
pub(crate) struct Class {
    pub(crate) name: String,
    pub(crate) attributes: HashMap<String, Value>,
}

#[derive(Debug)]
pub(crate) struct Instance {
    pub(crate) class: ObjectRef,
    pub(crate) attributes: HashMap<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VmFunction {
    pub(crate) function_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoundMethod {
    pub(crate) receiver: ObjectRef,
    pub(crate) function: ObjectRef,
}
