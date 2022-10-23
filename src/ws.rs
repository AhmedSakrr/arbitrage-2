use crate::{Client, Clients};
use futures::{FutureExt, StreamExt};
use log::{error, info};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::WebSocket;

pub async fn client_connection(ws: WebSocket, clients: Clients) {
    info!("establishing client connection... {:?}", ws);

    let (client_ws_sender, _client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);

    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            error!("error sending websocket msg: {}", e);
        }
    }));

    let uuid = Uuid::new_v4().simple().to_string();

    let new_client = Client {
        client_id: uuid.clone(),
        sender: Some(client_sender),
    };

    clients.lock().await.insert(uuid.clone(), new_client);

    clients.lock().await.remove(&uuid);
    info!("{} disconnected", uuid);
}
