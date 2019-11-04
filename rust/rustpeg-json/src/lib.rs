mod frontend;
pub use frontend::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_null() {
        let actual = parser::json("null").unwrap();
        assert_eq!(actual, Json::Null);
    }

    #[test]
    fn test_true() {
        let actual = parser::json("true").unwrap();
        assert_eq!(actual, Json::Boolean(true));
    }

    #[test]
    fn test_false() {
        let actual = parser::json("false").unwrap();
        assert_eq!(actual, Json::Boolean(false));
    }

    #[test]
    fn test_number() {
        let actual = parser::json("42").unwrap();
        assert_eq!(actual, Json::Number(42.0));
    }

    #[test]
    fn test_string() {
        let actual = parser::json(r#""TEST""#).unwrap();
        assert_eq!(actual, Json::String("TEST".into()));
    }

    #[test]
    fn test_array() {
        let actual = parser::json(r#"[1, null, true, "TEST"]"#).unwrap();
        assert_eq!(
            actual,
            Json::Array(vec![
                Json::Number(1.0),
                Json::Null,
                Json::Boolean(true),
                Json::String("TEST".into())
            ])
        );
    }

    #[test]
    fn test_object() {
        let actual = parser::json(r#"{"key1": [42], "key2": {}}"#).unwrap();
        let expected = Json::Object(Box::new({
            let mut map = HashMap::new();
            map.insert(String::from("key1"), Json::Array(vec![Json::Number(42.0)]));
            map.insert(String::from("key2"), Json::Object(Box::new(HashMap::new())));
            map
        }));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_escape() {
        let actual = parser::json(r#"["\u3042", "TE\nST", "TE\"ST"]"#).unwrap();
        let expected = Json::Array(vec![
            Json::String(String::from("あ")),
            Json::String(String::from("TE\nST")),
            Json::String(String::from(r#"TE"ST"#)),
        ]);
        assert_eq!(actual, expected);
    }
}
