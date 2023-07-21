mod openai;

pub use openai::{
    RoleType,
    OError,
    Result,
    ask,
    draw,
    RequestMessageUnit,
    ResponseMessageUnit,
    FuncParamUnit,
    FuncParams,
    FuncUnit,
};
