use std::{pin::Pin, sync::Arc};

use ggez::{GameError, GameResult};
use webrtc::{
    api::{interceptor_registry, media_engine::MediaEngine, APIBuilder},
    data_channel::{data_channel_message::DataChannelMessage, RTCDataChannel},
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{configuration::RTCConfiguration, RTCPeerConnection},
};

use super::MultiplayerGameMessage;

pub struct MultiplayerTransport {
    peer_conn: Arc<RTCPeerConnection>,
    pub event_buffer: flume::Receiver<MultiplayerGameMessage>,
    pub event_push_buffer: flume::Sender<MultiplayerGameMessage>,
}

// Largely borrowed from webrtc-rs examples
impl MultiplayerTransport {
    async fn setup() -> GameResult<Arc<RTCPeerConnection>> {
        let mut engine = MediaEngine::default();
        engine.register_default_codecs().map_err(|_| {
            GameError::CustomError("Failed to register media engine codecs".to_string())
        })?;
        let mut registry = Registry::new();
        registry = interceptor_registry::register_default_interceptors(registry, &mut engine)
            .map_err(|_| {
                GameError::CustomError("Failed to register default interceptors".to_string())
            })?;

        let api = APIBuilder::new()
            .with_media_engine(engine)
            .with_interceptor_registry(registry)
            .build();

        let rtc_conf = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let peer_conn =
            Arc::new(api.new_peer_connection(rtc_conf).await.map_err(|_| {
                GameError::CustomError("Failed to make peer connection".to_string())
            })?);

        Ok(peer_conn)
    }

    // TODO Create the async runtime automatically
    // TODO Automatically send the handshake message with user details and populate Peer struct
    // with it.
    //
    // Clients can use the flume queue to communicate with the peer

    async fn channel_msg_handler(
        msg: DataChannelMessage,
        tx: flume::Sender<MultiplayerGameMessage>,
    ) -> Pin {
        if let Ok(msg_decode) = bincode::deserialize::<MultiplayerGameMessage>(&msg.data) {
            println!("Message from game data channel {:#?}", msg_decode);
            if let Err(e) = tx.send(msg_decode) {
                println!("Failed to send event to event buffer {:?}", e);
            }
        }
        Box::pin(async move {})
    }

    async fn create_game_async(creator: bool) -> GameResult<Self> {
        let peer_conn = Self::setup().await?;

        let (tx, rx) = flume::unbounded::<MultiplayerGameMessage>();
        let (push_tx, push_rx) = flume::unbounded::<MultiplayerGameMessage>();

        if creator {
            let channel = peer_conn
                .create_data_channel("MultiplayerGameData", None)
                .await
                .map_err(|_| {
                    GameError::CustomError("Failed to create a data channel".to_string())
                })?;
            channel.on_message(Box::new(|msg: DataChannelMessage| {
                Self::channel_msg_handler(msg, tx)
            }));
        } else {
            peer_conn.on_data_channel(Box::new(
                move |d: Arc<RTCDataChannel>| {
                    if d.label() == "MultiplayerGameData" {}
                },
            ));
        }

        tokio::spawn(async move {
            while let Ok(msg) = push_rx.recv_async().await {
                if let Ok(ser_msg) = bincode::serialize(&msg) {
                    if let Err(e) = channel.send(&bytes::Bytes::from(ser_msg)).await {
                        println!("Failed to send event to peer {:?}", e);
                    }
                }
            }
        });

        Ok(Self {
            peer_conn,
            event_buffer: rx,
            event_push_buffer: push_tx,
        })
    }

    pub fn create_game(conn_string: String) -> GameResult<Self> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(Self::create_game_async())
    }
    pub fn connect_game(c_string: String) -> GameResult<Self> {
        todo!()
    }
}
