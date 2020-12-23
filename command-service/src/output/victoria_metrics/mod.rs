use crate::communication::resolution::Resolution;
use crate::output::victoria_metrics::config::VictoriaMetricsConfig;
use crate::output::OutputPlugin;
use fnv::FnvHashMap;
use log::error;
use reqwest::Url;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use serde_json::Value;
use thiserror::Error as DeriveError;
use url::ParseError;
use utils::message_types::BorrowedInsertMessage;
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
    #[error("Data cannot be parsed `{0}`")]
    DataCannotBeParsed(serde_json::Error),
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
    async fn handle_message(&self, msg: BorrowedInsertMessage<'_>) -> Resolution {
        let mut url = self.url.clone();

        url.set_query(Some(&format!("db={}", msg.schema_id)));

        let BorrowedInsertMessage {
            object_id,
            schema_id,
            data,
            ..
        } = msg;

        match build_line_protocol(schema_id, object_id, data) {
            Ok(line_protocol) => send_data(url, &self.client, line_protocol).await,
            Err(err) => {
                let context = data.to_string();

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

#[derive(Serialize, Deserialize, Debug)]
struct Payload {
    fields: FnvHashMap<String, Value>,
    ts: u64,
}

fn build_line_protocol(measurement: Uuid, tag: Uuid, payload: &RawValue) -> Result<String, Error> {
    let payloads: Vec<Payload> =
        serde_json::from_str(payload.get()).map_err(Error::DataCannotBeParsed)?;
    let line_protocol = payloads
        .into_iter()
        .map(|obj| {
            Ok(format!(
                "{},objectId={} {} {}",
                measurement,
                tag,
                get_object_fields(&obj)?,
                obj.ts
            ))
        })
        .collect::<Result<Vec<String>, Error>>()?
        .join("\n");
    Ok(line_protocol)
}

fn get_object_fields(request_object: &Payload) -> Result<String, Error> {
    if request_object.fields.is_empty() {
        return Err(Error::EmptyFields);
    }
    Ok(request_object
        .fields
        .iter()
        .map(|(key, value)| {
            if value.is_array() || value.is_null() || value.is_object() {
                return Err(Error::InvalidFieldType);
            }
            Ok(format!("{}={}", key, value))
        })
        .collect::<Result<Vec<String>, Error>>()?
        .join(","))
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

        #[test_case(r#"[{"fields":{}, "ts": 15},
                       {"fields":{"a01": 10}, "ts": 15}]"#                => matches Err(Error::EmptyFields))]
        #[test_case(r#"[{"fields":{"y01": {}}, "ts": 10}]"#               => matches Err(Error::InvalidFieldType))]
        #[test_case(r#"[{"fields":{"y01": []}, "ts": 5}]"#                => matches Err(Error::InvalidFieldType))]
        #[test_case(r#"[{"fields":{"y01": null}, "ts": 1}]"#              => matches Err(Error::InvalidFieldType))]
        #[test_case(r#"[{"fields":{"y01": 123}, "ts": {}}]"#              => matches Err(Error::DataCannotBeParsed(_)))]
        #[test_case(r#"[{"fields":{"y01": 1234}, "ts": []}]"#             => matches Err(Error::DataCannotBeParsed(_)))]
        #[test_case(r#"[{"fields":{"y01": 12345}, "ts": null}]"#          => matches Err(Error::DataCannotBeParsed(_)))]
        #[test_case(r#"[{"fields":{"y01": 12345}, "ts": ""}]"#            => matches Err(Error::DataCannotBeParsed(_)))]
        #[test_case(r#"{"fields" : {"y01": 13.4}, "ts": 123 }"#           => matches Err(Error::DataCannotBeParsed(_)))]
        #[test_case(r#"[ 1, 13, { "fields": {"y01": 1}, "ts": 1234 }]"#   => matches Err(Error::DataCannotBeParsed(_)))]
        fn produces_desired_errors(payload: &str) -> Result<String, Error> {
            build_line_protocol(
                Uuid::default(),
                Uuid::default(),
                &RawValue::from_string(payload.to_string()).unwrap(),
            )
        }

        struct TestCase {
            object_id: &'static str,
            schema_id: &'static str,
            payload: &'static str,
        }

        const TEST_CASE_1: TestCase = TestCase {
            object_id: "00000000-0000-0000-0000-000000000000",
            schema_id: "00000000-0000-0000-0000-000000000000",
            payload: r#"[{"fields":{"y01": 13.4}, "ts": 123}]"#,
        };

        const TEST_CASE_2: TestCase = TestCase {
            object_id: "00000000-0000-0000-0000-000000000000",
            schema_id: "00000000-0000-0000-0000-000000000000",
            payload: r#"[{"fields":{"y01": 13.4}, "ts": 1603887165}]"#,
        };

        const TEST_CASE_3: TestCase = TestCase {
            object_id: "00000000-0000-0000-0000-000000000000",
            schema_id: "10b7a9cd-0daf-4cb6-a7ef-b9db6058a2d3",
            payload: r#"[{"fields": {"y01": 13.4}, "ts": 123}]"#,
        };

        const TEST_CASE_4: TestCase = TestCase {
            object_id: "85cf3b2e-0c9c-40e1-ade3-4596a2e1d9b6",
            schema_id: "00000000-0000-0000-0000-000000000000",
            payload: r#"[{"fields": {"y01": 13.4}, "ts": 123}]"#,
        };

        const TEST_CASE_5: TestCase = TestCase {
            object_id: "00000000-0000-0000-0000-000000000000",
            schema_id: "00000000-0000-0000-0000-000000000000",
            payload: r#"[{"fields": {"y01": 13.4, "y02": 12.2}, "ts": 123}]"#,
        };

        const TEST_CASE_6: TestCase = TestCase {
            object_id: "00000000-0000-0000-0000-000000000000",
            schema_id: "00000000-0000-0000-0000-000000000000",
            payload: r#"[{"fields": {"y01": true}, "ts": 123}]"#,
        };

        const TEST_CASE_7: TestCase = TestCase {
            object_id: "00000000-0000-0000-0000-000000000000",
            schema_id: "00000000-0000-0000-0000-000000000000",
            payload: r#"[{"fields": {"y01": "some-msg"}, "ts": 123}]"#,
        };

        const TEST_CASE_8: TestCase = TestCase {
            object_id: "00000000-0000-0000-0000-000000000000",
            schema_id: "00000000-0000-0000-0000-000000000000",
            payload: r#"[
                            { "fields": {"y01": 123, "y02": 54321}, "ts": 1234 },
                            { "fields": {"y01": 321, "y02": 12345}, "ts": 4321 },
                            { "fields": {"z01": true, "a02": "string"}, "ts": 0 }
                        ]"#,
        };

        #[test_case(TEST_CASE_1 => "00000000-0000-0000-0000-000000000000,objectId=00000000-0000-0000-0000-000000000000 y01=13.4 123")]
        #[test_case(TEST_CASE_2 => "00000000-0000-0000-0000-000000000000,objectId=00000000-0000-0000-0000-000000000000 y01=13.4 1603887165" ; "changes timestamp")]
        #[test_case(TEST_CASE_3 => "10b7a9cd-0daf-4cb6-a7ef-b9db6058a2d3,objectId=00000000-0000-0000-0000-000000000000 y01=13.4 123"          ; "changes schema_id")]
        #[test_case(TEST_CASE_4 => "00000000-0000-0000-0000-000000000000,objectId=85cf3b2e-0c9c-40e1-ade3-4596a2e1d9b6 y01=13.4 123"          ; "changes object_id")]
        #[test_case(TEST_CASE_5 => "00000000-0000-0000-0000-000000000000,objectId=00000000-0000-0000-0000-000000000000 y01=13.4,y02=12.2 123" ; "handles many fields")]
        #[test_case(TEST_CASE_6 => "00000000-0000-0000-0000-000000000000,objectId=00000000-0000-0000-0000-000000000000 y01=true 123"          ; "handles boolean fields")]
        #[test_case(TEST_CASE_7 => "00000000-0000-0000-0000-000000000000,objectId=00000000-0000-0000-0000-000000000000 y01=\"some-msg\" 123"  ; "handles string fields")]
        #[test_case(TEST_CASE_8 =>
"00000000-0000-0000-0000-000000000000,objectId=00000000-0000-0000-0000-000000000000 y01=123,y02=54321 1234
00000000-0000-0000-0000-000000000000,objectId=00000000-0000-0000-0000-000000000000 y01=321,y02=12345 4321
00000000-0000-0000-0000-000000000000,objectId=00000000-0000-0000-0000-000000000000 a02=\"string\",z01=true 0";
                                 "handles multiple objects")]

        fn produces_desired_correct_output(case: TestCase) -> String {
            build_line_protocol(
                case.schema_id.parse().unwrap(),
                case.object_id.parse().unwrap(),
                &RawValue::from_string(case.payload.to_string()).unwrap(),
            )
            .unwrap()
        }
    }
}
