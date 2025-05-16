use crate::models::{AppState, ChatMessage, UserSession};
use actix_ws::Message;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;
use log;

pub async fn handle_message(msg: Message, user_id: &str, app_state: &Arc<AppState>) -> bool {
    match msg {
        Message::Text(text) => {
            log::debug!("Received message from {}: {}", user_id, text);
            
            match serde_json::from_str::<ChatMessage>(&text) {
                Ok(mut chat_msg) => {
                    let current_room;
                    let current_username;
                    
                    {
                        let mut sessions = app_state.sessions.lock().unwrap();
                        if let Some(user_session) = sessions.get_mut(user_id) {
                            user_session.last_heartbeat = Instant::now();
                            
                            if user_session.username == "未命名用户" && chat_msg.username != "未命名用户" {
                                user_session.username = chat_msg.username.clone();
                                let join_msg = ChatMessage {
                                    msg_type: "system".to_string(),
                                    username: "服务器".to_string(),
                                    room: user_session.room.clone(),
                                    text: format!("{} 加入了聊天室", chat_msg.username),
                                    timestamp: chrono::Utc::now().timestamp() as u64,
                                    id: Uuid::new_v4().to_string(),
                                    target: None,
                                };
                                
                                broadcast_message_to_room(&join_msg, &user_session.room, app_state).await;
                                send_user_list(app_state, &user_session.room).await;
                            }
                            
                            current_room = user_session.room.clone();
                            current_username = user_session.username.clone();
                        } else {
                            return false;
                        }
                    }
                    
                    match chat_msg.msg_type.as_str() {
                        "chat" => {
                            chat_msg.username = current_username;
                            chat_msg.room = current_room.clone();
                            chat_msg.timestamp = chrono::Utc::now().timestamp() as u64;
                            broadcast_message_to_room(&chat_msg, &current_room, app_state).await;
                        },
                        "private" => {
                            if let Some(target_username) = &chat_msg.target {
                                chat_msg.username = current_username.clone();
                                chat_msg.timestamp = chrono::Utc::now().timestamp() as u64;
                                
                                let target_user_id = find_user_by_name(target_username, app_state);
                                
                                if let Some(target_id) = target_user_id {
                                    send_message_to_user(&chat_msg, &target_id, app_state).await;
                                    send_message_to_user(&chat_msg, user_id, app_state).await;
                                    log::info!("Private message from {} to {}", current_username, target_username);
                                } else {
                                    let error_msg = ChatMessage {
                                        msg_type: "system".to_string(),
                                        username: "服务器".to_string(),
                                        room: current_room.clone(),
                                        text: format!("用户 {} 不在线或不存在", target_username),
                                        timestamp: chrono::Utc::now().timestamp() as u64,
                                        id: Uuid::new_v4().to_string(),
                                        target: None,
                                    };
                                    
                                    send_message_to_user(&error_msg, user_id, app_state).await;
                                }
                            }
                        },
                        "ping" => {
                            let pong_msg = ChatMessage {
                                msg_type: "pong".to_string(),
                                username: "服务器".to_string(),
                                room: "".to_string(),
                                text: chat_msg.text,
                                timestamp: chrono::Utc::now().timestamp() as u64,
                                id: Uuid::new_v4().to_string(),
                                target: None,
                            };
                            send_message_to_user(&pong_msg, user_id, app_state).await;
                        },
                        "pong" => {
                            let mut sessions = app_state.sessions.lock().unwrap();
                            if let Some(user_session) = sessions.get_mut(user_id) {
                                user_session.last_heartbeat = Instant::now();
                            }
                        },
                        "join" => {
                            if !chat_msg.room.is_empty() {
                                let new_room = chat_msg.room.clone();
                                join_room(user_id, &new_room, app_state).await;
                            }
                        },
                        "command" => {
                            let response = handle_command(chat_msg.text.clone(), user_id, app_state).await;
                            
                            if !response.is_empty() {
                                let cmd_response = ChatMessage {
                                    msg_type: "system".to_string(),
                                    username: "服务器".to_string(),
                                    room: current_room,
                                    text: response,
                                    timestamp: chrono::Utc::now().timestamp() as u64,
                                    id: Uuid::new_v4().to_string(),
                                    target: None,
                                };
                                
                                send_message_to_user(&cmd_response, user_id, app_state).await;
                            }
                        },
                        _ => {
                            log::warn!("Unknown message type: {}", chat_msg.msg_type);
                        }
                    }
                },
                Err(e) => {
                    log::error!("Failed to parse message: {:?}, error: {:?}", text, e);
                    
                    let error_msg = ChatMessage {
                        msg_type: "system".to_string(),
                        username: "服务器".to_string(),
                        room: "".to_string(),
                        text: "消息格式错误，请检查客户端代码".to_string(),
                        timestamp: chrono::Utc::now().timestamp() as u64,
                        id: Uuid::new_v4().to_string(),
                        target: None,
                    };
                    
                    send_message_to_user(&error_msg, user_id, app_state).await;
                }
            }
            
            true
        },
        Message::Close(reason) => {
            log::info!("Client {} disconnected: {:?}", user_id, reason);
            false
        },
        Message::Ping(bytes) => {
            let mut sessions = app_state.sessions.lock().unwrap();
            if let Some(user_session) = sessions.get_mut(user_id) {
                user_session.last_heartbeat = Instant::now();
                if let Err(e) = user_session.session.pong(&bytes).await {
                    log::error!("Error sending pong to {}: {:?}", user_id, e);
                    return false;
                }
            }
            true
        },
        Message::Pong(_) => {
            let mut sessions = app_state.sessions.lock().unwrap();
            if let Some(user_session) = sessions.get_mut(user_id) {
                user_session.last_heartbeat = Instant::now();
            }
            true
        },
        Message::Binary(_) => true,
        Message::Continuation(_) => true,
        Message::Nop => true,
    }
}

pub async fn broadcast_message_to_room(message: &ChatMessage, room: &str, app_state: &Arc<AppState>) {
    log::info!("Broadcasting to room {}: type={}, from={}, text={}", 
               room, message.msg_type, message.username, 
               if message.text.len() > 30 { format!("{}...", &message.text[..30]) } else { message.text.clone() });
    
    let user_ids = {
        let rooms = app_state.rooms.lock().unwrap();
        match rooms.get(room) {
            Some(user_set) => {
                let users = user_set.clone();
                log::info!("Room {} has {} users: {:?}", room, users.len(), &users);
                users
            },
            None => {
                log::warn!("Trying to broadcast to non-existent room: {}", room);
                return;
            }
        }
    };
    
    if user_ids.is_empty() {
        log::warn!("No users in room {}, message not delivered", room);
        return;
    }
    
    let message_json = match serde_json::to_string(message) {
        Ok(json) => json,
        Err(e) => {
            log::error!("Failed to serialize message: {:?}", e);
            return;
        }
    };
    
    let mut sessions = app_state.sessions.lock().unwrap();
    for user_id in user_ids {
        if let Some(user_session) = sessions.get_mut(&user_id) {
            log::debug!("Sending to user {} in room {}: {:?}", user_session.username, room, message);
            if let Err(e) = user_session.session.text(message_json.clone()).await {
                log::error!("Error sending message to {}: {:?}", user_id, e);
            } else {
                log::debug!("Message sent successfully to user {}", user_session.username);
            }
        } else {
            log::warn!("User {} not found in sessions", user_id);
        }
    }
}

pub async fn send_message_to_user(message: &ChatMessage, user_id: &str, app_state: &Arc<AppState>) {
    log::debug!("Sending to user {}: {:?}", user_id, message);
    
    let message_json = match serde_json::to_string(message) {
        Ok(json) => json,
        Err(e) => {
            log::error!("Failed to serialize message: {:?}", e);
            return;
        }
    };
    
    let mut sessions = app_state.sessions.lock().unwrap();
    if let Some(user_session) = sessions.get_mut(user_id) {
        if let Err(e) = user_session.session.text(message_json).await {
            log::error!("Error sending message to {}: {:?}", user_id, e);
        }
    }
}

pub async fn send_user_list(app_state: &Arc<AppState>, room: &str) {
    let mut user_list = Vec::new();
    
    {
        let sessions = app_state.sessions.lock().unwrap();
        let rooms = app_state.rooms.lock().unwrap();
        
        if let Some(user_ids) = rooms.get(room) {
            for user_id in user_ids {
                if let Some(user_session) = sessions.get(user_id) {
                    user_list.push(format!("{}:{}", user_session.username, user_session.addr));
                }
            }
        }
    }
    
    let user_list_msg = ChatMessage {
        msg_type: "userlist".to_string(),
        username: "服务器".to_string(),
        room: room.to_string(),
        text: user_list.join(","),
        timestamp: chrono::Utc::now().timestamp() as u64,
        id: Uuid::new_v4().to_string(),
        target: None,
    };
    
    broadcast_message_to_room(&user_list_msg, room, app_state).await;
}

pub fn find_user_by_name(username: &str, app_state: &Arc<AppState>) -> Option<String> {
    let sessions = app_state.sessions.lock().unwrap();
    
    for (id, session) in sessions.iter() {
        if session.username == username {
            return Some(id.clone());
        }
    }
    
    None
}

pub async fn join_room(user_id: &str, new_room: &str, app_state: &Arc<AppState>) {
    let username;
    let old_room;
    
    {
        let mut sessions = app_state.sessions.lock().unwrap();
        
        if let Some(user_session) = sessions.get_mut(user_id) {
            username = user_session.username.clone();
            old_room = user_session.room.clone();
            
            if old_room == new_room {
                let already_msg = ChatMessage {
                    msg_type: "system".to_string(),
                    username: "服务器".to_string(),
                    room: old_room.clone(),
                    text: format!("您已经在房间 {} 中", new_room),
                    timestamp: chrono::Utc::now().timestamp() as u64,
                    id: Uuid::new_v4().to_string(),
                    target: None,
                };
                
                send_message_to_user(&already_msg, user_id, app_state).await;
                return;
            }
            
            user_session.room = new_room.to_string();
        } else {
            return;
        }
    }
    
    {
        let mut rooms = app_state.rooms.lock().unwrap();
        if let Some(room_users) = rooms.get_mut(&old_room) {
            room_users.remove(user_id);
        }
    }
    
    let leave_msg = ChatMessage {
        msg_type: "system".to_string(),
        username: "服务器".to_string(),
        room: old_room.clone(),
        text: format!("{} 离开了房间", username),
        timestamp: chrono::Utc::now().timestamp() as u64,
        id: Uuid::new_v4().to_string(),
        target: None,
    };
    
    broadcast_message_to_room(&leave_msg, &old_room, app_state).await;
    
    {
        let mut rooms = app_state.rooms.lock().unwrap();
        rooms.entry(new_room.to_string())
             .or_insert_with(std::collections::HashSet::new)
             .insert(user_id.to_string());
    }
    
    let join_msg = ChatMessage {
        msg_type: "system".to_string(),
        username: "服务器".to_string(),
        room: new_room.to_string(),
        text: format!("{} 加入了房间", username),
        timestamp: chrono::Utc::now().timestamp() as u64,
        id: Uuid::new_v4().to_string(),
        target: None,
    };
    
    broadcast_message_to_room(&join_msg, new_room, app_state).await;
    
    send_user_list(app_state, &old_room).await;
    send_user_list(app_state, new_room).await;
    
    log::info!("User {} moved from room {} to room {}", username, old_room, new_room);
}

pub async fn handle_disconnect(user_id: &str, app_state: &Arc<AppState>) {
    let username;
    let room;
    
    {
        let mut sessions = app_state.sessions.lock().unwrap();
        if let Some(user_session) = sessions.remove(user_id) {
            username = user_session.username;
            room = user_session.room;
            
            let mut rooms = app_state.rooms.lock().unwrap();
            if let Some(room_users) = rooms.get_mut(&room) {
                room_users.remove(user_id);
                if room != "大厅" && room_users.is_empty() {
                    rooms.remove(&room);
                }
            }
        } else {
            return;
        }
    }
    
    if username != "未命名用户" {
        let leave_msg = ChatMessage {
            msg_type: "system".to_string(),
            username: "服务器".to_string(),
            room: room.clone(),
            text: format!("{} 离开了聊天室", username),
            timestamp: chrono::Utc::now().timestamp() as u64,
            id: Uuid::new_v4().to_string(),
            target: None,
        };
        
        broadcast_message_to_room(&leave_msg, &room, app_state).await;
        
        send_user_list(app_state, &room).await;
    }
    
    log::info!("Connection closed for {} ({})", user_id, username);
}

pub async fn handle_command(command: String, user_id: &str, app_state: &Arc<AppState>) -> String {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return "请输入有效命令".to_string();
    }
    
    match parts[0] {
        "/help" => {
            return "可用命令:\n\
                   /help - 显示帮助\n\
                   /rooms - 显示所有房间\n\
                   /join <房间名> - 加入指定房间\n\
                   /users - 显示当前房间用户\n\
                   /msg <用户名> <消息> - 发送私聊消息\n\
                   /ping - 测试网络连接\n\
                   /stats - 显示网络统计信息".to_string();
        },
        "/rooms" => {
            let rooms = app_state.rooms.lock().unwrap();
            let room_list: Vec<String> = rooms.keys()
                .map(|name| format!("{} ({} 人在线)", name, rooms[name].len()))
                .collect();
            return format!("可用房间: \n{}", room_list.join("\n"));
        },
        "/users" => {
            let mut user_count = 0;
            let mut user_list = Vec::new();
            
            let sessions = app_state.sessions.lock().unwrap();
            if let Some(user_session) = sessions.get(user_id) {
                let room = &user_session.room;
                let rooms = app_state.rooms.lock().unwrap();
                
                if let Some(user_ids) = rooms.get(room) {
                    user_count = user_ids.len();
                    for uid in user_ids {
                        if let Some(u_session) = sessions.get(uid) {
                            user_list.push(format!("{} ({})", u_session.username, u_session.addr));
                        }
                    }
                }
            }
            
            return format!("当前房间有 {} 名用户:\n{}", user_count, user_list.join("\n"));
        },
        "/ping" => {
            let ping_msg = ChatMessage {
                msg_type: "ping".to_string(),
                username: "服务器".to_string(),
                room: "".to_string(),
                text: chrono::Utc::now().timestamp_micros().to_string(),
                timestamp: chrono::Utc::now().timestamp() as u64,
                id: Uuid::new_v4().to_string(),
                target: None,
            };
            
            send_message_to_user(&ping_msg, user_id, app_state).await;
            return "".to_string();
        },
        "/stats" => {
            return format!(
                "网络统计信息:\n\
                 总连接数: {}\n\
                 总房间数: {}",
                app_state.sessions.lock().unwrap().len(),
                app_state.rooms.lock().unwrap().len()
            );
        },
        _ => return format!("未知命令: {}", command),
    }
} 