use comfy_table::{presets::UTF8_FULL, Table};
use serde::Serialize;

#[allow(dead_code)]
pub fn print_table<T: Serialize>(value: &T, columns: &[&str]) {
    let json = match serde_json::to_value(value) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to serialize: {e}");
            return;
        }
    };

    let items = match json {
        serde_json::Value::Array(arr) => arr,
        other => vec![other],
    };

    if items.is_empty() {
        println!("(no results)");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(columns.iter().map(|c| c.to_uppercase()));

    for item in &items {
        let row: Vec<String> = columns
            .iter()
            .map(|col| {
                item.get(col)
                    .or_else(|| {
                        // Try camelCase version
                        let camel = to_camel_case(col);
                        item.get(camel.as_str())
                    })
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Null => String::new(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Number(n) => n.to_string(),
                        other => other.to_string(),
                    })
                    .unwrap_or_default()
            })
            .collect();
        table.add_row(row);
    }

    println!("{table}");
}

fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camel_case_display_name() {
        assert_eq!(to_camel_case("display_name"), "displayName");
    }

    #[test]
    fn camel_case_single_word() {
        assert_eq!(to_camel_case("id"), "id");
    }

    #[test]
    fn camel_case_thread_id() {
        assert_eq!(to_camel_case("thread_id"), "threadId");
    }

    #[test]
    fn camel_case_empty_string() {
        assert_eq!(to_camel_case(""), "");
    }

    #[test]
    fn camel_case_multiple_underscores() {
        assert_eq!(to_camel_case("a_b_c"), "aBC");
    }

    #[test]
    fn camel_case_trailing_underscore() {
        // Trailing underscore: capitalize_next is set but no char follows
        assert_eq!(to_camel_case("foo_"), "foo");
    }

    #[test]
    fn camel_case_leading_underscore() {
        // Leading underscore: first real char gets capitalized
        assert_eq!(to_camel_case("_foo"), "Foo");
    }

    #[test]
    fn camel_case_double_underscore() {
        // Double underscore: second underscore sets capitalize_next again, net effect same
        assert_eq!(to_camel_case("a__b"), "aB");
    }

    #[test]
    fn camel_case_already_mixed() {
        // "already_camelCase" -> underscore capitalizes 'c' -> "alreadyCamelCase"
        // Note: the existing uppercase letters in "Case" are preserved as-is
        assert_eq!(to_camel_case("already_camelCase"), "alreadyCamelCase");
    }
}
