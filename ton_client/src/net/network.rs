use crate::client::ClientEnv;
use crate::error::{AddNetworkUrl, ClientResult};
use crate::net::queries::deserialize_result;
use crate::net::subscriptions::SubscriptionAction;
use crate::net::{
    ChainIterator, ParamsOfQueryCollection, ResultOfQueryCollection, ResultOfSubscription,
    ServerLink,
};
use crate::{client, net};
use failure::bail;
use futures::FutureExt;
use futures::StreamExt;
use rand::RngCore;
use std::collections::HashMap;
use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use ton_types::UInt256;
use crate::utils::json::JsonHelper;

#[derive(Debug)]
pub(crate) struct NetworkUID {
    pub(crate) zerostate_root_hash: UInt256,
    pub(crate) first_master_block_root_hash: UInt256,
}

pub struct NetworkContext {
    pub(crate) env: Arc<ClientEnv>,
    pub(crate) server_link: Option<ServerLink>,
    pub(crate) subscriptions: Mutex<HashMap<u32, mpsc::Sender<SubscriptionAction>>>,
    pub(crate) iterators: Mutex<HashMap<u32, Arc<Mutex<Box<dyn ChainIterator + Send + Sync>>>>>,
    pub(crate) network_uid: RwLock<Option<Arc<NetworkUID>>>,
}

impl NetworkContext {
    pub(crate) fn get_server_link(&self) -> ClientResult<&ServerLink> {
        self.server_link
            .as_ref()
            .ok_or_else(|| client::Error::net_module_not_init())
    }

    pub async fn query_collection(
        &self,
        params: ParamsOfQueryCollection,
    ) -> ClientResult<ResultOfQueryCollection> {
        let server_link = self.get_server_link()?;
        let result = server_link.query_collection(params, None).await;
        Ok(ResultOfQueryCollection {
            result: deserialize_result(result, server_link).await?,
        })
    }
    pub(crate) async fn add_subscription_handle(
        &self,
        handle: u32,
        sender: mpsc::Sender<SubscriptionAction>,
    ) {
        self.subscriptions.lock().await.insert(handle, sender);
    }

    pub(crate) async fn extract_subscription_handle(
        &self,
        handle: &u32,
    ) -> Option<mpsc::Sender<SubscriptionAction>> {
        self.subscriptions.lock().await.remove(handle)
    }

    pub(crate) async fn unsubscribe(&self, handle: u32) -> ClientResult<()> {
        if let Some(sender) = self.extract_subscription_handle(&handle).await {
            let _ = sender.send(SubscriptionAction::Finish).await;
        }
        Ok(())
    }

    pub async fn subscribe_collection<F: Future<Output = ()> + Send>(
        &self,
        collection: String,
        filter: Option<serde_json::Value>,
        result: String,
        callback: impl Fn(ClientResult<ResultOfSubscription>) -> F + Send + Sync + 'static,
    ) -> ClientResult<u32> {
        let server_link = self.get_server_link()?;
        let subscription = server_link
            .subscribe_collection(&collection, filter.as_ref().unwrap_or(&json!({})), &result)
            .await
            .map_err(|err| net::Error::queries_subscribe_failed(err))
            .add_network_url(server_link)
            .await?;
        self.run_subscription(subscription, callback).await
    }

    async fn run_subscription<F: Future<Output = ()> + Send>(
        &self,
        subscription: super::server_link::Subscription,
        callback: impl Fn(ClientResult<ResultOfSubscription>) -> F + Send + Sync + 'static,
    ) -> ClientResult<u32> {
        let (sender, mut receiver) = mpsc::channel(10);
        let handle = rand::thread_rng().next_u32();
        self.add_subscription_handle(handle, sender).await;

        // spawn thread which reads subscription stream and calls callback with data
        self.env.spawn(Box::pin(async move {
            let mut data_stream = subscription.data_stream.fuse();
            let wait_action = receiver.recv().fuse();
            futures::pin_mut!(wait_action);
            loop {
                futures::select!(
                    // waiting next subscription data
                    data = data_stream.select_next_some() => {
                        callback(data.map(|data| ResultOfSubscription { result: data })).await
                    },
                    // waiting for some action with subscription (the only action is Finish)
                    _action = wait_action => {
                        break;
                    }
                );
            }
            subscription.unsubscribe.await;
        }));

        Ok(handle)
    }

    pub async fn subscribe<F: Future<Output = ()> + Send>(
        &self,
        subscription: String,
        variables: Option<serde_json::Value>,
        callback: impl Fn(ClientResult<ResultOfSubscription>) -> F + Send + Sync + 'static,
    ) -> ClientResult<u32> {
        let server_link = self.get_server_link()?;
        let subscription = server_link
            .subscribe(subscription, variables)
            .await
            .map_err(|err| net::Error::queries_subscribe_failed(err))
            .add_network_url(server_link)
            .await?;
        self.run_subscription(subscription, callback).await
    }

    pub(crate) async fn get_current_network_uid(&self) -> ton_types::Result<Arc<NetworkUID>> {
        if let Some(ref uid) = *self.network_uid.read().await {
            return Ok(Arc::clone(uid));
        }

        let queried_uid = self.query_current_network_uid().await?;

        let mut write_guard = self.network_uid.write().await;
        if let Some(ref stored_uid) = *write_guard {
            return Ok(Arc::clone(stored_uid));
        }

        *write_guard = Some(Arc::clone(&queried_uid));

        Ok(queried_uid)
    }

    pub(crate) async fn query_current_network_uid(&self) -> ton_types::Result<Arc<NetworkUID>> {
        let blocks = self
            .query_collection(ParamsOfQueryCollection {
                collection: "blocks".to_string(),
                filter: Some(json!({
                    "workchain_id": {
                        "eq": - 1
                    },
                    "seq_no": {
                    "eq": 1
                    },
                })),
                result: "id, prev_ref{root_hash}".to_string(),
                limit: Some(1),
                ..Default::default()
            })
            .await?
            .result;

        if blocks.is_empty() {
            bail!("Unable to resolve zerostate's root hash: can't get masterchain block #1");
        }

        let prev_ref = &blocks[0]["prev_ref"];
        if prev_ref.is_null() {
            bail!("Unable to resolve zerostate's root hash: prev_ref of the block #1 is not set");
        }

        let first_master_block_root_hash = UInt256::from_str(blocks[0].get_str("id")?)?;
        let zerostate_root_hash = UInt256::from_str(prev_ref.get_str("root_hash")?)?;

        Ok(Arc::new(NetworkUID {
            zerostate_root_hash,
            first_master_block_root_hash,
        }))
    }
}
