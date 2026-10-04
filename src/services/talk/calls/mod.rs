//! Call services. These cover the REST side of calls; media (WebRTC) is not
//! part of this layer.

pub mod call;
pub mod recording;
pub mod signaling;
pub mod transcription;

use eventful_rs::ShardRcHandle;

pub use call::CallService;
pub use recording::RecordingService;
pub use signaling::SignalingService;
pub use transcription::LiveTranscriptionService;

use crate::services::ncclient::{NCClient, NcError};

#[derive(Clone)]
pub struct CallServices {
    pub calls: ShardRcHandle<CallService>,
    pub recording: ShardRcHandle<RecordingService>,
    pub transcription: ShardRcHandle<LiveTranscriptionService>,
    pub signaling: ShardRcHandle<SignalingService>,
}

impl CallServices {
    pub async fn new(client: &ShardRcHandle<NCClient>) -> Result<Self, NcError> {
        Ok(Self {
            calls: CallService::new(client).await?,
            recording: RecordingService::new(client).await?,
            transcription: LiveTranscriptionService::new(client).await?,
            signaling: SignalingService::new(client).await?,
        })
    }
}
