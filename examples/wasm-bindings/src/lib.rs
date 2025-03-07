use std::{panic, sync::Arc};

pub use ankurah::Node;
pub use ankurah_storage_indexeddb_wasm::IndexedDBStorageEngine;
pub use ankurah_websocket_client_wasm::WebsocketClient;
use example_model::*;
use tracing::{error, info};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    tracing_wasm::set_as_global_default();
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let _ = any_spawner::Executor::init_wasm_bindgen();
    Ok(())
}

#[wasm_bindgen]
pub async fn create_client() -> Result<WebsocketClient, JsValue> {
    let storage_engine = IndexedDBStorageEngine::open("ankurah_example_app").await.map_err(|e| JsValue::from_str(&e.to_string()))?;
    let node = Node::new(Arc::new(storage_engine));
    let connector = WebsocketClient::new(node.clone(), "ws://127.0.0.1:9797")?;

    info!("Waiting for client to connect");

    Ok(connector)
}

use ankurah::{changes::ChangeSet, ResultSet, WasmSignal};
#[wasm_bindgen]
pub async fn fetch_test_items(client: &WebsocketClient) -> Result<Vec<SessionView>, JsValue> {
    let sessions: ResultSet<SessionView> =
        client.node().fetch("date_connected = '2024-01-01'").await.map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(sessions.into())
}

#[wasm_bindgen]
pub fn subscribe_test_items(client: &WebsocketClient) -> Result<TestResultSetSignal, JsValue> {
    let (signal, rwsignal) = reactive_graph::signal::RwSignal::new(TestResultSet::default()).split();

    let client = client.clone();
    wasm_bindgen_futures::spawn_local(async move {
        client.ready().await;

        use reactive_graph::traits::Set;
        match client
            .node()
            .subscribe("date_connected = '2024-01-01'", move |changeset: ChangeSet<SessionView>| {
                rwsignal.set(TestResultSet(Arc::new(changeset.resultset.clone())));
                // let mut received = received_changesets_clone.lock().unwrap();
                // received.push(changeset);
            })
            .await
        {
            Ok(handle) => {
                // HACK
                std::mem::forget(handle);
            }
            Err(e) => {
                error!("Failed to subscribe to changes: {}", e);
            }
        }
    });

    Ok(signal.into())
}

#[wasm_bindgen]
pub async fn create_test_entity(client: &WebsocketClient) -> Result<(), JsValue> {
    let trx = client.node().begin();
    let _session = trx
        .create(&Session {
            date_connected: "2024-01-01".to_string(),
            ip_address: "127.0.0.1".to_string(),
            node_id: client.node().id.clone().into(),
        })
        .await;
    trx.commit().await.unwrap();
    Ok(())
}

#[wasm_bindgen]
#[derive(WasmSignal, Clone, Default)]
pub struct TestResultSet(Arc<ResultSet<SessionView>>);

#[wasm_bindgen]
impl TestResultSet {
    pub fn resultset(&self) -> Vec<SessionView> { self.0.items.to_vec() }
}
