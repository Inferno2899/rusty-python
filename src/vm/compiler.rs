use crate::frontend::{BinaryOperator, Expression, Statement};
use crate::vm::{CompiledClass, CompiledFunction, CompiledProgram, Instruction};

pub struct Compiler {
    functions: Vec<CompiledFunction>,
    classes: Vec<CompiledClass>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            classes: Vec::new(),
        }
    }

    pub fn compile(mut self, program: &[Statement]) -> Result<CompiledProgram, String> {
        let mut module = Vec::new();

        for statement in program {
            self.compile_statement(statement, &mut module)?;
        }

        Ok(CompiledProgram {
            module,
            functions: self.functions,
            classes: self.classes,
        })
    }

    fn compile_statement(
        &mut self,
        smnt: &Statement,
        instructions: &mut Vec<Instruction>,
    ) -> Result<(), String> {
        match smnt {
            Statement::Assignment { name, value } => {
                match name {
                    Expression::Variable(name) => {
                        self.compile_expression(value, instructions)?;
                        instructions.push(Instruction::StoreName(name.clone()))
                    }
                    Expression::Index { object, index } => {
                        self.compile_expression(value, instructions)?;
                        self.compile_expression(object, instructions)?;
                        self.compile_expression(index, instructions)?;
                        instructions.push(Instruction::SetItem);
                    }
                    Expression::Attribute { object, name } => {
                        self.compile_expression(value, instructions)?;
                        self.compile_expression(object, instructions)?;
                        instructions.push(Instruction::StoreAttribute(name.clone()));
                    }
                    _ => return Err(format!("invalid assignment target: {:?}", name)),
                }
                Ok(())
            }
            Statement::Expression(expr) => {
                self.compile_expression(expr, instructions)?;
                instructions.push(Instruction::Pop);
                Ok(())
            }
            Statement::If { condition, body } => {
                self.compile_expression(condition, instructions)?;

                let jump_index = instructions.len();
                instructions.push(Instruction::JumpIfFalse(0));

                for smnt in body {
                    self.compile_statement(smnt, instructions)?;
                }

                let jump_destination = instructions.len();
                instructions[jump_index] = Instruction::JumpIfFalse(jump_destination);

                Ok(())
            }
            Statement::While { condition, body } => {
                let backward_jump_index = instructions.len();
                self.compile_expression(condition, instructions)?;

                let jump_index = instructions.len();
                instructions.push(Instruction::JumpIfFalse(0));

                for smnt in body {
                    self.compile_statement(smnt, instructions)?;
                }

                instructions.push(Instruction::Jump(backward_jump_index));

                let jump_destination = instructions.len();
                instructions[jump_index] = Instruction::JumpIfFalse(jump_destination);

                Ok(())
            }
            Statement::Function {
                name,
                parameters,
                body,
            } => {
                let mut function_instructions = Vec::new();

                for statement in body {
                    self.compile_statement(statement, &mut function_instructions)?;
                }

                let function = CompiledFunction {
                    name: name.clone(),
                    parameters: parameters.clone(),
                    instructions: function_instructions,
                };

                let function_index = self.functions.len();

                self.functions.push(function);

                instructions.push(Instruction::LoadFunction(function_index));
                instructions.push(Instruction::StoreName(name.clone()));

                Ok(())
            }
            Statement::Class { name, body } => {
                let mut class_instructions = Vec::new();

                for statement in body {
                    self.compile_statement(statement, &mut class_instructions)?;
                }

                let class = CompiledClass {
                    name: name.clone(),
                    instructions: class_instructions,
                };

                let class_index = self.classes.len();

                self.classes.push(class);

                instructions.push(Instruction::LoadClass(class_index));
                instructions.push(Instruction::StoreName(name.clone()));

                Ok(())
            }
            Statement::Return(expr) => {
                self.compile_expression(expr, instructions)?;
                instructions.push(Instruction::Return);
                Ok(())
            }
            Statement::Pass => Ok(()),
        }
    }

    fn compile_expression(
        &mut self,
        expr: &Expression,
        instructions: &mut Vec<Instruction>,
    ) -> Result<(), String> {
        match expr {
            Expression::None => instructions.push(Instruction::LoadNone),
            Expression::Integer(value) => instructions.push(Instruction::LoadInteger(*value)),
            Expression::Str(value) => instructions.push(Instruction::LoadString(value.clone())),
            Expression::List(items) => {
                for item in items {
                    self.compile_expression(item, instructions)?;
                }
                instructions.push(Instruction::BuildList(items.len()));
            }
            Expression::Index { object, index } => {
                self.compile_expression(object, instructions)?;
                self.compile_expression(index, instructions)?;
                instructions.push(Instruction::GetItem);
            }
            Expression::Variable(name) => instructions.push(Instruction::LoadName(name.clone())),
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                self.compile_expression(left, instructions)?;
                self.compile_expression(right, instructions)?;
                match operator {
                    BinaryOperator::Plus => instructions.push(Instruction::Add),
                    BinaryOperator::Minus => instructions.push(Instruction::Subtract),
                    BinaryOperator::Multiply => instructions.push(Instruction::Multiply),
                    BinaryOperator::Divide => instructions.push(Instruction::Divide),
                    BinaryOperator::Greater => instructions.push(Instruction::Greater),
                    BinaryOperator::Less => instructions.push(Instruction::Less),
                    BinaryOperator::Equal => instructions.push(Instruction::Equal),
                }
            }
            Expression::Call {
                function,
                arguments,
            } => {
                self.compile_expression(function, instructions)?;
                for argument in arguments {
                    self.compile_expression(argument, instructions)?;
                }
                instructions.push(Instruction::Call(arguments.len()));
            }
            Expression::Attribute { object, name } => {
                self.compile_expression(object, instructions)?;
                instructions.push(Instruction::LoadAttribute(name.clone()));
            }
        }
        Ok(())
    }
}
