use std::collections::HashMap;

use crate::runtime::{
    BoundMethod, BuiltinFunction, Class, Instance, Object, ObjectRef, Runtime, Value, VmFunction,
};
use crate::vm::{CompiledProgram, Instruction};

pub struct Machine {
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    frames: Vec<Frame>,
    runtime: Runtime,
}

impl Machine {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        globals.insert(
            "print".to_string(),
            Value::BuiltinFunction(BuiltinFunction::Print),
        );

        let frames = vec![Frame::Module { ip: 0 }];

        Self {
            stack: Vec::new(),
            globals,
            frames,
            runtime: Runtime::new(),
        }
    }

    pub fn execute(&mut self, program: CompiledProgram) -> Result<(), String> {
        loop {
            let instruction = match self.current_instruction(&program) {
                Some(instruction) => instruction,
                None => match self.frames.last().unwrap() {
                    Frame::Module { .. } => break,
                    Frame::Function { .. } => {
                        self.finish_function(Value::None)?;
                        continue;
                    }
                    Frame::Class { .. } => {
                        let frame = self.frames.pop().unwrap();

                        let Frame::Class {
                            class_index,
                            namespace,
                            ..
                        } = frame
                        else {
                            unreachable!()
                        };

                        let name = program.classes[class_index].name.clone();
                        let object = Object::Class(Class {
                            name,
                            attributes: namespace,
                        });
                        let reference = self.runtime.allocate(object);
                        self.stack.push(Value::Object(reference));

                        continue;
                    }
                },
            };
            match instruction {
                Instruction::LoadNone => {
                    self.stack.push(Value::None);
                    *self.current_frame_mut().ip_mut() += 1;
                }
                Instruction::LoadClass(index) => {
                    *self.current_frame_mut().ip_mut() += 1;
                    let frame = Frame::Class {
                        class_index: *index,
                        ip: 0,
                        namespace: HashMap::new(),
                    };
                    self.frames.push(frame);
                }
                Instruction::LoadInteger(x) => {
                    self.stack.push(Value::Integer(*x));
                    *self.current_frame_mut().ip_mut() += 1;
                }
                Instruction::LoadString(value) => {
                    let reference = self.runtime.allocate(Object::Str(value.clone()));
                    self.stack.push(Value::Object(reference));
                    *self.current_frame_mut().ip_mut() += 1;
                }
                Instruction::LoadAttribute(attribute_name) => {
                    let object = self.stack.pop().unwrap();
                    let attribute = self.runtime.get_attribute(object, attribute_name)?;
                    self.stack.push(attribute);
                    *self.current_frame_mut().ip_mut() += 1;
                }
                Instruction::StoreAttribute(attribute_name) => {
                    let object = self.stack.pop().unwrap();
                    let value = self.stack.pop().unwrap();
                    self.runtime.set_attribute(object, attribute_name, value)?;
                    *self.current_frame_mut().ip_mut() += 1;
                }
                Instruction::GetItem => {
                    let index = self.stack.pop().unwrap();
                    let object = self.stack.pop().unwrap();
                    let value = self.runtime.get_item(object, index)?;
                    self.stack.push(value);
                    *self.current_frame_mut().ip_mut() += 1;
                }
                Instruction::SetItem => {
                    let index = self.stack.pop().unwrap();
                    let object = self.stack.pop().unwrap();
                    let value = self.stack.pop().unwrap();
                    self.runtime.set_item(object, index, value)?;
                    *self.current_frame_mut().ip_mut() += 1;
                }
                Instruction::BuildList(size) => {
                    let split_index = self.stack.len() - size;
                    let values = self.stack.split_off(split_index);

                    let reference = self.runtime.allocate(Object::List(values));
                    self.stack.push(Value::Object(reference));
                    *self.current_frame_mut().ip_mut() += 1;
                }
                Instruction::StoreName(name) => {
                    let value = self.stack.pop().unwrap();
                    self.store_name(name, value);
                    *self.frames.last_mut().unwrap().ip_mut() += 1;
                }
                Instruction::LoadName(name) => {
                    self.load_name(name);
                    *self.frames.last_mut().unwrap().ip_mut() += 1;
                }
                Instruction::Call(arguments_size) => {
                    *self.current_frame_mut().ip_mut() += 1;

                    let mut arguments = Vec::with_capacity(*arguments_size);
                    for _ in 0usize..*arguments_size {
                        arguments.push(self.stack.pop().unwrap());
                    }
                    let callable = self.stack.pop().unwrap();

                    self.call_value(&program, callable, arguments)?
                }
                Instruction::LoadFunction(index) => {
                    let function = self.runtime.allocate(Object::Function(VmFunction {
                        function_index: *index,
                    }));
                    self.stack.push(Value::Object(function));
                    *self.current_frame_mut().ip_mut() += 1;
                }
                Instruction::Return => {
                    let value = self.stack.pop().unwrap();
                    self.finish_function(value)?;
                }
                Instruction::Add => {
                    self.binary_op(Runtime::add)?;
                }
                Instruction::Subtract => {
                    self.binary_op(Runtime::subtract)?;
                }
                Instruction::Multiply => {
                    self.binary_op(Runtime::multiply)?;
                }
                Instruction::Divide => {
                    self.binary_op(Runtime::divide)?;
                }
                Instruction::Greater => {
                    self.binary_op(Runtime::greater)?;
                }
                Instruction::Less => {
                    self.binary_op(Runtime::less)?;
                }
                Instruction::Equal => {
                    self.binary_op(Runtime::equal)?;
                }
                Instruction::Jump(index) => {
                    let frame = self.frames.last_mut().unwrap();
                    *frame.ip_mut() = *index;
                }
                Instruction::JumpIfFalse(index) => {
                    let frame = self.frames.last_mut().unwrap();
                    let value = self.stack.pop();
                    match (frame, value) {
                        (Frame::Module { ip }, Some(Value::Boolean(condition))) => {
                            if !condition {
                                *ip = *index
                            } else {
                                *ip += 1;
                            }
                        }
                        (Frame::Function { ip, .. }, Some(Value::Boolean(condition))) => {
                            if !condition {
                                *ip = *index
                            } else {
                                *ip += 1;
                            }
                        }
                        _ => unreachable!("JumpIfFalse must have boolean on the stack before it"),
                    }
                }
                Instruction::Pop => {
                    self.stack.pop();
                    *self.current_frame_mut().ip_mut() += 1;
                }
            }
            self.collect_garbage();
        }
        println!("{:?}", self.stack);
        println!("{:?}", self.globals);
        Ok(())
    }
}

// Name storing and loading
impl Machine {
    fn store_name(&mut self, name: &str, value: Value) {
        let frame = self.frames.last_mut().unwrap();
        match frame {
            Frame::Module { .. } => {
                self.globals.insert(name.to_owned(), value);
            }
            Frame::Function { locals, .. } => {
                locals.insert(name.to_owned(), value);
            }
            Frame::Class { namespace, .. } => {
                namespace.insert(name.to_owned(), value);
            }
        }
    }

    fn load_name(&mut self, name: &str) {
        let frame = self.frames.last_mut().unwrap();
        match frame {
            Frame::Module { .. } => {
                self.stack.push(*self.globals.get(name).unwrap());
            }
            Frame::Function { locals, .. } => {
                if let Some(value) = locals.get(name) {
                    self.stack.push(*value);
                } else {
                    self.stack.push(*self.globals.get(name).unwrap())
                }
            }
            Frame::Class { namespace, .. } => {
                if let Some(value) = namespace.get(name) {
                    self.stack.push(*value);
                } else {
                    self.stack.push(*self.globals.get(name).unwrap())
                }
            }
        }
    }
}

// Frame finalization
impl Machine {
    fn finish_function(&mut self, return_value: Value) -> Result<(), String> {
        let frame = self.frames.pop().unwrap();

        let Frame::Function {
            return_behaviour, ..
        } = frame
        else {
            return Err("expected function frame".into());
        };

        let result = match return_behaviour {
            ReturnBehavior::Normal => return_value,

            ReturnBehavior::Constructor(reference) => {
                if !matches!(return_value, Value::None) {
                    return Err("__init__ must return None".into());
                }

                Value::Object(reference)
            }
        };

        self.stack.push(result);

        Ok(())
    }
}

// Callable dispatch
impl Machine {
    fn call_value(
        &mut self,
        program: &CompiledProgram,
        callable: Value,
        mut arguments: Vec<Value>,
    ) -> Result<(), String> {
        match callable {
            Value::BuiltinFunction(function) => {
                let value = function.call(&mut self.runtime, arguments)?;
                self.stack.push(value);
                Ok(())
            }
            Value::BuiltinMethod {
                object_ref,
                mut method,
            } => {
                let value = method.call(&mut self.runtime, object_ref, arguments)?;
                self.stack.push(value);
                Ok(())
            }
            Value::Object(reference) => {
                let object = self.runtime.get(reference);
                match object {
                    Object::Class(Class { attributes, .. }) => {
                        let init = attributes.get("__init__").cloned();
                        self.call_class(program, reference, init, arguments)
                    }
                    Object::Function(VmFunction { function_index }) => self.call_vm_function(
                        program,
                        *function_index,
                        &arguments,
                        ReturnBehavior::Normal,
                    ),
                    Object::BoundMethod(BoundMethod { receiver, function }) => {
                        arguments.push(Value::Object(*receiver));
                        let function = self.runtime.get(*function);
                        if let Object::Function(VmFunction { function_index }) = function {
                            return self.call_vm_function(
                                program,
                                *function_index,
                                &arguments,
                                ReturnBehavior::Normal,
                            );
                        }
                        Ok(())
                    }
                    _ => Err(format!("object {object:?} is not callable")),
                }
            }
            _ => Err(format!("{:?} is not callable", callable)),
        }
    }

    fn call_vm_function(
        &mut self,
        program: &CompiledProgram,
        function_index: usize,
        arguments: &[Value],
        return_behaviour: ReturnBehavior,
    ) -> Result<(), String> {
        let parameters = &program.functions[function_index].parameters;
        if parameters.len() != arguments.len() {
            return Err(format!("function expected {} arguments but {} were provided", parameters.len(), arguments.len()))
        }
        
        let mut locals = HashMap::new();

        for (parameter, argument) in parameters.iter().zip(arguments.iter().rev()) {
            locals.insert(parameter.clone(), *argument);
        }

        let frame = Frame::Function {
            function_index,
            ip: 0,
            locals,
            return_behaviour,
        };
        self.frames.push(frame);
        Ok(())
    }

    fn call_class(
        &mut self,
        program: &CompiledProgram,
        class_reference: ObjectRef,
        init: Option<Value>,
        mut arguments: Vec<Value>,
    ) -> Result<(), String> {
        let instance = Object::Instance(Instance {
            class: class_reference,
            attributes: HashMap::new(),
        });
        let instance_ref = self.runtime.allocate(instance);

        if let Some(Value::Object(function_reference)) = init {
            let function = self.runtime.get(function_reference);
            if let Object::Function(VmFunction { function_index }) = function {
                arguments.push(Value::Object(instance_ref));
                self.call_vm_function(
                    program,
                    *function_index,
                    &arguments,
                    ReturnBehavior::Constructor(instance_ref),
                )?;
            }
            Ok(())
        } else if init.is_none() {
            self.stack.push(Value::Object(instance_ref));
            Ok(())
        } else {
            Err("__init__ is not a supported callable".into())
        }
    }
}

// Helpers
impl Machine {
    fn binary_op<F>(&mut self, operation: F) -> Result<(), String>
    where
        F: FnOnce(&mut Runtime, Value, Value) -> Result<Value, String>,
    {
        let right = self.stack.pop().unwrap();
        let left = self.stack.pop().unwrap();

        let result = operation(&mut self.runtime, left, right)?;

        self.stack.push(result);

        *self.current_frame_mut().ip_mut() += 1;

        Ok(())
    }
}

// Instruction choice
impl Machine {
    fn current_instruction<'a>(&self, program: &'a CompiledProgram) -> Option<&'a Instruction> {
        match self.current_frame() {
            Frame::Module { ip } => program.module.get(*ip),
            Frame::Function {
                function_index, ip, ..
            } => program.functions[*function_index].instructions.get(*ip),
            Frame::Class {
                class_index, ip, ..
            } => program.classes[*class_index].instructions.get(*ip),
        }
    }

    fn current_frame(&self) -> &Frame {
        self.frames.last().unwrap()
    }

    fn current_frame_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap()
    }
}

// Garbage collection
impl Machine {
    fn collect_garbage(&mut self) {
        if self.runtime.should_collect() {
            for reference in self.collect_roots() {
                self.runtime.mark(reference);
            }
            self.runtime.sweep();
        }
    }

    fn collect_roots(&self) -> Vec<ObjectRef> {
        let mut roots = Vec::new();
        for frame in &self.frames {
            match frame {
                Frame::Module { .. } => roots.append(
                    &mut self
                        .globals
                        .values()
                        .filter_map(|value| match value {
                            Value::Object(reference) => Some(reference),
                            _ => None,
                        })
                        .cloned()
                        .collect(),
                ),
                Frame::Function {
                    locals,
                    return_behaviour,
                    ..
                } => {
                    roots.append(
                        &mut locals
                            .values()
                            .filter_map(|value| match value {
                                Value::Object(reference) => Some(reference),
                                _ => None,
                            })
                            .cloned()
                            .collect(),
                    );
                    if let ReturnBehavior::Constructor(reference) = return_behaviour {
                        roots.push(*reference);
                    }
                }
                Frame::Class { namespace, .. } => roots.append(
                    &mut namespace
                        .values()
                        .filter_map(|value| match value {
                            Value::Object(reference) => Some(reference),
                            _ => None,
                        })
                        .cloned()
                        .collect(),
                ),
            }
        }
        roots.append(
            &mut self
                .stack
                .iter()
                .filter_map(|value| match value {
                    Value::Object(reference) => Some(reference),
                    _ => None,
                })
                .cloned()
                .collect(),
        );
        roots
    }
}

#[derive(Debug)]
pub(super) enum Frame {
    Module {
        ip: usize,
    },

    Function {
        function_index: usize,
        ip: usize,
        locals: HashMap<String, Value>,
        return_behaviour: ReturnBehavior,
    },
    Class {
        class_index: usize,
        ip: usize,
        namespace: HashMap<String, Value>,
    },
}

impl Frame {
    fn ip(&self) -> usize {
        match self {
            Frame::Module { ip } => *ip,
            Frame::Function { ip, .. } => *ip,
            Frame::Class { ip, .. } => *ip,
        }
    }

    fn ip_mut(&mut self) -> &mut usize {
        match self {
            Frame::Module { ip } => ip,
            Frame::Function { ip, .. } => ip,
            Frame::Class { ip, .. } => ip,
        }
    }
}

#[derive(Debug)]
pub(super) enum ReturnBehavior {
    Normal,
    Constructor(ObjectRef),
}
