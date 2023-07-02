mod openai;

pub use openai::{
    RoleType,
    OError,
    Result,
    ask,
    draw,
    FuncParamUnit,
    FuncParams,
    FuncUnit,
};
