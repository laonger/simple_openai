mod openai;
mod utils;
mod errors;

pub use openai::{
    RoleType,
    OError,
    Result,
    ask,
    draw,
    speak,
    OpenAISpeedVoice,
    RequestMessageUnit,
    ResponseMessageUnit,
    FuncParamUnit,
    FuncParams,
    FuncUnit,
};
