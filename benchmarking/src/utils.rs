use anyhow::Context;
use pbr::ProgressBar;
use serde::Serialize;
use serde_json::Value;
use std::io::Stdout;
use tokio::sync::Mutex;
use uuid::Uuid;

pub fn create_progress_bar(count: u64) -> Mutex<ProgressBar<Stdout>> {
    let mut pb = ProgressBar::new(count);
    pb.format("╢▌▌░╟");

    Mutex::new(pb)
}

pub fn load_samples() -> anyhow::Result<Vec<Value>> {
    let sample_data = vec![
        include_str!("../sample_json/1.json"),
        include_str!("../sample_json/2.json"),
        include_str!("../sample_json/3.json"),
        include_str!("../sample_json/4.json"),
        include_str!("../sample_json/5.json"),
        include_str!("../sample_json/6.json"),
        include_str!("../sample_json/7.json"),
        include_str!("../sample_json/8.json"),
    ];

    sample_data
        .into_iter()
        .map(|data| serde_json::from_str(&data).context("invalid sample"))
        .collect()
}

pub fn generate_messages(
    samples: Vec<Value>,
    schema_id: Uuid,
) -> impl Iterator<Item = (Uuid, Vec<u8>)> {
    let mut index = 0;

    std::iter::from_fn(move || {
        let sample = samples.get(index).expect("no samples");
        index = (index + 1) % samples.len();

        let message = Message::new(schema_id, sample);
        let object_id = message.object_id;

        Some((
            object_id,
            serde_json::to_vec(&message).expect("serialization error"),
        ))
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Message<'v> {
    object_id: Uuid,
    schema_id: Uuid,
    data: &'v Value,
}

impl<'v> Message<'v> {
    pub fn new(schema_id: Uuid, data: &'v Value) -> Message<'v> {
        Message {
            object_id: Uuid::new_v4(),
            schema_id,
            data,
        }
    }
}
