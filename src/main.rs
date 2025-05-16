mod models;
mod handlers;
mod utils;

use actix_files as fs;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, middleware};
use actix_ws::Message;
use std::sync::Arc;
use models::{AppState, ChatMessage, UserSession};
use handlers::*;
use utils::*;
use uuid::Uuid;

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
    
    // 初始化用户会话
    let user_session = UserSession {
        id: id.clone(),
        username: default_username.clone(),
        room: "大厅".to_string(),
        addr: client_addr.clone(),
        session: session.clone(),
        last_heartbeat: std::time::Instant::now(),
        join_time: std::time::Instant::now(),
    };
    
    // 存储连接前先检查并清理可能存在的同IP陈旧连接
    {
        let mut sessions = app_state.sessions.lock().unwrap();
        let mut stale_sessions = Vec::new();
        
        // 检查是否有来自相同IP的陈旧连接
        for (session_id, existing_session) in sessions.iter() {
            if existing_session.addr == client_addr && 
               existing_session.last_heartbeat.elapsed() > std::time::Duration::from_secs(60) {
                stale_sessions.push(session_id.clone());
            }
        }
        
        // 移除陈旧连接
        for stale_id in &stale_sessions {
            log::info!("Removing stale connection: {} from same IP {}", stale_id, client_addr);
            if let Some(stale_session) = sessions.remove(stale_id) {
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
             .or_insert_with(std::collections::HashSet::new)
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
        let mut ping_interval = actix_web::rt::time::interval(std::time::Duration::from_secs(30));
        
        loop {
            tokio::select! {
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
                
                _ = ping_interval.tick() => {
                    let mut sessions = app_state_clone.sessions.lock().unwrap();
                    if let Some(user_session) = sessions.get_mut(&id_clone) {
                        if user_session.last_heartbeat.elapsed() > std::time::Duration::from_secs(90) {
                            log::info!("Client {} timed out", id_clone);
                            break;
                        }
                        
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
        
        log::info!("WebSocket handler loop exited for {}, cleaning up", id_clone);
        handle_disconnect(&id_clone, &app_state_clone).await;
    });
    
    Ok(response)
}

// 静态文件路由
async fn index() -> actix_web::Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("vue-client/dist/index.html")?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    log::info!("启动计算机网络实验服务器在 http://localhost:8080");
    
    let app_state = web::Data::new(Arc::new(AppState {
        sessions: std::sync::Mutex::new(std::collections::HashMap::new()),
        rooms: std::sync::Mutex::new({
            let mut rooms = std::collections::HashMap::new();
            rooms.insert("大厅".to_string(), std::collections::HashSet::new());
            rooms
        }),
    }));
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(middleware::Logger::default())
            .service(web::resource("/ws").route(web::get().to(ws_route)))
            .service(fs::Files::new("/", "vue-client/dist").index_file("index.html"))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}