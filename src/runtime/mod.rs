mod builtin;
mod heap;
mod object;
mod value;

pub(crate) use builtin::{BuiltinFunction, BuiltinMethod};
pub(crate) use object::{BoundMethod, Class, Instance, Object, VmFunction};
pub(crate) use value::{ObjectRef, Value};

use heap::Heap;

pub(crate) struct Runtime {
    heap: Heap,
    allocations_since_gc: usize,
    gc_threshold: usize,
}

impl Runtime {
    pub(crate) fn new() -> Self {
        Self {
            heap: Heap::new(),
            allocations_since_gc: 0,
            gc_threshold: 1,
        }
    }
}

impl Runtime {
    pub(crate) fn get(&self, reference: ObjectRef) -> &Object {
        &self.heap.get(reference).object
    }

    pub(crate) fn get_mut(&mut self, reference: ObjectRef) -> &mut Object {
        &mut self.heap.get_mut(reference).object
    }
}

// Attribute manipulation
impl Runtime {
    pub(crate) fn get_attribute(
        &mut self,
        object: Value,
        attribute_name: &str,
    ) -> Result<Value, String> {
        match object {
            Value::Object(reference) => match (self.get(reference), attribute_name) {
                (Object::List(_), "append") => Ok(Value::BuiltinMethod {
                    object_ref: reference,
                    method: BuiltinMethod::ListAppend,
                }),
                (Object::Class(class), attribute_name) => {
                    if let Some(attribute) = class.attributes.get(attribute_name) {
                        Ok(*attribute)
                    } else {
                        Err(format!(
                            "class {} has no attribute '{attribute_name}'",
                            class.name
                        ))
                    }
                }
                (Object::Instance(instance), attribute_name) => {
                    if let Some(attribute) = instance.attributes.get(attribute_name) {
                        return Ok(*attribute);
                    }

                    let class_ref = instance.class;

                    let Object::Class(class) = self.get(class_ref) else {
                        unreachable!("instance must reference a class");
                    };

                    let attribute = *class.attributes.get(attribute_name).ok_or_else(|| {
                        format!("{} object has no attribute '{attribute_name}'", class.name)
                    })?;

                    if let Value::Object(function_reference) = attribute
                        && matches!(self.get(function_reference), Object::Function(_))
                    {
                        let object_ref = self.allocate(Object::BoundMethod(BoundMethod {
                            receiver: reference,
                            function: function_reference,
                        }));
                        Ok(Value::Object(object_ref))
                    } else {
                        Ok(attribute)
                    }
                }
                (object, attribute_name) => Err(format!(
                    "object {object:?} has no attribute '{attribute_name}'"
                )),
            },
            _ => Err("only objects have attributes".to_string()),
        }
    }

    pub(crate) fn set_attribute(
        &mut self,
        object: Value,
        attribute_name: &String,
        value: Value,
    ) -> Result<(), String> {
        match object {
            Value::Object(reference) => match (self.get_mut(reference), attribute_name) {
                (Object::Class(class), attribute_name) => {
                    class.attributes.insert(attribute_name.clone(), value);
                    Ok(())
                }
                (Object::Instance(instance), attribute_name) => {
                    instance.attributes.insert(attribute_name.clone(), value);
                    Ok(())
                }
                (object, attribute_name) => Err(format!(
                    "setting attribute '{attribute_name}' to object {object:?} is not supported"
                )),
            },
            _ => Err("only objects have attributes".to_string()),
        }
    }
}

// Object indexing
impl Runtime {
    pub(crate) fn get_item(&mut self, object: Value, index: Value) -> Result<Value, String> {
        if let Value::Integer(index) = index {
            match object {
                Value::Object(reference) => match self.get(reference) {
                    Object::List(values) => Ok(values[Self::normalize_index(index, values.len())?]),
                    Object::Str(s) => {
                        let c = s
                            .chars()
                            .nth(Self::normalize_index(index, s.len())?)
                            .ok_or_else(|| "string index out of range".to_string())?;
                        let value = self.allocate(Object::Str(c.to_string()));
                        Ok(Value::Object(value))
                    }
                    _ => Err("type cannot be indexed".to_string()),
                },
                _ => Err("type cannot be indexed".to_string()),
            }
        } else {
            Err(format!("{index:?} cannot be an index"))
        }
    }

    pub(crate) fn set_item(
        &mut self,
        object: Value,
        index: Value,
        value: Value,
    ) -> Result<(), String> {
        if let Value::Integer(index) = index {
            match object {
                Value::Object(reference) => match self.get_mut(reference) {
                    Object::List(values) => {
                        let length = values.len();
                        values[Self::normalize_index(index, length)?] = value;
                        Ok(())
                    }
                    Object::Str(_) => Err("cannot assign to string".to_string()),
                    _ => Err("type cannot be indexed".to_string()),
                },
                _ => Err("type cannot be indexed".to_string()),
            }
        } else {
            Err(format!("{index:?} cannot be an index"))
        }
    }

    fn normalize_index(index: i64, length: usize) -> Result<usize, String> {
        let length = i64::try_from(length).map_err(|_| "sequence too large".to_string())?;

        let normalized = if index < 0 {
            index.checked_add(length).ok_or("index out of range")?
        } else {
            index
        };

        if normalized < 0 || normalized >= length {
            return Err("index out of range".into());
        }

        Ok(normalized as usize)
    }
}

// Memory management
impl Runtime {
    pub(crate) fn mark(&mut self, reference: ObjectRef) {
        self.heap.mark(reference);
    }

    pub(crate) fn sweep(&mut self) {
        self.heap.sweep();
        self.allocations_since_gc = 0;
    }

    pub(crate) fn should_collect(&self) -> bool {
        self.allocations_since_gc >= self.gc_threshold
    }

    pub(crate) fn allocate(&mut self, object: Object) -> ObjectRef {
        let reference = self.heap.allocate(object);
        self.allocations_since_gc += 1;
        reference
    }
}

// Debug display
impl Runtime {
    pub(crate) fn debug_display(&self, value: &Value) -> String {
        match value {
            Value::Integer(value) => value.to_string(),
            Value::Double(value) => value.to_string(),

            Value::Boolean(true) => "True".to_string(),
            Value::Boolean(false) => "False".to_string(),

            Value::None => "None".to_string(),

            Value::Object(object_ref) => self.debug_display_object(*object_ref),

            Value::BuiltinFunction(_) => "<built-in function>".to_string(),

            Value::BuiltinMethod { .. } => "<built-in method>".to_string(),
        }
    }

    fn debug_display_object(&self, object_ref: ObjectRef) -> String {
        match &self.heap.get(object_ref).object {
            Object::Str(value) => value.clone(),
            Object::List(list) => {
                let values = list
                    .iter()
                    .map(|value| self.debug_display(value))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{values}]")
            }
            Object::Class(class) => format!("<class {}>", class.name),
            Object::Instance(instance) => {
                if let Object::Class(class) = &self.heap.get(instance.class).object {
                    format!("<{} object>", class.name)
                } else {
                    unreachable!("instance must have a pointer to a class")
                }
            }
            Object::Function(function) => {
                format!("<function {}>", function.function_index)
            }
            Object::BoundMethod(bound_method) => {
                let object = self.get(bound_method.receiver);
                let function = self.get(bound_method.function);
                match (object, function) {
                    (
                        Object::Instance(Instance { class, .. }),
                        Object::Function(VmFunction { .. }),
                    ) => {
                        format!("<bound method {:?}>", class)
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}

// Arithmetic operations
impl Runtime {
    pub(crate) fn add(&mut self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),

            (left, right) => Err(format!("cannot add {left:?} and {right:?}")),
        }
    }

    pub(crate) fn subtract(&mut self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),

            (left, right) => Err(format!("cannot subtract {left:?} and {right:?}")),
        }
    }

    pub(crate) fn multiply(&mut self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),

            (left, right) => Err(format!("cannot multiply {left:?} and {right:?}")),
        }
    }

    pub(crate) fn divide(&mut self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (_, Value::Integer(0)) => Err("zero division".to_string()),

            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Double((a / b) as f64)),

            (left, right) => Err(format!("cannot divide {left:?} and {right:?}")),
        }
    }
}

// Compare operations
impl Runtime {
    pub(crate) fn greater(&mut self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a > b)),

            (left, right) => Err(format!("cannot compare {left:?} and {right:?}")),
        }
    }

    pub(crate) fn less(&mut self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a < b)),

            (left, right) => Err(format!("cannot compare {left:?} and {right:?}")),
        }
    }

    pub(crate) fn equal(&mut self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a == b)),

            (left, right) => Err(format!("cannot compare {left:?} and {right:?}")),
        }
    }
}
