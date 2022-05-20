use futures_util::{future, StreamExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use std::sync::Mutex;
use once_cell::sync::OnceCell;
use ctor::dtor;

static RUNTIME: OnceCell<Mutex<Option<Runtime>>> = OnceCell::new();

async fn websocket_server_main() {
    let addr = "localhost:8000";

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }
}

async fn accept_connection(stream: TcpStream) {
    let _addr = stream.peer_addr().expect("connected streams should have a peer address");

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    let (write, read) = ws_stream.split();
    // We should not forward messages other than text or binary.
    read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        .forward(write)
        .await
        .expect("Failed to forward messages")
}

fn end_server() {
    let runtime: Runtime = RUNTIME.get().unwrap().lock().unwrap().take().unwrap();
    runtime.shutdown_background();
}

byond_fn!(fn start_server() {
    let runtime = Runtime::new().unwrap();
    runtime.spawn(websocket_server_main());

    RUNTIME.set(Mutex::new(Some(runtime))).expect("there was already a runtime");

    Some("")
});

byond_fn!(fn kill_server() {
    end_server();
    Some("")
});

#[dtor]
fn shutdown() {
    end_server();
}
