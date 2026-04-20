use serde::Serialize;

pub fn print_plain<T: Serialize>(value: &T) {
    let json = match serde_json::to_value(value) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to serialize: {e}");
            return;
        }
    };
    match json {
        serde_json::Value::Object(map) => {
            for (k, v) in &map {
                match v {
                    serde_json::Value::String(s) => println!("{k}\t{s}"),
                    serde_json::Value::Null => println!("{k}\t"),
                    other => println!("{k}\t{other}"),
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in &arr {
                if let serde_json::Value::Object(map) = item {
                    let vals: Vec<String> = map
                        .values()
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Null => String::new(),
                            other => other.to_string(),
                        })
                        .collect();
                    println!("{}", vals.join("\t"));
                }
            }
        }
        other => println!("{other}"),
    }
}
