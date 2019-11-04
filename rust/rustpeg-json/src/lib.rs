mod frontend;
use frontend::*;

pub fn parse(s: &str) -> Result<Json, frontend::parser::ParseError> {
    parser::json(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_null() {
        let actual = parse("null").unwrap();
        assert_eq!(actual, Json::Null);
    }

    #[test]
    fn test_true() {
        let actual = parse("true").unwrap();
        assert_eq!(actual, Json::Boolean(true));
    }

    #[test]
    fn test_false() {
        let actual = parse("false").unwrap();
        assert_eq!(actual, Json::Boolean(false));
    }

    #[test]
    fn test_number() {
        let actual = parse("42").unwrap();
        assert_eq!(actual, Json::Number(42.0));
    }

    #[test]
    fn test_string() {
        let actual = parse(r#""TEST""#).unwrap();
        assert_eq!(actual, Json::String("TEST".into()));
    }

    #[test]
    fn test_array() {
        let actual = parse(r#"[1, null, true, "TEST"]"#).unwrap();
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
        let actual = parse(r#"{"key1": [42], "key2": {}}"#).unwrap();
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
        let actual = parse(r#"["\u3042", "TE\nST", "TE\"ST"]"#).unwrap();
        let expected = Json::Array(vec![
            Json::String(String::from("あ")),
            Json::String(String::from("TE\nST")),
            Json::String(String::from(r#"TE"ST"#)),
        ]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_large_json() {
        // https://github.com/prettier/prettier/tree/master/tests/json
        let actual = parse(
            r###"
[
    "JSON Test Pattern pass1",
    {"object with 1 member":["array with 1 element"]},
    {},
    [],
    -42,
    true,
    false,
    null,
    {

        "integer": 1234567890,
        "real": -9876.543210,
        "e": 0.123456789e-12,
        "E": 1.234567890E+34,
        "":  23456789012E66,
        "zero": 0,
        "one": 1,
        "space": " ",
        "quote": "\"",
        "backslash": "\\",
        "controls": "\b\f\n\r\t",
        "slash": "/ & \/",
        "alpha": "abcdefghijklmnopqrstuvwyz",
        "ALPHA": "ABCDEFGHIJKLMNOPQRSTUVWYZ",
        "digit": "0123456789",
        "0123456789": "digit",
        "special": "`1~!@#$%^&*()_+-={':[,]}|;.</>?",
        "hex": "\u0123\u4567\u89AB\uCDEF\uabcd\uef4A",
        "true": true,
        "false": false,
        "null": null,
        "array":[  ],
        "object":{  },
        "address": "50 St. James Street",
        "url": "http://www.JSON.org/",
        "comment": "// /* <!-- --",
        "# -- --> */": " ",
        " s p a c e d " :[1,2 , 3

,

4 , 5        ,          6           ,7        ],"compact":[1,2,3,4,5,6,7],
        "jsontext": "{\"object with 1 member\":[\"array with 1 element\"]}",
        "quotes": "&#34; \u0022 %22 0x22 034 &#x22;",
        "\/\\\"\uCAFE\uBABE\uAB98\uFCDE\ubcda\uef4A\b\f\n\r\t`1~!@#$%^&*()_+-=[]{}|;:',./<>?"
: "A key can be any string"
    },
    0.5 ,98.6
,
99.44
,

1066,
1e1,
0.1e1,
1e-1,
1e00,2e+00,2e-00
,"rosebud"]
"###,
        )
        .unwrap();
        let expected = Json::Array(vec![
            Json::String("JSON Test Pattern pass1".into()),
            Json::Object(Box::new({
                let mut map = HashMap::new();
                map.insert(
                    "object with 1 member".into(),
                    Json::Array(vec![Json::String("array with 1 element".into())]),
                );
                map
            })),
            Json::Object(Box::new(HashMap::new())),
            Json::Array(vec![]),
            Json::Number(-42.0),
            Json::Boolean(true),
            Json::Boolean(false),
            Json::Null,
            Json::Object(Box::new({
                let mut map = HashMap::new();
                map.insert("integer".into(), Json::Number(1234567890.0));
                map.insert("real".into(), Json::Number(-9876.543210));
                map.insert("e".into(), Json::Number(0.123456789e-12));
                map.insert("E".into(), Json::Number(1.234567890E+34));
                map.insert("".into(), Json::Number(23456789012E66));
                map.insert("zero".into(), Json::Number(0.0));
                map.insert("one".into(), Json::Number(1.0));
                map.insert("space".into(), Json::String(" ".into()));
                map.insert("quote".into(), Json::String("\"".into()));
                map.insert("backslash".into(), Json::String("\\".into()));
                map.insert(
                    "controls".into(),
                    Json::String("\u{0008}\u{000C}\n\r\t".into()),
                );
                map.insert("slash".into(), Json::String("/ & /".into()));
                map.insert(
                    "alpha".into(),
                    Json::String("abcdefghijklmnopqrstuvwyz".into()),
                );
                map.insert(
                    "ALPHA".into(),
                    Json::String("ABCDEFGHIJKLMNOPQRSTUVWYZ".into()),
                );
                map.insert("digit".into(), Json::String("0123456789".into()));
                map.insert("0123456789".into(), Json::String("digit".into()));
                map.insert(
                    "special".into(),
                    Json::String("`1~!@#$%^&*()_+-={':[,]}|;.</>?".into()),
                );
                map.insert(
                    "hex".into(),
                    Json::String("\u{0123}\u{4567}\u{89AB}\u{CDEF}\u{abcd}\u{ef4A}".into()),
                );
                map.insert("true".into(), Json::Boolean(true));
                map.insert("false".into(), Json::Boolean(false));
                map.insert("null".into(), Json::Null);
                map.insert("array".into(), Json::Array(vec![]));
                map.insert("object".into(), Json::Object(Box::new(HashMap::new())));
                map.insert("address".into(), Json::String("50 St. James Street".into()));
                map.insert("url".into(), Json::String("http://www.JSON.org/".into()));
                map.insert("comment".into(), Json::String("// /* <!-- --".into()));
                map.insert("# -- --> */".into(), Json::String(" ".into()));
                map.insert(
                    " s p a c e d ".into(),
                    Json::Array(vec![
                        Json::Number(1.0),
                        Json::Number(2.0),
                        Json::Number(3.0),
                        Json::Number(4.0),
                        Json::Number(5.0),
                        Json::Number(6.0),
                        Json::Number(7.0),
                    ]),
                );
                map.insert(
                    "compact".into(),
                    Json::Array(vec![
                        Json::Number(1.0),
                        Json::Number(2.0),
                        Json::Number(3.0),
                        Json::Number(4.0),
                        Json::Number(5.0),
                        Json::Number(6.0),
                        Json::Number(7.0),
                    ]),
                );
                map.insert(
                    "jsontext".into(),
                    Json::String("{\"object with 1 member\":[\"array with 1 element\"]}".into()),
                );
                map.insert(
                    "quotes".into(),
                    Json::String("&#34; \u{0022} %22 0x22 034 &#x22;".into()),
                );
                map.insert(
                    "/\\\"\u{CAFE}\u{BABE}\u{AB98}\u{FCDE}\u{bcda}\u{ef4A}\u{0008}\u{000C}\n\r\t`1~!@#$%^&*()_+-=[]{}|;:',./<>?".into(),
                    Json::String("A key can be any string".into()),
                );
                map
            })),
            Json::Number(0.5),
            Json::Number(98.6),
            Json::Number(99.44),
            Json::Number(1066.0),
            Json::Number(1e1),
            Json::Number(0.1e1),
            Json::Number(1e-1),
            Json::Number(1e00),
            Json::Number(2e+00),
            Json::Number(2e-00),
            Json::String("rosebud".into()),
        ]);
        assert_eq!(actual, expected);
    }
}
