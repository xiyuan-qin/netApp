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
    log::info!("New WebSocket connection from {}", client_addr);
    
    // 为新连接创建唯一标识符
    let id = Uuid::new_v4().to_string();
    
    // 初始化用户会话(用户名和房间稍后会通过消息更新)
    let user_session = UserSession {
        id: id.clone(),
        username: "未命名用户".to_string(),
        room: "大厅".to_string(),
        addr: client_addr.clone(),
        session: session.clone(),
        last_heartbeat: Instant::now(),
    };
    
    // 存储连接
    {
        let mut sessions = app_state.sessions.lock().unwrap();
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
    
    if let Err(e) = session.text(serde_json::to_string(&server_info).unwrap()).await {
        log::error!("Error sending welcome message: {:?}", e);
    }
    
    // 发送当前在线用户列表
    send_user_list(&app_state, "大厅".to_string()).await;
    
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
                                break;
                            }
                        }
                        Some(Err(e)) => {
                            log::error!("WebSocket error: {:?}", e);
                            break;
                        }
                        None => break,
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
                        break;
                    }
                }
            }
        }
        
        // 连接关闭，处理用户离开
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
                                send_user_list(&app_state, user_session.room.clone()).await;
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
                        "pong" => {
                            // 处理客户端的pong响应
                            let mut sessions = app_state.sessions.lock().unwrap();
                            if let Some(user_session) = sessions.get_mut(user_id) {
                                user_session.last_heartbeat = Instant::now();
                            }
                        },
                        "join" => {
                            // 处理用户加入房间请求
                            if !chat_msg.room.is_empty() {
                                let new_room = chat_msg.room.clone();
                                let mut sessions = app_state.sessions.lock().unwrap();
                                
                                if let Some(user_session) = sessions.get_mut(user_id) {
                                    // 从旧房间移除
                                    let old_room = user_session.room.clone();
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
                                        text: format!("{} 离开了房间", user_session.username),
                                        timestamp: chrono::Utc::now().timestamp() as u64,
                                        id: Uuid::new_v4().to_string(),
                                        target: None,
                                    };
                                    
                                    broadcast_message_to_room(&leave_msg, &old_room, app_state).await;
                                    
                                    // 加入新房间
                                    user_session.room = new_room.clone();
                                    {
                                        let mut rooms = app_state.rooms.lock().unwrap();
                                        rooms.entry(new_room.clone())
                                             .or_insert_with(HashSet::new)
                                             .insert(user_id.to_string());
                                    }
                                    
                                    // 发送加入消息到新房间
                                    let join_msg = ChatMessage {
                                        msg_type: "system".to_string(),
                                        username: "服务器".to_string(),
                                        room: new_room.clone(),
                                        text: format!("{} 加入了房间", user_session.username),
                                        timestamp: chrono::Utc::now().timestamp() as u64,
                                        id: Uuid::new_v4().to_string(),
                                        target: None,
                                    };
                                    
                                    broadcast_message_to_room(&join_msg, &new_room, app_state).await;
                                    
                                    // 更新两个房间的用户列表
                                    send_user_list(app_state, old_room).await;
                                    send_user_list(app_state, new_room).await;
                                }
                            }
                        },
                        "command" => {
                            // 处理命令消息
                            let response = handle_command(chat_msg.text, user_id, app_state).await;
                            
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
        send_user_list(app_state, room).await;
    }
    
    log::info!("Connection closed for {} ({})", user_id, username);
}

// 向指定房间广播消息
async fn broadcast_message_to_room(message: &ChatMessage, room: &str, app_state: &Arc<AppState>) {
    log::debug!("Broadcasting to room {}: {:?}", room, message);
    
    let user_ids = {
        let rooms = app_state.rooms.lock().unwrap();
        match rooms.get(room) {
            Some(user_set) => user_set.clone(),
            None => return,
        }
    };
    
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
            if let Err(e) = user_session.session.text(message_json.clone()).await {
                log::error!("Error sending message to {}: {:?}", user_id, e);
            }
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
async fn send_user_list(app_state: &Arc<AppState>, room: String) {
    let mut user_list = Vec::new();
    
    {
        let sessions = app_state.sessions.lock().unwrap();
        let rooms = app_state.rooms.lock().unwrap();
        
        if let Some(user_ids) = rooms.get(&room) {
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
        room: room.clone(),
        text: user_list.join(","),
        timestamp: chrono::Utc::now().timestamp() as u64,
        id: Uuid::new_v4().to_string(),
        target: None,
    };
    
    broadcast_message_to_room(&user_list_msg, &room, app_state).await;
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
            let start_time = Instant::now();
            let ping_msg = ChatMessage {
                msg_type: "ping".to_string(),
                username: "服务器".to_string(),
                room: "".to_string(),
                text: start_time.elapsed().as_micros().to_string(),
                timestamp: chrono::Utc::now().timestamp() as u64,
                id: Uuid::new_v4().to_string(),
                target: None,
            };
            
            send_message_to_user(&ping_msg, user_id, app_state).await;
            return "测量中...".to_string();
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
    Ok(fs::NamedFile::open("static/index.html")?)
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
            .service(web::resource("/").route(web::get().to(index)))
            .service(fs::Files::new("/static", "static"))
    })
    .bind("0.0.0.0:8080")?  // 监听所有接口，方便局域网内访问
    .run()
    .await
}