use std::collections::HashMap;
use std::fmt;
use rust_decimal::Decimal;

#[derive(Clone, Debug)]
pub enum Value {
    Num(Decimal),
    Str(String),
    Bool(bool),
    Object(HashMap<String, Value>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Num(n) => write!(f, "{n}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Object(map) => {
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{k}: {v}")?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl Value {
    fn as_num(&self) -> Result<Decimal, String> {
        match self {
            Value::Num(n) => Ok(*n),
            _ => Err(format!("expected number, got {self}")),
        }
    }

    fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Num(n) => !n.is_zero(),
            Value::Str(s) => !s.is_empty(),
            Value::Object(_) => true,
        }
    }
}

pub enum Expr {
    Num(Decimal),
    Str(String),
    Bool(bool),
    Var(String),
    Access(Box<Expr>, String),
    Neg(Box<Expr>),
    Not(Box<Expr>),
    BinOp(Box<Expr>, Op, Box<Expr>),
    Cmp(Box<Expr>, CmpOp, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
}

pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

pub enum CmpOp {
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
}

impl Expr {
    pub fn eval(&self, vars: &mut HashMap<String, Value>) -> Result<Value, String> {
        match self {
            Expr::Num(n) => Ok(Value::Num(*n)),
            Expr::Str(s) => Ok(Value::Str(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Var(name) => vars.get(name)
                .cloned()
                .ok_or_else(|| format!("undefined variable: {name}")),
            Expr::Neg(e) => Ok(Value::Num(-e.eval(vars)?.as_num()?)),
            Expr::Not(e) => Ok(Value::Bool(!e.eval(vars)?.is_truthy())),
            Expr::Access(expr, field) => {
                let val = expr.eval(vars)?;
                match val {
                    Value::Object(map) => map.get(field)
                        .cloned()
                        .ok_or_else(|| format!("no field '{field}' on object")),
                    _ => Err(format!("cannot access field '{field}' on {val}")),
                }
            }
            Expr::BinOp(l, op, r) => {
                let lv = l.eval(vars)?;
                let rv = r.eval(vars)?;
                match op {
                    Op::Add => match (&lv, &rv) {
                        (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{a}{b}"))),
                        (Value::Str(a), b) => Ok(Value::Str(format!("{a}{b}"))),
                        (a, Value::Str(b)) => Ok(Value::Str(format!("{a}{b}"))),
                        _ => Ok(Value::Num(lv.as_num()? + rv.as_num()?)),
                    },
                    Op::Sub => Ok(Value::Num(lv.as_num()? - rv.as_num()?)),
                    Op::Mul => Ok(Value::Num(lv.as_num()? * rv.as_num()?)),
                    Op::Div => {
                        let d = rv.as_num()?;
                        if d.is_zero() {
                            return Err("division by zero".to_string());
                        }
                        Ok(Value::Num(lv.as_num()?.checked_div(d)
                            .ok_or_else(|| "division overflow".to_string())?))
                    }
                    Op::Mod => {
                        let d = rv.as_num()?;
                        if d.is_zero() {
                            return Err("modulo by zero".to_string());
                        }
                        Ok(Value::Num(lv.as_num()? % d))
                    }
                    Op::Pow => {
                        use rust_decimal::MathematicalOps;
                        Ok(Value::Num(lv.as_num()?.powd(rv.as_num()?)))
                    }
                }
            }
            Expr::Cmp(l, op, r) => {
                let lv = l.eval(vars)?;
                let rv = r.eval(vars)?;
                let result = match op {
                    CmpOp::Eq => match (&lv, &rv) {
                        (Value::Num(a), Value::Num(b)) => a == b,
                        (Value::Str(a), Value::Str(b)) => a == b,
                        (Value::Bool(a), Value::Bool(b)) => a == b,
                        _ => false,
                    },
                    CmpOp::Neq => match (&lv, &rv) {
                        (Value::Num(a), Value::Num(b)) => a != b,
                        (Value::Str(a), Value::Str(b)) => a != b,
                        (Value::Bool(a), Value::Bool(b)) => a != b,
                        _ => true,
                    },
                    CmpOp::Lt => lv.as_num()? < rv.as_num()?,
                    CmpOp::Gt => lv.as_num()? > rv.as_num()?,
                    CmpOp::Lte => lv.as_num()? <= rv.as_num()?,
                    CmpOp::Gte => lv.as_num()? >= rv.as_num()?,
                };
                Ok(Value::Bool(result))
            }
            Expr::And(l, r) => {
                if l.eval(vars)?.is_truthy() {
                    r.eval(vars)
                } else {
                    Ok(Value::Bool(false))
                }
            }
            Expr::Or(l, r) => {
                let lv = l.eval(vars)?;
                if lv.is_truthy() {
                    Ok(lv)
                } else {
                    r.eval(vars)
                }
            }
            Expr::If(cond, then_br, else_br) => {
                if cond.eval(vars)?.is_truthy() {
                    then_br.eval(vars)
                } else {
                    else_br.eval(vars)
                }
            }
            Expr::Let(name, value, body) => {
                let v = value.eval(vars)?;
                let old = vars.insert(name.clone(), v);
                let result = body.eval(vars);
                match old {
                    Some(prev) => vars.insert(name.clone(), prev),
                    None => vars.remove(name),
                };
                result
            }
        }
    }
}
