#[must_use]
#[derive(Clone, Debug, PartialEq)]
pub enum Resolution {
    StorageLayerFailure {
        description: String,
    },
    CommandServiceFailure,
    UserFailure {
        description: String,
        context: String,
    },
    Success,
}
