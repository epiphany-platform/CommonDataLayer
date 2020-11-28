pub fn validate_schema(schema: &str) -> bool {
    schema
        .chars()
        .all(|c| c == '_' || c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("test" => true)]
    #[test_case("test;" => false)]
    #[test_case("test4" => true)]
    #[test_case("te_st4" => true)]
    #[test_case("te st4" => false)]
    #[test_case("test4`" => false)]
    #[test_case("test4\n" => false)]
    #[test_case("test4$1" => false)]
    fn validate_schema_tests(schema: &str) -> bool {
        validate_schema(schema)
    }
}
