use crate::communication::resolution::Resolution;
use crate::communication::GenericMessage;
use crate::output::victoria_metrics::config::VictoriaMetricsConfig;
use crate::output::OutputPlugin;
use itertools::Itertools;
use log::error;
use reqwest::Url;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::iter;
use thiserror::Error as DeriveError;
use url::ParseError;
use utils::metrics::counter;
use uuid::Uuid;

pub mod config;

pub struct VictoriaMetricsOutputPlugin {
    client: Client,
    url: Url,
}

#[derive(Debug, DeriveError)]
pub enum Error {
    #[error("Url was invalid `{0}`")]
    InvalidUrl(ParseError),
    #[error("Data section must be deserializable to JSON")]
    DataIsNotValidJson(serde_json::Error),
    #[error("Data section must be a JSON Object")]
    DataIsNotAJsonObject,
    #[error("Value cannot be an object, array or null")]
    InvalidFieldType,
    #[error("Cannot handle empty payload")]
    EmptyFields,
}

impl VictoriaMetricsOutputPlugin {
    pub fn new(config: VictoriaMetricsConfig) -> Result<VictoriaMetricsOutputPlugin, Error> {
        let client = Client::new();

        Ok(VictoriaMetricsOutputPlugin {
            client,
            url: config.url.join("write").map_err(Error::InvalidUrl)?,
        })
    }
}

#[async_trait::async_trait]
impl OutputPlugin for VictoriaMetricsOutputPlugin {
    async fn handle_message(&self, msg: GenericMessage) -> Resolution {
        let mut url = self.url.clone();

        url.set_query(Some(&format!("db={}", msg.schema_id)));

        let GenericMessage {
            object_id,
            schema_id,
            timestamp,
            payload,
        } = msg;

        match build_line_protocol(schema_id, object_id, timestamp, &payload) {
            Ok(line_protocol) => send_data(url, &self.client, line_protocol).await,
            Err(err) => {
                let context = String::from_utf8_lossy(&payload).to_string();

                error!(
                    "Failed to convert payload to line_protocol, cause `{}`, context `{}`",
                    err, context
                );
                Resolution::UserFailure {
                    description: err.to_string(),
                    context,
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "Victoria metrics"
    }
}

fn build_line_protocol(
    measurement: Uuid,
    tag: Uuid,
    timestamp: i64,
    payload: &[u8],
) -> Result<String, Error> {
    let fields_raw: Value = serde_json::from_slice(payload).map_err(Error::DataIsNotValidJson)?;

    if let Value::Object(obj) = fields_raw {
        if obj.is_empty() {
            return Err(Error::EmptyFields);
        }

        let len = obj.len();

        let fields = obj
            .into_iter()
            .map(|(key, value)| {
                if value.is_array() || value.is_object() || value.is_null() {
                    Err(Error::InvalidFieldType)
                } else {
                    Ok(format!("{}={}", key, value))
                }
            })
            .interleave(iter::repeat_with(|| Ok(",".to_string())).take(len - 1))
            .collect::<Result<String, Error>>()?;

        Ok(format!(
            "{},object_id={} {} {}",
            measurement, tag, fields, timestamp
        ))
    } else {
        Err(Error::DataIsNotAJsonObject)
    }
}

async fn send_data(url: Url, client: &Client, line_protocol: String) -> Resolution {
    match client.post(url).body(line_protocol).send().await {
        Ok(response) => {
            if matches!(response.status(), StatusCode::OK | StatusCode::NO_CONTENT) {
                counter!("cdl.command-service.store.victoria_metrics", 1);

                Resolution::Success
            } else {
                Resolution::StorageLayerFailure {
                    description: response
                        .text()
                        .await
                        .unwrap_or_else(|err| format!("No description. Error `{}`", err)),
                }
            }
        }
        Err(err) => {
            error!("Failed to send data to vm `{}`", err);
            Resolution::CommandServiceFailure
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod describe_build_line_protocol {
        use super::*;
        use test_case::test_case;
        use uuid::Uuid;

        #[test_case("{}"                => matches Err(Error::EmptyFields))]
        #[test_case("{ \"y01\": {} }"   => matches Err(Error::InvalidFieldType))]
        #[test_case("y00=32q,y01=14.0f" => matches Err(Error::DataIsNotValidJson(_)))]
        #[test_case("[ 1, 13 ]"         => matches Err(Error::DataIsNotAJsonObject))]
        fn produces_desired_errors(payload: &str) -> Result<String, Error> {
            build_line_protocol(
                Uuid::default(),
                Uuid::default(),
                i64::default(),
                payload.as_bytes(),
            )
        }

        struct TestCase {
            object_id: &'static str,
            schema_id: &'static str,
            version: i64,
            payload: &'static str,
        }

        const TEST_CASE_1: TestCase = TestCase {
            object_id: "00000000-0000-0000-0000-000000000000",
            schema_id: "00000000-0000-0000-0000-000000000000",
            version: 0,
            payload: r#"{ "y01": 13.4 }"#,
        };

        const TEST_CASE_2: TestCase = TestCase {
            object_id: "00000000-0000-0000-0000-000000000000",
            schema_id: "00000000-0000-0000-0000-000000000000",
            version: 1603887165,
            payload: r#"{ "y01": 13.4 }"#,
        };

        const TEST_CASE_3: TestCase = TestCase {
            object_id: "00000000-0000-0000-0000-000000000000",
            schema_id: "10b7a9cd-0daf-4cb6-a7ef-b9db6058a2d3",
            version: 0,
            payload: r#"{ "y01": 13.4 }"#,
        };

        const TEST_CASE_4: TestCase = TestCase {
            object_id: "85cf3b2e-0c9c-40e1-ade3-4596a2e1d9b6",
            schema_id: "00000000-0000-0000-0000-000000000000",
            version: 0,
            payload: r#"{ "y01": 13.4 }"#,
        };

        const TEST_CASE_5: TestCase = TestCase {
            object_id: "00000000-0000-0000-0000-000000000000",
            schema_id: "00000000-0000-0000-0000-000000000000",
            version: 0,
            payload: r#"{ "y01": 13.4, "y02": 12.2 }"#,
        };

        const TEST_CASE_6: TestCase = TestCase {
            object_id: "00000000-0000-0000-0000-000000000000",
            schema_id: "00000000-0000-0000-0000-000000000000",
            version: 0,
            payload: r#"{ "y01": true }"#,
        };

        const TEST_CASE_7: TestCase = TestCase {
            object_id: "00000000-0000-0000-0000-000000000000",
            schema_id: "00000000-0000-0000-0000-000000000000",
            version: 0,
            payload: r#"{ "y01": "some-msg" }"#,
        };

        #[test_case(TEST_CASE_1 => "00000000-0000-0000-0000-000000000000,object_id=00000000-0000-0000-0000-000000000000 y01=13.4 0")]
        #[test_case(TEST_CASE_2 => "00000000-0000-0000-0000-000000000000,object_id=00000000-0000-0000-0000-000000000000 y01=13.4 1603887165" ; "changes timestamp")]
        #[test_case(TEST_CASE_3 => "10b7a9cd-0daf-4cb6-a7ef-b9db6058a2d3,object_id=00000000-0000-0000-0000-000000000000 y01=13.4 0"          ; "changes schema_id")]
        #[test_case(TEST_CASE_4 => "00000000-0000-0000-0000-000000000000,object_id=85cf3b2e-0c9c-40e1-ade3-4596a2e1d9b6 y01=13.4 0"          ; "changes object_id")]
        #[test_case(TEST_CASE_5 => "00000000-0000-0000-0000-000000000000,object_id=00000000-0000-0000-0000-000000000000 y01=13.4,y02=12.2 0" ; "handles many fields")]
        #[test_case(TEST_CASE_6 => "00000000-0000-0000-0000-000000000000,object_id=00000000-0000-0000-0000-000000000000 y01=true 0"          ; "handles boolean fields")]
        #[test_case(TEST_CASE_7 => "00000000-0000-0000-0000-000000000000,object_id=00000000-0000-0000-0000-000000000000 y01=\"some-msg\" 0"  ; "handles string fields")]
        fn produces_desired_correct_output(case: TestCase) -> String {
            build_line_protocol(
                case.schema_id.parse().unwrap(),
                case.object_id.parse().unwrap(),
                case.version,
                case.payload.as_bytes(),
            )
            .unwrap()
        }
    }
}
