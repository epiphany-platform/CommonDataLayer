use std::{
    panic, process,
    sync::PoisonError,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn abort_on_poison<T>(_e: PoisonError<T>) -> T {
    tracing::error!("Encountered mutex poisoning. Aborting.");
    process::abort();
}

pub fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64
}

pub fn set_aborting_panic_hook() {
    let orig_panic_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        orig_panic_hook(info);
        process::abort();
    }));
}

pub mod serde_json {
    use ::serde_json::Value;
    use itertools::Itertools;
    use serde::Serialize;

    pub fn to_string_sorted_pretty<T>(t: &T) -> ::serde_json::Result<String>
    where
        T: Serialize,
    {
        let json = ::serde_json::to_value(t)?;
        let sorted = sort_value(json);
        let sorted_string = ::serde_json::to_string_pretty(&sorted)?;

        Ok(sorted_string)
    }

    fn sort_value(json: Value) -> Value {
        match json {
            Value::Object(obj) => Value::Object(
                obj.into_iter()
                    .sorted_by(|(k_a, _), (k_b, _)| Ord::cmp(k_a, k_b))
                    .map(|(k, v)| (k, sort_value(v)))
                    .collect(),
            ),
            Value::Array(arr) => Value::Array(arr.into_iter().map(sort_value).collect()),
            other => other,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json::json;
        use test_case::test_case;

        #[test_case(json!({"b": 1, "c": 2, "a": 3,}) => "{\n  \"a\": 3,\n  \"b\": 1,\n  \"c\": 2\n}" ; "simple")]
        #[test_case(json!([{"b": 1, "c": 2, "a": 3,}]) => "[\n  {\n    \"a\": 3,\n    \"b\": 1,\n    \"c\": 2\n  }\n]" ; "in array")]
        #[test_case(json!({"foo": {"b": 1, "c": 2, "a": 3,}}) => "{\n  \"foo\": {\n    \"a\": 3,\n    \"b\": 1,\n    \"c\": 2\n  }\n}" ; "nested object")]
        fn it_sorts(input: Value) -> String {
            let sorted = to_string_sorted_pretty(&input).unwrap();

            sorted
        }
    }
}
