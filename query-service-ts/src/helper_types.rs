use crate::schema::DataPoint;
use serde::Serialize;

#[derive(Serialize)]
#[serde(remote = "DataPoint")]
pub struct DataPointDef {
    timestamp: String,
    value: f32,
}

#[derive(Serialize)]
pub struct DataPointSerializable(#[serde(with = "DataPointDef")] pub DataPoint);

pub fn make_serializable_timeseries(timeseries: Vec<DataPoint>) -> Vec<DataPointSerializable> {
    timeseries
        .into_iter()
        .map(DataPointSerializable)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::helper_types::make_serializable_timeseries;
    use crate::schema::DataPoint;
    #[test]
    fn is_data_point_serializable() {
        let given_input = vec![
            (DataPoint {
                timestamp: "2017-01-15T01:30:15.01Z".to_string(),
                value: 10.0,
            }),
            (DataPoint {
                timestamp: "2017-01-15T01:30:17.01Z".to_string(),
                value: 11.1,
            }),
        ];

        let serializable_input = make_serializable_timeseries(given_input);
        let output = serde_json::to_string(&serializable_input);
        // Fixme: Use more compact format of serializing
        let expected_output ="[{\"timestamp\":\"2017-01-15T01:30:15.01Z\",\"value\":10.0},{\"timestamp\":\"2017-01-15T01:30:17.01Z\",\"value\":11.1}]".to_string();
        match output {
            Ok(a) => assert_eq!(a, expected_output),
            Err(_) => assert!(false, "Serialization failed"),
        }
    }
}
