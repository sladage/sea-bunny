use eventful_rs::ShardRcHandle;

use crate::services::{ncclient::NCClient, talk::TalkServices};

pub struct AppContext {
    pub client: ShardRcHandle<NCClient>,
    pub talk: TalkServices,
}
