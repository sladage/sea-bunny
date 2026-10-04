//! API services. [`ncclient::NCClient`] is the authenticated transport; every
//! other service covers one API area and lives on the client's shard, so it can
//! use the client's shard-local request API directly.

use eventful_rs::{Eventful, HasEvents, ShardRc, ShardRcHandle};

use ncclient::{NCClient, NCClientShard, NcError};

/// Declare a stateless API service: an eventful struct on [`NCClientShard`]
/// holding a local reference to the client, with an async `new` that binds it.
///
/// ```ignore
/// service! {
///     /// Reactions to chat messages.
///     pub struct ReactionService;
/// }
/// ```
macro_rules! service {
    ($(#[$meta:meta])* pub struct $name:ident;) => {
        $(#[$meta])*
        #[eventful_rs::eventful(shard = $crate::services::ncclient::NCClientShard)]
        pub struct $name {
            client: eventful_rs::ShardRc<$crate::services::ncclient::NCClient>,
        }

        impl $name {
            pub async fn new(
                client: &eventful_rs::ShardRcHandle<$crate::services::ncclient::NCClient>,
            ) -> Result<
                eventful_rs::ShardRcHandle<Self>,
                $crate::services::ncclient::NcError,
            > {
                $crate::services::bind_service(client, |client| Self {
                    client,
                    events: Default::default(),
                })
                .await
            }
        }
    };
}

pub mod ncclient;
pub mod notifications;
pub mod talk;

#[cfg(test)]
pub(crate) mod test_support;

/// Bind a service on the client's shard, handing it a local reference to the client.
pub(crate) async fn bind_service<S, F>(
    client: &ShardRcHandle<NCClient>,
    make: F,
) -> Result<ShardRcHandle<S>, NcError>
where
    S: Eventful<Shard = NCClientShard> + HasEvents<S::EventSetType> + 'static,
    F: FnOnce(ShardRc<NCClient>) -> S + Send + 'static,
{
    let client = client.clone();
    Ok(S::spawn(move || {
        // Cannot fail: the factory runs on NCClientShard (enforced by the bound on
        // `S`), and the captured strong handle keeps the client stored there.
        let client = client
            .try_local()
            .expect("services are bound on the client's shard");
        make(client)
    })
    .await?)
}
