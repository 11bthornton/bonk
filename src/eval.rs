use std::collections::HashMap;

use lalrpop_util::lalrpop_mod;
use rust_decimal::Decimal;
use serde_json::Value as JsonValue;

use crate::ast::Value;

lalrpop_mod!(pub expr);

pub fn json_to_value(v: &JsonValue) -> Result<Value, String> {
    match v {
        JsonValue::Number(n) => n
            .to_string()
            .parse::<Decimal>()
            .map(Value::Num)
            .map_err(|e| format!("invalid number: {e}")),
        JsonValue::String(s) => Ok(Value::Str(s.clone())),
        JsonValue::Bool(b) => Ok(Value::Bool(*b)),
        JsonValue::Object(map) => {
            let mut result = HashMap::new();
            for (k, v) in map {
                result.insert(k.clone(), json_to_value(v)?);
            }
            Ok(Value::Object(result))
        }
        _ => Err(format!("unsupported context value: {v}")),
    }
}

pub fn eval_expr(input: &str, context: &Option<HashMap<String, JsonValue>>) -> Result<String, String> {
    let parser = expr::ProgramParser::new();
    let mut vars = HashMap::new();
    if let Some(ctx) = context {
        for (k, v) in ctx {
            vars.insert(k.clone(), json_to_value(v)?);
        }
    }
    let ast = parser.parse(input).map_err(|e| {
        if input.trim().len() > 0 && !input.trim().ends_with("please") {
            "Parse error: you forgot to say please".to_string()
        } else {
            format!("Parse error: {e}")
        }
    })?;
    let val = ast.eval(&mut vars)?;
    Ok(val.to_string())
}
