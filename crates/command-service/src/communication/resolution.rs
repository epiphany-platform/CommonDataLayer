use std::fmt;

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

impl fmt::Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Resolution::StorageLayerFailure { description } => {
                write!(f, "failed on database layer, `{}`", description)
            }
            Resolution::CommandServiceFailure => write!(f, "internal server error"),
            Resolution::UserFailure {
                description,
                context,
            } => write!(f, "{}; caused by {}", description, context),
            Resolution::Success => write!(f, "success"),
        }
    }
}
