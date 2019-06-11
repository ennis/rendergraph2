//! Op parameters

pub enum TypeDesc
{
    Float,
    Int,
}

pub struct Expression {
    s: String,
}

pub struct Parameter<'a> {
    /// Name of the parameter.
    pub name: &'a str,
    /// Expected type
    pub ty: TypeDesc,
    /// Expression to evaluate
    pub expr: Expression,
}

/*
impl Parameter {

}*/