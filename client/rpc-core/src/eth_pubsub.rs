use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use jsonrpc_pubsub::{typed, SubscriptionId};

use crate::types::pubsub;

pub use rpc_impl_EthPubSubApi::gen_server::EthPubSubApi as EthPubSubApiServer;

#[rpc(server)]
pub trait EthPubSubApi {
    type Metadata;

    #[pubsub(subscription = "eth_subscription", subscribe, name = "eth_subscribe")]
    fn subscribe(
        &self,
        _: Self::Metadata,
        _: typed::Subscriber<pubsub::Result>,
        _: pubsub::Kind,
        _: Option<pubsub::Params>,
    );

    #[pubsub(
    subscription = "eth_subscription",
    unsubscribe,
    name = "eth_unsubscribe"
    )]
    fn unsubscribe(&self, _: Option<Self::Metadata>, _: SubscriptionId) -> Result<bool>;
}