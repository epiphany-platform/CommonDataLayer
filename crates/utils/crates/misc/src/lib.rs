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

    #[derive(Default, Debug, Clone, Copy)]
    pub struct SortSettings {
        pub pretty: bool,
        pub sort_arrays: bool,
    }

    pub fn to_string_sorted<T>(t: &T, settings: SortSettings) -> ::serde_json::Result<String>
    where
        T: Serialize,
    {
        let json = ::serde_json::to_value(t)?;
        let sorted = sort_value(json, settings);

        let sorted_string = if settings.pretty {
            ::serde_json::to_string_pretty(&sorted)
        } else {
            ::serde_json::to_string(&sorted)
        }?;

        Ok(sorted_string)
    }

    fn sort_value(json: Value, settings: SortSettings) -> Value {
        match json {
            Value::Object(obj) => Value::Object(
                obj.into_iter()
                    .map(|(k, v)| (k, sort_value(v, settings)))
                    .sorted_by(|(k_a, _), (k_b, _)| Ord::cmp(k_a, k_b))
                    .collect(),
            ),
            Value::Array(arr) if settings.sort_arrays => Value::Array(
                arr.into_iter()
                    .map(|v| sort_value(v, settings))
                    .sorted_by(|a, b| {
                        let sa = ::serde_json::to_string(&a).unwrap_or_default();
                        let sb = ::serde_json::to_string(&b).unwrap_or_default();
                        Ord::cmp(&sa, &sb)
                    })
                    .collect(),
            ),
            Value::Array(arr) => {
                Value::Array(arr.into_iter().map(|v| sort_value(v, settings)).collect())
            }
            other => other,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json::json;
        use test_case::test_case;

        #[test_case(json!({"b": 1, "c": 2, "a": 3,}), SortSettings::default() => r#"{"a":3,"b":1,"c":2}"# ; "simple")]
        #[test_case(json!([{"b": 1, "c": 2, "a": 3,}]), SortSettings::default() => r#"[{"a":3,"b":1,"c":2}]"# ; "in array")]
        #[test_case(json!({"foo": {"b": 1, "c": 2, "a": 3,}}), SortSettings::default() => r#"{"foo":{"a":3,"b":1,"c":2}}"# ; "nested object")]
        fn it_sorts(input: Value, settings: SortSettings) -> String {
            to_string_sorted(&input, settings).unwrap()
        }
    }
}
