use std::{pin::Pin, sync::Arc};

use ggez::{GameError, GameResult};
use log::trace;
use serde::{de::DeserializeOwned, Deserialize};
use webrtc::{
    api::{interceptor_registry, media_engine::MediaEngine, APIBuilder},
    data_channel::{
        data_channel_message::DataChannelMessage, data_channel_state::RTCDataChannelState,
        RTCDataChannel,
    },
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription, RTCPeerConnection,
    },
    Error,
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
            ice_servers: vec![
                RTCIceServer {
                    urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                    ..Default::default()
                },
                RTCIceServer {
                    urls: vec!["turn:openrelay.metered.ca:80".to_owned()],
                    username: "openrelayproject".to_string(),
                    credential: "openrelayproject".to_string(),
                    ..Default::default()
                },
                RTCIceServer {
                    urls: vec!["turn:openrelay.metered.ca:443".to_owned()],
                    username: "openrelayproject".to_string(),
                    credential: "openrelayproject".to_string(),
                    ..Default::default()
                },
            ],
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
        tx: Arc<flume::Sender<MultiplayerGameMessage>>,
    ) {
        if let Ok(msg_decode) = bincode::deserialize::<MultiplayerGameMessage>(&msg.data) {
            trace!("Channel msg handled {:?}", msg_decode);
            if let Err(e) = tx.send_async(msg_decode).await {
                println!("Failed to send event to event buffer {:?}", e);
            }
        }
    }

    async fn channel_push_handler(
        push_rx: Arc<flume::Receiver<MultiplayerGameMessage>>,
        exit_tx: flume::Sender<bool>,
        channel: Arc<RTCDataChannel>,
    ) {
        while let Ok(msg) = push_rx.recv_async().await {
            if let Ok(ser_msg) = bincode::serialize(&msg) {
                match channel.send(&bytes::Bytes::from(ser_msg)).await {
                    Err(e) => println!("Failed to send event to peer {:?}", e),
                    Ok(k) => trace!("Sent data to peer with size {:?}", k),
                }
                if let MultiplayerGameMessage::CloseConnection = msg {
                    exit_tx.send(true).unwrap();
                    return;
                }
            }
        }
    }

    fn session_desc_from_str<T>(conn_string: String) -> GameResult<T>
    where
        T: DeserializeOwned,
    {
        let json = base64::decode(conn_string).map_err(|_| {
            GameError::ConfigError("Failed to decode base64 conn string".to_string())
        })?;
        serde_json::from_slice(&json).map_err(|_| {
            GameError::ConfigError(
                "Failed to convert connection string to RTCSessionDescription".to_string(),
            )
        })
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

        let (tx_exit, rx_exit) = flume::bounded::<bool>(1);
        let txe_c = tx_exit.clone();

        let channel = if conn_string.is_none() {
            let channel = peer_conn
                .create_data_channel("MultiplayerGameData", None)
                .await
                .map_err(|_| {
                    GameError::CustomError("Failed to create a data channel".to_string())
                })?;

            let channel_c = channel.clone();
            let rx_cc = rx_c.clone();

            channel.on_open(Box::new(move || {
                println!("Data channel open!");

                Box::pin(async move {
                    Self::channel_push_handler(rx_cc, tx_exit, channel_c.clone()).await;
                })
            }));

            // Register handlers
            channel.on_message(Box::new(move |msg: DataChannelMessage| {
                let tx_cc = tx_c.clone();
                Box::pin(async move { Self::channel_msg_handler(msg, tx_cc.clone()).await })
            }));

            channel.on_error(Box::new(move |msg: Error| {
                println!("channel err {:?}", msg);
                Box::pin(async move {})
            }));
            channel.on_close(Box::new(move || {
                println!("channel close");
                Box::pin(async move {})
            }));
            // Add a listener to set the remote description for the peer
            Some(channel)
        } else {
            peer_conn.on_data_channel(Box::new(move |channel: Arc<RTCDataChannel>| {
                println!("New channel found {}", channel.label());
                if channel.label() == "MultiplayerGameData" {
                    let tx_cc = tx_c.clone();
                    // Register handlers
                    channel.on_message(Box::new(move |msg: DataChannelMessage| {
                        let tx_ccc = tx_cc.clone();
                        Box::pin(async move {
                            Self::channel_msg_handler(msg, tx_ccc.clone()).await;
                        })
                    }));

                    channel.on_error(Box::new(move |msg: Error| {
                        println!("channel err {:?}", msg);
                        Box::pin(async move {})
                    }));
                    channel.on_close(Box::new(move || {
                        println!("channel close");
                        Box::pin(async move {})
                    }));

                    let rx_cc = rx_c.clone();
                    let txe_cc = txe_c.clone();
                    let channel_c = channel.clone();

                    channel.on_open(Box::new(move || {
                        println!("Channel opened");
                        Box::pin(async move {
                            Self::channel_push_handler(rx_cc.clone(), txe_cc, channel_c).await;
                        })
                    }));
                }
                Box::pin(async move {})
            }));
            None
        };
        peer_conn.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            println!("Peer Connection State has changed: {}", s);
            Box::pin(async {})
        }));

        let offer = if let Some(c_s) = &conn_string {
            let offer = Self::session_desc_from_str::<RTCSessionDescription>(c_s.to_string())?;
            peer_conn.set_remote_description(offer).await.map_err(|e| {
                GameError::CustomError(format!("Failed to set remote description {:?}", e))
            })?;
            let answer = peer_conn
                .create_answer(None)
                .await
                .map_err(|e| GameError::CustomError(format!("Failed to create answer {:?}", e)))?;
            answer
        } else {
            peer_conn
                .create_offer(None)
                .await
                .map_err(|_| GameError::CustomError("Failed to create offer".to_string()))?
        };
        let mut g_c = peer_conn.gathering_complete_promise().await;
        peer_conn.set_local_description(offer).await.map_err(|e| {
            GameError::CustomError(format!("Failed to set local description {:?}", e))
        })?;
        g_c.recv().await;

        // Push this into the tx
        let base64_conn_str = if let Some(l_d) = peer_conn.local_description().await {
            base64::encode(serde_json::to_string(&l_d).map_err(|e| {
                GameError::CustomError("Failed to convert peer base64 to json".to_string())
            })?)
        } else {
            return Err(GameError::CustomError(
                "Failed to get peer base64".to_string(),
            ));
        };

        tx.send(MultiplayerGameMessage::ConnectionString(base64_conn_str))
            .map_err(|_| GameError::CustomError("Failed to send the conn str to tx".to_string()))?;

        if conn_string.is_none() {
            if let Ok(MultiplayerGameMessage::ConnectionString(s)) = rx.recv() {
                peer_conn
                    .set_remote_description(
                        Self::session_desc_from_str(s)
                            .expect("Failed to parse remote description string"),
                    )
                    .await
                    .expect("Failed to set remote description");
                println!("peer conn has set remote desc");
            }
        }
        if let Ok(true) = rx_exit.recv() {}
        println!("Event thread exiting");

        Ok(())
    }

    pub fn create_game(conn_string: Option<String>) -> GameResult<Self> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let (tx, rx) = flume::unbounded::<MultiplayerGameMessage>();
        let (push_tx, push_rx) = flume::unbounded::<MultiplayerGameMessage>();
        std::thread::spawn(move || {
            rt.block_on(Self::create_game_async(conn_string, push_tx, rx))
                .unwrap();
        });

        Ok(Self {
            event_buffer: push_rx,
            event_push_buffer: tx,
        })
    }
}
