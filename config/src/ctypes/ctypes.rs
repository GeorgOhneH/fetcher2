use crate::*;

#[derive(Debug, Clone)]
pub enum CTypes {
    String(CString),
    Bool(CBool),
    Integer(CInteger),
    Struct(Box<CStruct>),
    CheckableStruct(Box<CCheckableStruct>),
    Vec(Box<CVec>),
    Enum(CEnum),
}
