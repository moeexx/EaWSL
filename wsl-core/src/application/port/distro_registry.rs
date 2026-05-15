use crate::domain::model::distro::RegisteredDistroMetadata;
use crate::WslError;

pub(crate) trait DistroRegistryPort: Send + Sync {
    fn read_all_distros(&self) -> Result<Vec<RegisteredDistroMetadata>, WslError>;
}
