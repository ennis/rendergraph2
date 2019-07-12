use crate::error::Error;
use rlua::ToLua;

//--------------------------------------------------------------------------------------------------
pub enum Value {
    Number(f64),
    String(String),
}

impl Value {
    pub fn as_number(&self) -> Result<f64, Error> {
        match self {
            Value::Number(v) => Ok(*v),
            _ => Err(Error::TypeError),
        }
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Number(v)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Number(v as f64)
    }
}

impl<'lua> From<rlua::Value<'lua>> for Value {
    fn from(v: rlua::Value) -> Self {
        match v {
            rlua::Value::Number(n) => Value::Number(n),
            rlua::Value::String(s) => Value::String(s.to_str().unwrap().to_string()),
            _ => unimplemented!(),
        }
    }
}

impl<'lua> ToLua<'lua> for Value {
    fn to_lua(self, ctx: rlua::Context<'lua>) -> rlua::Result<rlua::Value<'lua>> {
        match self {
            Value::String(s) => Ok(rlua::Value::String(ctx.create_string(&s)?)),
            Value::Number(num) => Ok(rlua::Value::Number(num)),
            _ => unimplemented!(),
        }
    }
}

//--------------------------------------------------------------------------------------------------
pub struct Expr {
    code: String,
}

impl Expr {
    pub fn new(code: impl Into<String>) -> Expr {
        Expr { code: code.into() }
    }

    pub fn code(&self) -> &str {
        &self.code
    }
}
