use crate::application::port::distro_registry::DistroRegistryPort;
use crate::domain::model::distro::RegisteredDistroMetadata;
use crate::WslError;

pub(crate) struct SystemDistroRegistryAdapter;

impl DistroRegistryPort for SystemDistroRegistryAdapter {
    fn read_all_distros(&self) -> Result<Vec<RegisteredDistroMetadata>, WslError> {
        super::reader::read_all_distros()
    }
}
