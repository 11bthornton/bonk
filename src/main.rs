use std::collections::HashMap;
use std::io::{self, Write, BufRead};

use axum::{Router, Json, response::Html, routing::{get, post}};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use rust_decimal::Decimal;
use lalrpop_util::lalrpop_mod;

mod ast;
use ast::Value;

lalrpop_mod!(pub expr);

fn json_to_value(v: &JsonValue) -> Result<Value, String> {
    match v {
        JsonValue::Number(n) => n.to_string().parse::<Decimal>().map(Value::Num).map_err(|e| format!("invalid number: {e}")),
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

fn eval_expr(input: &str, context: &Option<HashMap<String, JsonValue>>) -> Result<String, String> {
    let parser = expr::ExprParser::new();
    let mut vars = HashMap::new();
    if let Some(ctx) = context {
        for (k, v) in ctx {
            vars.insert(k.clone(), json_to_value(v)?);
        }
    }
    let ast = parser.parse(input).map_err(|e| format!("Parse error: {e}"))?;
    let val = ast.eval(&mut vars)?;
    Ok(val.to_string())
}

fn repl() {
    let mut vars = HashMap::new();
    let parser = expr::ExprParser::new();
    let stdin = io::stdin();

    loop {
        print!("ballsack> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap() == 0 {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match parser.parse(line) {
            Ok(ast) => match ast.eval(&mut vars) {
                Ok(val) => println!("{val}"),
                Err(e) => eprintln!("Runtime error: {e}"),
            },
            Err(e) => eprintln!("Parse error: {e}"),
        }
    }
}

#[derive(Deserialize)]
struct EvalRequest {
    expr: String,
    #[serde(default)]
    context: Option<HashMap<String, JsonValue>>,
}

#[derive(Serialize)]
struct EvalResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

async fn handle_eval(Json(req): Json<EvalRequest>) -> Json<EvalResponse> {
    match eval_expr(&req.expr, &req.context) {
        Ok(result) => Json(EvalResponse { result: Some(result), error: None }),
        Err(e) => Json(EvalResponse { result: None, error: Some(e) }),
    }
}

async fn handle_index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn serve() {
    let app = Router::new()
        .route("/", get(handle_index))
        .route("/eval", post(handle_eval));

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    println!("ballsack server running at http://localhost:{port}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[tokio::main]
async fn main() {
    if std::env::args().nth(1).as_deref() == Some("serve") {
        serve().await;
    } else {
        repl();
    }
}
