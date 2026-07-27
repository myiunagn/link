use std::io::{self, Read, Write};
use serde_json::{Value, json};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    
    let req: Value = match serde_json::from_str(&input) {
        Ok(v) => v,
        Err(e) => {
            let resp = json!({"error": format!("Failed to parse request: {}", e)});
            println!("{}", resp.to_string());
            return;
        }
    };
    
    let function = req["function"].as_str().unwrap_or("");
    let args = req["args"].as_array().cloned().unwrap_or_default();
    
    let result = match function {
        "add" => {
            if args.len() < 2 {
                json!({"error": "add requires 2 arguments"})
            } else {
                let a = args[0].as_i64().or_else(|| args[0].as_f64().map(|f| f as i64)).unwrap_or(0);
                let b = args[1].as_i64().or_else(|| args[1].as_f64().map(|f| f as i64)).unwrap_or(0);
                json!(a + b)
            }
        }
        "subtract" => {
            if args.len() < 2 {
                json!({"error": "subtract requires 2 arguments"})
            } else {
                let a = args[0].as_i64().or_else(|| args[0].as_f64().map(|f| f as i64)).unwrap_or(0);
                let b = args[1].as_i64().or_else(|| args[1].as_f64().map(|f| f as i64)).unwrap_or(0);
                json!(a - b)
            }
        }
        "multiply" => {
            if args.len() < 2 {
                json!({"error": "multiply requires 2 arguments"})
            } else {
                let a = args[0].as_i64().or_else(|| args[0].as_f64().map(|f| f as i64)).unwrap_or(0);
                let b = args[1].as_i64().or_else(|| args[1].as_f64().map(|f| f as i64)).unwrap_or(0);
                json!(a * b)
            }
        }
        "greet" => {
            if args.is_empty() {
                json!({"error": "greet requires 1 argument"})
            } else {
                let name = args[0].as_str().unwrap_or("unknown");
                json!(format!("Hello from Rust, {}!", name))
            }
        }
        _ => {
            json!({"error": format!("Unknown function: {}", function)})
        }
    };
    
    let resp = json!({"result": result});
    println!("{}", resp.to_string());
    io::stdout().flush().unwrap();
}