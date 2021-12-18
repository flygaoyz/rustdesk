use hbb_common::{
    base_proto::PeerInfo,
    discovery_proto::Discovery as DiscoveryProto,
    env_logger::*,
    log, protobuf,
    tokio::{self, sync::Notify},
};
use socket_cs::{discovery::*, udp::*};
use std::env;
use std::sync::Arc;

async fn lan_discover(port: u16, port_back: u16) {
    let peer = PeerInfo {
        username: "client username".to_owned(),
        hostname: "client hostname".to_owned(),
        ..Default::default()
    };
    let client = DiscoveryClient::create(DiscoveryProto {
        peer: protobuf::MessageField::from_option(Some(peer)),
        port: port_back as i32,
        ..Default::default()
    })
    .await
    .unwrap();

    client.lan_discover(port).await.unwrap();
}

async fn listen_discovery_back(port: u16) {
    fn proc_peer(peer: PeerInfo) {
        log::info!(
            "peer recived, username: {}, hostname: {}",
            peer.username,
            peer.hostname
        );
    }

    let exit_notify = Notify::new();
    let handlers = UdpHandlers::new().handle(
        CMD_DISCOVERY_BACK.as_bytes().to_vec(),
        Box::new(HandlerDiscoveryBack::new(proc_peer)),
    );

    let server = Server::new(port, Arc::new(exit_notify));
    server.start(handlers).await.unwrap();

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}

async fn listen_discovery(port: u16) {
    let peer = PeerInfo {
        username: "server username".to_owned(),
        hostname: "server hostname".to_owned(),
        ..Default::default()
    };

    let exit_notify = Notify::new();
    let handlers = UdpHandlers::new().handle(
        CMD_DISCOVERY.as_bytes().to_vec(),
        Box::new(HandlerDiscovery::new(peer)),
    );

    let server = Server::new(port, Arc::new(exit_notify));
    server.start(handlers).await.unwrap();
    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}

#[tokio::main]
async fn main() {
    init_from_env(Env::default().filter_or(DEFAULT_FILTER_ENV, "trace"));

    let args: Vec<String> = env::args().collect();

    let port_back = 9801u16;
    let server_port: u16 = 9802u16;

    if args.len() == 1 {
        lan_discover(server_port, port_back).await;
    } else if args.len() == 2 {
        listen_discovery_back(port_back).await;
    } else {
        listen_discovery(server_port).await;
    }
}
