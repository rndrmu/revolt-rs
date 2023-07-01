
use std::{sync::{Arc}, time::Duration};
use tracing::{debug, info, error};

use futures_util::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};
use serde_json::json;
use tokio::{net::TcpStream, spawn, sync::{Mutex, mpsc::{UnboundedSender, UnboundedReceiver}}};
use crate::{models::{message::Message as RevoltMessage, ready::Ready, gateway::{message::MessageUpdate, message::MessageDelete, message::MessageReact, message::MessageUnreact, message::MessageRemoveReactions, channel::ChannelCreate}, gateway::{channel::{ChannelUpdate, ChannelDelete, ChannelGroupJoin, ChannelGroupLeave, ChannelStartTyping, ChannelStopTyping, ChannelAck}, server::{ServerUpdate, ServerDelete, ServerMemberUpdate, ServerMemberJoin, ServerMemberLeave, ServerRoleUpdate, ServerRoleDelete}, user::{UserUpdate, UserRelationship}}}, context::Context, http::Http, client::EventHandler};

use async_tungstenite::tungstenite::*;
use async_tungstenite::{tokio::{ConnectStream, connect_async}, tungstenite::connect};
use async_tungstenite::WebSocketStream;

pub type WsStream = WebSocketStream<ConnectStream>;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

type JsonParseResult<T> = std::result::Result<T, serde_json::Error>;

pub struct Shard {
    ws: WsStream,
    event_handler: Arc<dyn EventHandler>,
}

pub(crate) async fn create_client(url: String) -> Result<WsStream> {
    let config = async_tungstenite::tungstenite::protocol::WebSocketConfig {
        max_message_size: None,
        max_frame_size: None,
        max_send_queue: None,
        accept_unmasked_frames: false,
    };
    let (stream, _) =
        async_tungstenite::tokio::connect_async_with_config(url, Some(config)).await?;

    Ok(stream)
}

impl Shard {
    pub async fn new(socket_url: String ,handler: Arc<dyn EventHandler>) -> Self {
        let ws = create_client(socket_url).await.unwrap();

        Self {
            event_handler: handler,
            ws
        }
    }

    pub async fn connect(mut self, token: String) {
        self.authenticate(token.clone()).await;
        
         // Spawn a task to receive messages from the reader and send them to the channel
         let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

         // Spawn a task to receive messages from the reader and send them to the channel
        let mut socket_clone = self.ws;
         tokio::spawn(async move {
             while let Some(Ok(message)) = socket_clone.next().await {
                 // Non-blocking send operation
                 if let Err(e) = sender.send(message) {
                     error!("Error sending message: {:?}", e);
                 }
             }
         });

        // Spawn a task to process the messages received on the channel
        let handler = self.event_handler.clone();

        tokio::spawn(async move {
            crate::gateway::shard::handle_events(receiver, Arc::new(token), handler).await;
        }).await.unwrap_or_else(|e| {
            error!("Error spawning task: {:?}", e);
        });
    }

    async fn authenticate(&mut self, token: String) {
        self.ws.send(Message::Text(json!({
            "type": "Authenticate",
            "token": token
        }).to_string())).await.unwrap_or_else(|e| {
            error!("Error sending message: {:?}", e);
        });
    }



       
}


pub async fn handle_events(
    mut receiver: UnboundedReceiver<Message>,
    token: Arc<String>,
    event: Arc<dyn EventHandler>
) {
        while let Some(message) = receiver.recv().await {

                    if message.is_text() {
                        let json: serde_json::Value = serde_json::from_str(&message.to_string()).unwrap();
                        let json_clone = json.clone();
                        info!("Received message: {:?}", json);
                        
                        match json["type"].as_str() {
                            Some("Ready") => {
                                let ready: Ready = serde_json::from_value(json).unwrap();
                                event.ready(Context::new(&token, &message.to_string()), ready).await;

                                
                            },
                            
                            Some("Authenticated") => {
                                event.authenticated().await;

                                // spawn heartbeat thread 
                                
                            },
                            Some("Message") => {
                                let message: JsonParseResult<crate::models::message::Message> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.on_message(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("MessageUpdate") => {
                                let message: JsonParseResult<MessageUpdate> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.message_update(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("MessageDelete") => {
                                let message: JsonParseResult<MessageDelete> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.message_delete(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("MessageReact") => {
                                let message: JsonParseResult<MessageReact> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.message_react(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                                
                            },

                            Some("MessageUnreact") => {
                                let message: JsonParseResult<MessageUnreact> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.message_unreact(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("MessageRemoveReactions") => {
                                let message: JsonParseResult<MessageRemoveReactions> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.message_remove_reactions(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("ChannelCreate") => {
                                let message: JsonParseResult<ChannelCreate> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.channel_create(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("ChannelUpdate") => {
                                let message: JsonParseResult<ChannelUpdate> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.channel_update(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("ChannelDelete") => {
                                let message: JsonParseResult<ChannelDelete> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.channel_delete(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            /* Some("ServerCreate") => {
                                let message: Result<ServerCreate> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.server_create(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            }, */

                            Some("ServerUpdate") => {
                                let message: JsonParseResult<ServerUpdate> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.server_update(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("ServerDelete") => {
                                let message: JsonParseResult<ServerDelete> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.server_delete(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("ChannelGroupJoin") => {
                                let message: JsonParseResult<ChannelGroupJoin> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.channel_group_join(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("ChannelGroup") => {
                                let message: JsonParseResult<ChannelGroupLeave> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.channel_group_leave(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("ChannelStartTyping") => {
                                let message: JsonParseResult<ChannelStartTyping> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.channel_start_typing(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("ChannelStopTyping") => {
                                let message: JsonParseResult<ChannelStopTyping> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.channel_stop_typing(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("ChannelAck") => {
                                let message: JsonParseResult<ChannelAck> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.channel_ack(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("ServerUpdate") => {
                                let message: JsonParseResult<ServerUpdate> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.server_update(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("ServerDelete") => {
                                let message: JsonParseResult<ServerDelete> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.server_delete(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("ServerMemberJoin") => {
                                let message: JsonParseResult<ServerMemberJoin> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.server_member_join(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("ServerMemberLeave") => {
                                let message: JsonParseResult<ServerMemberLeave> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.server_member_leave(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("ServerMemberUpdate") => {
                                let message: JsonParseResult<ServerMemberUpdate> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.server_member_update(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("ServerRoleUpdate") => {
                                let message: JsonParseResult<ServerRoleUpdate> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.server_role_update(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("ServerRoleDelete") => {
                                let message: JsonParseResult<ServerRoleDelete> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.server_role_delete(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("UserUpdate") => {
                                let message: JsonParseResult<UserUpdate> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.user_update(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },

                            Some("UserRelationship") => {
                                let message: JsonParseResult<UserRelationship> = serde_json::from_value(json);
                                if let Ok(message) = message {
                                    event.user_relationship(Context::new(&token, &json_clone.to_string()), message).await;
                                }
                            },


                            Some(&_) => {
                                info!("[GATEWAY_RECV] Received Unknown Message Type: {} -> {}", json["type"].as_str().unwrap(), json);
                            },
                            None => {},
                        }
                    }
                
            }
    }