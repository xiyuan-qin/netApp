use actix_files as fs;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, middleware};
use actix_ws::Message;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;

// 定义消息类型
#[derive(Serialize, Deserialize, Clone, Debug)]
struct ChatMessage {
    msg_type: String, // "chat", "system", "command", "ping", "pong", "join", "leave", "userlist", "private"
    username: String,
    room: String,
    text: String,
    timestamp: u64,
    id: String, // 消息唯一ID，用于确认机制
    target: Option<String>, // 私聊目标用户名
}

// 用户会话信息
struct UserSession {
    id: String,
    username: String,
    room: String,
    addr: String,  // 客户端IP地址
    session: actix_ws::Session,
    last_heartbeat: Instant,
    join_time: Instant, // 添加加入时间字段，用于会话管理
}

// 应用状态
struct AppState {
    sessions: Mutex<HashMap<String, UserSession>>,
    rooms: Mutex<HashMap<String, HashSet<String>>>, // room_name -> set of user_ids
}

// 通过用户名查找用户ID
fn find_user_by_name(username: &str, app_state: &Arc<AppState>) -> Option<String> {
    let sessions = app_state.sessions.lock().unwrap();
    
    for (id, session) in sessions.iter() {
        if session.username == username {
            return Some(id.clone());
        }
    }
    
    None
}

// 处理WebSocket连接
async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<Arc<AppState>>,
) -> Result<HttpResponse, Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;
    
    // 获取客户端IP地址
    let connection_info = req.connection_info();
    let client_addr = connection_info.peer_addr().unwrap_or("unknown").to_string();
    
    // 为新连接创建唯一标识符
    let id = Uuid::new_v4().to_string();
    log::info!("New WebSocket connection from {}, session_id: {}", client_addr, &id);
    
    // 生成随机数字后缀的用户名，避免用户名冲突
    let random_suffix = rand::random::<u16>() % 1000;
    let default_username = format!("用户{}", random_suffix);
    
    // 初始化用户会话(用户名和房间稍后会通过消息更新)
    let user_session = UserSession {
        id: id.clone(),
        username: default_username.clone(),
        room: "大厅".to_string(),
        addr: client_addr.clone(),
        session: session.clone(),
        last_heartbeat: Instant::now(),
        join_time: Instant::now(),
    };
    
    // 存储连接前先检查并清理可能存在的同IP陈旧连接
    {
        let mut sessions = app_state.sessions.lock().unwrap();
        let mut stale_sessions = Vec::new();
        
        // 检查是否有来自相同IP的陈旧连接
        for (session_id, existing_session) in sessions.iter() {
            // 如果是相同IP地址并且心跳超过60秒，认为是陈旧连接
            if existing_session.addr == client_addr && 
               existing_session.last_heartbeat.elapsed() > Duration::from_secs(60) {
                stale_sessions.push(session_id.clone());
            }
        }
        
        // 移除陈旧连接
        for stale_id in &stale_sessions {
            log::info!("Removing stale connection: {} from same IP {}", stale_id, client_addr);
            if let Some(stale_session) = sessions.remove(stale_id) {
                // 从房间中移除
                let mut rooms = app_state.rooms.lock().unwrap();
                if let Some(room_users) = rooms.get_mut(&stale_session.room) {
                    room_users.remove(stale_id);
                }
            }
        }
        
        // 添加新连接
        sessions.insert(id.clone(), user_session);
        
        // 将用户添加到默认房间
        let mut rooms = app_state.rooms.lock().unwrap();
        rooms.entry("大厅".to_string())
             .or_insert_with(HashSet::new)
             .insert(id.clone());
    }
    
    // 发送连接成功消息与服务器信息
    let server_info = ChatMessage {
        msg_type: "system".to_string(),
        username: "服务器".to_string(),
        room: "大厅".to_string(),
        text: format!("连接成功！服务器信息: 本地地址 {}，您的IP地址: {}", 
                     req.connection_info().host(), client_addr),
        timestamp: chrono::Utc::now().timestamp() as u64,
        id: Uuid::new_v4().to_string(),
        target: None,
    };
    
    // 记录信息到日志，帮助调试
    log::info!("Sending welcome message to new connection {}", id);
    
    if let Err(e) = session.text(serde_json::to_string(&server_info).unwrap()).await {
        log::error!("Error sending welcome message: {:?}", e);
    }
    
    // 初始化时发送默认用户名
    let init_msg = ChatMessage {
        msg_type: "chat".to_string(),
        username: default_username,
        room: "大厅".to_string(),
        text: "".to_string(),
        timestamp: chrono::Utc::now().timestamp() as u64,
        id: Uuid::new_v4().to_string(),
        target: None,
    };
    
    if let Err(e) = session.text(serde_json::to_string(&init_msg).unwrap()).await {
        log::error!("Error sending init message: {:?}", e);
    }
    
    // 发送当前在线用户列表
    send_user_list(&app_state, "大厅").await;
    
    // 在新线程处理消息
    let app_state_clone = app_state.clone();
    let id_clone = id.clone();
    
    actix_web::rt::spawn(async move {
        let mut ping_interval = actix_web::rt::time::interval(Duration::from_secs(30));
        
        loop {
            tokio::select! {
                // 处理接收到的WebSocket消息
                msg = msg_stream.recv() => {
                    match msg {
                        Some(Ok(ws_msg)) => {
                            if !handle_message(ws_msg, &id_clone, &app_state_clone).await {
                                log::info!("Connection {} message handler returned false, breaking loop", id_clone);
                                break;
                            }
                        }
                        Some(Err(e)) => {
                            log::error!("WebSocket error for {}: {:?}", id_clone, e);
                            break;
                        }
                        None => {
                            log::info!("WebSocket stream ended for {}", id_clone);
                            break;
                        }
                    }
                }
                
                // 定时发送ping检查连接状态
                _ = ping_interval.tick() => {
                    let mut sessions = app_state_clone.sessions.lock().unwrap();
                    if let Some(user_session) = sessions.get_mut(&id_clone) {
                        // 如果超过90秒没有心跳，断开连接
                        if user_session.last_heartbeat.elapsed() > Duration::from_secs(90) {
                            log::info!("Client {} timed out", id_clone);
                            break;
                        }
                        
                        // 发送ping消息
                        let ping_msg = ChatMessage {
                            msg_type: "ping".to_string(),
                            username: "服务器".to_string(),
                            room: "".to_string(),
                            text: "".to_string(),
                            timestamp: chrono::Utc::now().timestamp() as u64,
                            id: Uuid::new_v4().to_string(),
                            target: None,
                        };
                        
                        if let Err(e) = user_session.session.text(serde_json::to_string(&ping_msg).unwrap()).await {
                            log::error!("Error sending ping to {}: {:?}", id_clone, e);
                            break;
                        }
                    } else {
                        log::warn!("Session {} not found during ping", id_clone);
                        break;
                    }
                }
            }
        }
        
        // 连接关闭，处理用户离开
        log::info!("WebSocket handler loop exited for {}, cleaning up", id_clone);
        handle_disconnect(&id_clone, &app_state_clone).await;
    });
    
    Ok(response)
}

// 处理接收到的消息
async fn handle_message(msg: Message, user_id: &str, app_state: &Arc<AppState>) -> bool {
    match msg {
        Message::Text(text) => {
            log::debug!("Received message from {}: {}", user_id, text);
            
            // 尝试解析为JSON消息
            match serde_json::from_str::<ChatMessage>(&text) {
                Ok(mut chat_msg) => {
                    // 声明变量但暂不初始化
                    let current_room;
                    let current_username;
                    
                    // 更新会话信息
                    {
                        let mut sessions = app_state.sessions.lock().unwrap();
                        if let Some(user_session) = sessions.get_mut(user_id) {
                            user_session.last_heartbeat = Instant::now();
                            
                            // 如果是第一次设置用户名，处理加入房间
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
                                send_user_list(&app_state, &user_session.room).await;
                            }
                            
                            current_room = user_session.room.clone();
                            current_username = user_session.username.clone();
                        } else {
                            return false; // 用户会话不存在
                        }
                    }
                    
                    // 根据消息类型处理
                    match chat_msg.msg_type.as_str() {
                        "chat" => {
                            // 修正发送者信息并广播
                            chat_msg.username = current_username;
                            chat_msg.room = current_room.clone();
                            chat_msg.timestamp = chrono::Utc::now().timestamp() as u64;
                            
                            broadcast_message_to_room(&chat_msg, &current_room, app_state).await;
                        },
                        "private" => {
                            // 处理私聊消息
                            if let Some(target_username) = &chat_msg.target {
                                // 修正发送者信息
                                chat_msg.username = current_username.clone();
                                chat_msg.timestamp = chrono::Utc::now().timestamp() as u64;
                                
                                // 查找目标用户
                                let target_user_id = find_user_by_name(target_username, app_state);
                                
                                if let Some(target_id) = target_user_id {
                                    // 发送给接收方
                                    send_message_to_user(&chat_msg, &target_id, app_state).await;
                                    
                                    // 也发送给发送方（回显）
                                    send_message_to_user(&chat_msg, user_id, app_state).await;
                                    
                                    log::info!("Private message from {} to {}", current_username, target_username);
                                } else {
                                    // 用户不存在，发送错误消息
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
                            // 处理客户端ping请求，直接回复pong消息
                            let pong_msg = ChatMessage {
                                msg_type: "pong".to_string(),
                                username: "服务器".to_string(),
                                room: "".to_string(),
                                text: chat_msg.text, // 返回相同的内容，客户端可用于计算延迟
                                timestamp: chrono::Utc::now().timestamp() as u64,
                                id: Uuid::new_v4().to_string(),
                                target: None,
                            };
                            send_message_to_user(&pong_msg, user_id, app_state).await;
                        },
                        "pong" => {
                            // 处理客户端的pong响应
                            let mut sessions = app_state.sessions.lock().unwrap();
                            if let Some(user_session) = sessions.get_mut(user_id) {
                                user_session.last_heartbeat = Instant::now();
                            }
                        },
                        "join" => {
                            // 处理用户加入/创建房间请求
                            if !chat_msg.room.is_empty() {
                                let new_room = chat_msg.room.clone();
                                join_room(user_id, &new_room, app_state).await;
                            }
                        },
                        "command" => {
                            // 处理命令消息
                            let response = handle_command(chat_msg.text.clone(), user_id, app_state).await;
                            
                            // 发送命令响应
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
                    
                    // 发送错误消息给用户
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
            // 处理WebSocket协议层Ping
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
            // 处理WebSocket协议层Pong
            let mut sessions = app_state.sessions.lock().unwrap();
            if let Some(user_session) = sessions.get_mut(user_id) {
                user_session.last_heartbeat = Instant::now();
            }
            true
        },
        Message::Binary(_) => {
            // 暂不处理二进制消息
            true
        },
        Message::Continuation(_) => {
            // 暂不处理分片消息
            true
        },
        Message::Nop => true,
    }
}

// 新增加入房间的独立函数，确保创建房间逻辑统一
async fn join_room(user_id: &str, new_room: &str, app_state: &Arc<AppState>) {
    let username;
    let old_room;
    
    {
        let mut sessions = app_state.sessions.lock().unwrap();
        
        if let Some(user_session) = sessions.get_mut(user_id) {
            username = user_session.username.clone();
            old_room = user_session.room.clone();
            
            // 检查是否已经在该房间
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
            
            // 更新用户房间
            user_session.room = new_room.to_string();
        } else {
            return;
        }
    }
    
    // 从旧房间移除用户
    {
        let mut rooms = app_state.rooms.lock().unwrap();
        if let Some(room_users) = rooms.get_mut(&old_room) {
            room_users.remove(user_id);
        }
    }
    
    // 发送离开消息到旧房间
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
    
    // 将用户添加到新房间
    {
        let mut rooms = app_state.rooms.lock().unwrap();
        rooms.entry(new_room.to_string())
             .or_insert_with(HashSet::new)
             .insert(user_id.to_string());
    }
    
    // 发送加入消息到新房间
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
    
    // 更新两个房间的用户列表
    send_user_list(app_state, &old_room).await;
    send_user_list(app_state, new_room).await;
    
    log::info!("User {} moved from room {} to room {}", username, old_room, new_room);
}

// 处理用户断开连接
async fn handle_disconnect(user_id: &str, app_state: &Arc<AppState>) {
    let username;
    let room;
    
    // 获取用户信息并从会话中移除
    {
        let mut sessions = app_state.sessions.lock().unwrap();
        if let Some(user_session) = sessions.remove(user_id) {
            username = user_session.username;
            room = user_session.room;
            
            // 从房间中移除用户
            let mut rooms = app_state.rooms.lock().unwrap();
            if let Some(room_users) = rooms.get_mut(&room) {
                room_users.remove(user_id);
                // 如果房间为空且非大厅，则移除房间
                if room != "大厅" && room_users.is_empty() {
                    rooms.remove(&room);
                }
            }
        } else {
            return;
        }
    }
    
    // 通知其他用户
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
        
        // 更新用户列表
        send_user_list(app_state, &room).await;
    }
    
    log::info!("Connection closed for {} ({})", user_id, username);
}

// 向指定房间广播消息
async fn broadcast_message_to_room(message: &ChatMessage, room: &str, app_state: &Arc<AppState>) {
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

// 发送消息给特定用户
async fn send_message_to_user(message: &ChatMessage, user_id: &str, app_state: &Arc<AppState>) {
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

// 发送用户列表信息
async fn send_user_list(app_state: &Arc<AppState>, room: &str) {
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

// 处理命令
async fn handle_command(command: String, user_id: &str, app_state: &Arc<AppState>) -> String {
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
            // 直接发送ping消息，而不是返回文本
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
            
            // 返回空字符串，因为ping消息已经直接发送
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

// 静态文件路由
async fn index() -> actix_web::Result<fs::NamedFile> {
    // 修改为使用Vue构建的index.html文件
    Ok(fs::NamedFile::open("vue-client/dist/index.html")?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    log::info!("启动计算机网络实验服务器在 http://localhost:8080");
    
    let app_state = web::Data::new(Arc::new(AppState {
        sessions: Mutex::new(HashMap::new()),
        rooms: Mutex::new({
            let mut rooms = HashMap::new();
            rooms.insert("大厅".to_string(), HashSet::new());
            rooms
        }),
    }));
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(middleware::Logger::default())
            .service(web::resource("/ws").route(web::get().to(ws_route)))
            // Use only one handler for the root path
            .service(fs::Files::new("/", "vue-client/dist").index_file("index.html"))
    })
    .bind("0.0.0.0:8080")?  // 监听所有接口，方便局域网内访问
    .run()
    .await
}