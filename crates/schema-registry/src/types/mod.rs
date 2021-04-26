use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use self::schema::FullSchema;

pub mod schema;
pub mod view;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionedUuid {
    pub id: Uuid,
    pub version_req: VersionReq,
}

impl VersionedUuid {
    pub fn new(id: Uuid, version_req: VersionReq) -> Self {
        Self { id, version_req }
    }

    pub fn exact(id: Uuid, version: Version) -> Self {
        Self {
            id,
            version_req: VersionReq::exact(&version),
        }
    }

    pub fn any(id: Uuid) -> Self {
        Self {
            id,
            version_req: VersionReq::any(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbExport {
    pub schemas: Vec<FullSchema>,
}
