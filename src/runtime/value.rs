use super::builtin::{BuiltinFunction, BuiltinMethod};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ObjectRef(pub(crate) usize);

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Value {
    Integer(i64),
    Double(f64),
    Boolean(bool),
    None,
    Object(ObjectRef),
    BuiltinFunction(BuiltinFunction),

    BuiltinMethod {
        object_ref: ObjectRef,
        method: BuiltinMethod,
    },
}
