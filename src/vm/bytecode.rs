#[derive(Debug, Clone)]
pub enum Instruction {
    LoadNone,

    LoadClass(usize),

    LoadInteger(i64),
    LoadString(String),

    LoadAttribute(String),
    StoreAttribute(String),

    GetItem,
    SetItem,

    BuildList(usize),

    StoreName(String),
    LoadName(String),

    Call(usize),
    LoadFunction(usize),
    Return,

    Add,
    Subtract,
    Multiply,
    Divide,

    Greater,
    Less,
    Equal,

    Jump(usize),
    JumpIfFalse(usize),

    Pop,
}

#[derive(Debug)]
pub struct CompiledProgram {
    pub module: Vec<Instruction>,
    pub functions: Vec<CompiledFunction>,
    pub classes: Vec<CompiledClass>,
}

#[derive(Debug, Clone)]
pub struct CompiledFunction {
    pub name: String,
    pub parameters: Vec<String>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug)]
pub struct CompiledClass {
    pub name: String,
    pub instructions: Vec<Instruction>,
}
