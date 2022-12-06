use std::{pin::Pin, sync::Arc};

use ggez::{GameError, GameResult};
use webrtc::{
    api::{interceptor_registry, media_engine::MediaEngine, APIBuilder},
    data_channel::{data_channel_message::DataChannelMessage, RTCDataChannel},
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration, sdp::session_description::RTCSessionDescription,
        RTCPeerConnection,
    },
};

use super::MultiplayerGameMessage;

pub struct MultiplayerTransport {
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

    fn channel_msg_handler(
        msg: DataChannelMessage,
        tx: Arc<flume::Sender<MultiplayerGameMessage>>,
    ) {
        if let Ok(msg_decode) = bincode::deserialize::<MultiplayerGameMessage>(&msg.data) {
            println!("Message from game data channel {:#?}", msg_decode);
            if let Err(e) = tx.send(msg_decode) {
                println!("Failed to send event to event buffer {:?}", e);
            }
        }
    }

    async fn channel_push_handler(
        push_rx: Arc<flume::Receiver<MultiplayerGameMessage>>,
        channel: Arc<RTCDataChannel>,
    ) {
        while let Ok(msg) = push_rx.recv() {
            if let Ok(ser_msg) = bincode::serialize(&msg) {
                if let Err(e) = channel.send(&bytes::Bytes::from(ser_msg)).await {
                    println!("Failed to send event to peer {:?}", e);
                }
            }
        }
    }

    async fn create_game_async(
        conn_string: Option<String>,
        tx: flume::Sender<MultiplayerGameMessage>,
        rx: flume::Receiver<MultiplayerGameMessage>,
    ) -> GameResult {
        let peer_conn = Self::setup().await?;

        let (tx, rx) = (Arc::new(tx), Arc::new(rx));
        let tx_c = tx.clone();
        let rx_c = rx.clone();

        if conn_string.is_none() {
            let channel = peer_conn
                .create_data_channel("MultiplayerGameData", None)
                .await
                .map_err(|_| {
                    GameError::CustomError("Failed to create a data channel".to_string())
                })?;

            // Register handlers
            channel.on_message(Box::new(move |msg: DataChannelMessage| {
                Self::channel_msg_handler(msg, tx_c.clone());
                Box::pin(async move {})
            }));
            tokio::spawn(async move { Self::channel_push_handler(rx_c, channel).await });
        } else {
            let json = base64::decode(conn_string.unwrap()).map_err(|_| {
                GameError::ConfigError("Failed to decode base64 conn string".to_string())
            })?;
            let sd: RTCSessionDescription = serde_json::from_slice(&json).map_err(|_| {
                GameError::ConfigError(
                    "Failed to convert connection string to RTCSessionDescription".to_string(),
                )
            })?;
            peer_conn.set_remote_description(sd).await.map_err(|_| {
                GameError::CustomError("Failed to set description for peer".to_string())
            })?;

            peer_conn.on_data_channel(Box::new(move |channel: Arc<RTCDataChannel>| {
                if channel.label() == "MultiplayerGameData" {
                    let tx_cc = tx_c.clone();
                    // Register handlers
                    channel.on_message(Box::new(move |msg: DataChannelMessage| {
                        Self::channel_msg_handler(msg, tx_cc.clone());
                        Box::pin(async move {})
                    }));
                    let rx_cc = rx_c.clone();
                    tokio::spawn(async move {
                        Self::channel_push_handler(rx_cc.clone(), channel).await
                    });
                }
                Box::pin(async move {})
            }));
        }

        let offer = peer_conn
            .create_offer(None)
            .await
            .map_err(|_| GameError::CustomError("Failed to create offer".to_string()))?;
        let mut g_c = peer_conn.gathering_complete_promise().await;
        peer_conn
            .set_local_description(offer)
            .await
            .map_err(|_| GameError::CustomError("Failed to set local description".to_string()))?;

        g_c.recv().await;

        let base64_conn_str = if let Some(l_d) = peer_conn.local_description().await {
            base64::encode(serde_json::to_string(&l_d).map_err(|e| {
                GameError::CustomError("Failed to convert peer base64 to json".to_string())
            })?)
        } else {
            return Err(GameError::CustomError(
                "Failed to get peer base64".to_string(),
            ));
        };

        Ok(())
    }

    pub fn create_game(conn_string: Option<String>) -> GameResult<Self> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let (tx, rx) = flume::unbounded::<MultiplayerGameMessage>();
        let (push_tx, push_rx) = flume::unbounded::<MultiplayerGameMessage>();
        rt.spawn(Self::create_game_async(conn_string, push_tx, rx));

        Ok(Self {
            event_buffer: push_rx,
            event_push_buffer: tx,
        })
    }
}
