use eventful_rs::ShardRcHandle;

use crate::services::ncclient::NCClient;

pub struct AppContext {
    pub client: ShardRcHandle<NCClient>,
}
