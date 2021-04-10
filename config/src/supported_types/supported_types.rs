use crate::*;

#[derive(Debug, Clone)]
pub enum SupportedTypes {
    String(ConfigArgString),
    Bool(ConfigArgBool),
    Integer(ConfigArgInteger),
    Struct(Box<ConfigStruct>),
    CheckableStruct(Box<ConfigCheckableStruct>),
    Vec(Box<ConfigVec>),
}
