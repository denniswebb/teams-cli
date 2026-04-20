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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // We can't easily capture stdout from print_plain since it uses println!
    // directly. Instead, we test the serialization logic that print_plain
    // relies on and verify the function doesn't panic on various inputs.

    #[test]
    fn print_plain_object_does_not_panic() {
        let val = json!({"name": "Alice", "age": 30});
        print_plain(&val);
    }

    #[test]
    fn print_plain_array_does_not_panic() {
        let val = json!([
            {"id": "1", "name": "Alice"},
            {"id": "2", "name": "Bob"}
        ]);
        print_plain(&val);
    }

    #[test]
    fn print_plain_simple_value_does_not_panic() {
        print_plain(&json!("hello"));
        print_plain(&json!(42));
        print_plain(&json!(true));
        print_plain(&json!(null));
    }

    #[test]
    fn print_plain_empty_object_does_not_panic() {
        print_plain(&json!({}));
    }

    #[test]
    fn print_plain_empty_array_does_not_panic() {
        print_plain(&json!([]));
    }

    #[test]
    fn print_plain_nested_object_does_not_panic() {
        let val = json!({"outer": {"inner": "value"}});
        print_plain(&val);
    }

    #[test]
    fn print_plain_array_of_non_objects_does_not_panic() {
        // Array items that are not objects are silently skipped
        let val = json!(["a", "b", "c"]);
        print_plain(&val);
    }
}
