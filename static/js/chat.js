document.addEventListener('DOMContentLoaded', () => {
    // DOM 元素
    const chatBox = document.getElementById('chat-box');
    const messageInput = document.getElementById('message-input');
    const sendButton = document.getElementById('send-button');
    const usernameInput = document.getElementById('username');
    const connectionStatus = document.getElementById('connection-status');
    const serverAddress = document.getElementById('server-address');
    const clientAddress = document.getElementById('client-address');
    const networkLatency = document.getElementById('network-latency');
    const pingButton = document.getElementById('ping-button');
    const sentCount = document.getElementById('sent-count');
    const receivedCount = document.getElementById('received-count');
    const averageLatency = document.getElementById('average-latency');
    const networkLog = document.getElementById('network-log');
    const roomList = document.getElementById('room-list');
    const userList = document.getElementById('user-list');
    const newRoomInput = document.getElementById('new-room');
    const createRoomBtn = document.getElementById('create-room-btn');
    const currentRoomLabel = document.getElementById('current-room');
    
    // 状态变量
    let socket = null;
    let currentRoom = '大厅';
    let pingStartTime = 0;
    let pingCount = 0;
    let totalLatency = 0;
    let messageIdMap = new Map(); // 用于消息确认机制
    let sentMessages = 0;
    let receivedMessages = 0;
    let messagesQueue = []; // 待确认消息队列
    let currentPrivateTarget = null; // 当前私聊对象
    
    // 初始化 - 添加调试消息
    function init() {
        console.log('初始化应用...');
        // 添加一条欢迎消息，测试消息显示功能
        displaySystemMessage('欢迎使用网络实验聊天应用！');
        
        // 确保聊天框可见
        if (chatBox) {
            chatBox.style.display = 'flex';
            console.log('聊天框初始化完成');
        } else {
            console.error('错误: 未找到聊天框元素!');
        }
        
        connect();
        addEventListeners();
    }
    
    // 添加事件监听
    function addEventListeners() {
        console.log('添加事件监听...');
        
        sendButton.addEventListener('click', sendMessage);
        
        messageInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                sendMessage();
            }
        });
        
        pingButton.addEventListener('click', () => {
            sendPing();
        });
        
        createRoomBtn.addEventListener('click', () => {
            const roomName = newRoomInput.value.trim();
            if (roomName) {
                joinRoom(roomName);
                newRoomInput.value = '';
            }
        });
        
        // 委托事件监听，处理房间点击
        roomList.addEventListener('click', (e) => {
            if (e.target.classList.contains('room')) {
                const room = e.target.dataset.room;
                joinRoom(room);
                // 退出私聊模式
                exitPrivateMode();
            }
        });
        
        // 用户列表点击事件 - 私聊功能
        userList.addEventListener('click', (e) => {
            const userElement = e.target.closest('.user');
            if (userElement) {
                const username = userElement.dataset.username;
                if (username && username !== usernameInput.value) {
                    startPrivateChat(username);
                }
            }
        });
    }
    
    // 进入私聊模式
    function startPrivateChat(username) {
        currentPrivateTarget = username;
        currentRoomLabel.innerHTML = `私聊: <span class="private-target">${username}</span> <button id="exit-private" class="exit-private-btn">返回房间</button>`;
        
        // 添加退出私聊按钮事件
        document.getElementById('exit-private').addEventListener('click', exitPrivateMode);
        
        // 显示进入私聊模式提示
        displaySystemMessage(`已进入与 ${username} 的私聊模式，消息仅对方可见`);
    }
    
    // 退出私聊模式
    function exitPrivateMode() {
        if (currentPrivateTarget) {
            currentPrivateTarget = null;
            currentRoomLabel.textContent = currentRoom;
            displaySystemMessage('已退出私聊模式，回到房间聊天');
        }
    }
    
    // 连接WebSocket服务器
    function connect() {
        // 检查当前协议是http还是https
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws`;
        
        updateConnectionStatus('正在连接...', 'orange');
        logNetwork('连接', `正在连接到 ${wsUrl}`, 'info');
        
        socket = new WebSocket(wsUrl);
        
        socket.onopen = () => {
            updateConnectionStatus('已连接', 'green');
            serverAddress.textContent = window.location.host;
            logNetwork('连接', '连接已建立', 'info');
            
            // 发送初始消息以设置用户名
            const username = usernameInput.value.trim() || '用户';
            sendChatMessage('chat', username, currentRoom, '');
            
            // 连接成功后显示系统消息
            displaySystemMessage('已成功连接到服务器');
        };
        
        socket.onmessage = handleMessage;
        
        socket.onclose = (event) => {
            updateConnectionStatus('已断开', 'red');
            logNetwork('断开', `连接已关闭: ${event.code} ${event.reason}`, 'error');
            
            // 5秒后重连
            setTimeout(connect, 5000);
        };
        
        socket.onerror = (error) => {
            updateConnectionStatus('连接错误', 'red');
            logNetwork('错误', '连接错误', 'error');
            console.error('WebSocket错误:', error);
        };
    }
    
    // 处理接收到的消息
    function handleMessage(event) {
        receivedMessages++;
        updateStats();
        
        try {
            const message = JSON.parse(event.data);
            logNetwork('接收', `${message.msg_type}: ${message.text ? (message.text.substring(0, 30) + (message.text.length > 30 ? '...' : '')) : '空消息'}`, 'received');
            
            // 调试输出 - 帮助诊断问题
            console.log('收到消息:', message);
            
            switch (message.msg_type) {
                case 'chat':
                    // 确保文本不为空才显示消息
                    if (message.text && message.text.trim() !== '') {
                        console.log('显示聊天消息:', message.text);
                        displayMessage(message.username, message.text, message.username === usernameInput.value, message.timestamp);
                    }
                    break;
                
                case 'system':
                    console.log('显示系统消息:', message.text);
                    displaySystemMessage(message.text);
                    
                    // 提取服务器信息中的客户端IP
                    if (message.text && message.text.includes('您的IP地址')) {
                        const ipMatch = message.text.match(/您的IP地址: ([^,]+)/);
                        if (ipMatch && ipMatch[1]) {
                            clientAddress.textContent = ipMatch[1];
                        }
                    }
                    break;
                
                case 'private':
                    // 处理私聊消息
                    if (!currentPrivateTarget && message.username !== usernameInput.value) {
                        // 如果不在私聊模式且收到对方发来的消息，自动进入私聊模式
                        startPrivateChat(message.username);
                    }
                    
                    // 显示私聊消息
                    if (message.text && message.text.trim() !== '') {
                        console.log('显示私聊消息:', message.text);
                        displayPrivateMessage(message.username, message.text, 
                            message.username === usernameInput.value, 
                            message.timestamp, 
                            message.target);
                    }
                    break;
                
                case 'userlist':
                    updateUserList(message.text);
                    break;
                
                case 'ping':
                    // 接收到服务器ping，回复pong
                    sendChatMessage('pong', usernameInput.value, currentRoom, message.text);
                    
                    // 如果是我们的ping请求，计算延迟
                    if (pingStartTime > 0) {
                        const latency = Date.now() - pingStartTime;
                        networkLatency.textContent = `${latency}ms`;
                        
                        // 更新平均延迟
                        pingCount++;
                        totalLatency += latency;
                        averageLatency.textContent = `${Math.round(totalLatency / pingCount)}ms`;
                        
                        pingStartTime = 0;
                    }
                    break;
                
                case 'pong':
                    // 收到服务器的pong响应，计算延迟
                    if (pingStartTime > 0) {
                        const latency = Date.now() - pingStartTime;
                        networkLatency.textContent = `${latency}ms`;
                        
                        // 更新平均延迟
                        pingCount++;
                        totalLatency += latency;
                        averageLatency.textContent = `${Math.round(totalLatency / pingCount)}ms`;
                        
                        pingStartTime = 0;
                    }
                    break;
            }
            
        } catch (e) {
            console.error('无法解析消息:', e, event.data);
            logNetwork('错误', '消息解析失败: ' + e.message, 'error');
        }
    }
    
    // 发送聊天消息
    function sendMessage() {
        const text = messageInput.value.trim();
        if (!text) return;
        
        const username = usernameInput.value.trim() || '匿名';
        
        // 添加本地显示
        if (!text.startsWith('/')) {
            // 如果是普通消息，先在本地显示，提高响应速度感
            if (currentPrivateTarget) {
                displayPrivateMessage(username, text, true, Date.now(), currentPrivateTarget);
            } else {
                displayMessage(username, text, true, Date.now());
            }
        }
        
        // 调试输出
        console.log('发送消息:', text);
        
        // 检查是否为命令
        if (text.startsWith('/')) {
            if (text === '/help') {
                // 客户端处理help命令
                displaySystemMessage(`可用命令:
                /help - 显示帮助
                /rooms - 显示所有房间
                /join <房间名> - 加入指定房间
                /users - 显示当前房间用户
                /msg <用户名> <消息> - 发送私聊消息
                /ping - 测试网络连接
                /stats - 显示网络统计信息`);
            } else if (text.startsWith('/join ')) {
                // 解析房间名
                const roomName = text.substring(6).trim();
                if (roomName) {
                    joinRoom(roomName);
                }
            } else if (text.startsWith('/msg ')) {
                // 解析私聊消息
                const parts = text.split(' ');
                if (parts.length >= 3) {
                    const targetUser = parts[1];
                    const msgText = parts.slice(2).join(' ');
                    sendPrivateMessage(targetUser, msgText);
                } else {
                    displaySystemMessage('用法: /msg <用户名> <消息内容>');
                }
            } else {
                // 其他命令发送到服务器处理
                sendChatMessage('command', username, currentRoom, text);
            }
        } else {
            // 普通消息
            if (currentPrivateTarget) {
                // 在私聊模式
                sendPrivateMessage(currentPrivateTarget, text);
            } else {
                // 房间消息
                sendChatMessage('chat', username, currentRoom, text);
            }
        }
        
        messageInput.value = '';
    }
    
    // 发送私聊消息
    function sendPrivateMessage(targetUser, text) {
        const username = usernameInput.value.trim() || '匿名';
        sendChatMessage('private', username, '私聊', text, targetUser);
    }
    
    // 发送ping请求测试延迟
    function sendPing() {
        pingStartTime = Date.now();
        sendChatMessage('command', usernameInput.value, currentRoom, '/ping');
        networkLatency.textContent = '测量中...';
    }
    
    // 加入房间
    function joinRoom(roomName) {
        if (roomName === currentRoom) return;
        
        sendChatMessage('join', usernameInput.value, roomName, '');
        displaySystemMessage(`正在加入房间: ${roomName}`);
        currentRoom = roomName;
        currentRoomLabel.textContent = roomName;
        
        // 更新UI
        const roomElements = document.querySelectorAll('.room');
        roomElements.forEach(el => el.classList.remove('active'));
        
        // 检查房间是否存在于列表中，如果不存在则创建
        let roomElement = Array.from(roomElements).find(el => el.dataset.room === roomName);
        if (!roomElement) {
            roomElement = document.createElement('div');
            roomElement.className = 'room';
            roomElement.dataset.room = roomName;
            roomElement.textContent = roomName;
            roomList.appendChild(roomElement);
        }
        
        roomElement.classList.add('active');
    }
    
    // 发送消息基础函数
    function sendChatMessage(type, username, room, text, target = null) {
        if (!socket || socket.readyState !== WebSocket.OPEN) {
            displaySystemMessage('未连接到服务器，无法发送消息');
            return;
        }
        
        const messageId = generateId();
        const message = {
            msg_type: type,
            username: username,
            room: room,
            text: text,
            timestamp: Date.now(),
            id: messageId,
            target: target
        };
        
        try {
            // 调试输出
            console.log('发送JSON消息:', JSON.stringify(message));
            
            socket.send(JSON.stringify(message));
            sentMessages++;
            updateStats();
            
            if ((type === 'chat' || type === 'private') && text) {
                // 为普通聊天消息添加确认机制
                messageIdMap.set(messageId, message);
                messagesQueue.push(messageId);
                
                // 60秒后如果未确认，标记为可能发送失败
                setTimeout(() => {
                    if (messageIdMap.has(messageId)) {
                        messageIdMap.delete(messageId);
                        displaySystemMessage('消息可能未送达: ' + text.substring(0, 20) + '...');
                    }
                }, 60000);
            }
            
            logNetwork('发送', `${type}: ${text.substring(0, 30)}${text.length > 30 ? '...' : ''}`, 'sent');
        } catch (e) {
            console.error('发送消息失败:', e);
            logNetwork('错误', '发送失败: ' + e.message, 'error');
        }
    }
    
    // 显示消息
    function displayMessage(username, text, isSelf, timestamp) {
        console.log(`显示消息: ${username}: ${text}`);
        
        const messageElement = document.createElement('div');
        messageElement.className = `message ${isSelf ? 'message-self' : 'message-other'}`;
        
        const usernameElement = document.createElement('div');
        usernameElement.className = 'username';
        usernameElement.textContent = username;
        
        const textElement = document.createElement('div');
        textElement.className = 'text';
        textElement.textContent = text;
        
        // 添加时间戳
        const timeElement = document.createElement('div');
        timeElement.className = 'timestamp';
        timeElement.textContent = formatTime(timestamp);
        
        messageElement.appendChild(usernameElement);
        messageElement.appendChild(textElement);
        messageElement.appendChild(timeElement);
        
        chatBox.appendChild(messageElement);
        scrollToBottom();
    }
    
    // 显示私聊消息
    function displayPrivateMessage(username, text, isSelf, timestamp, target) {
        console.log(`显示私聊消息: ${username} -> ${target}: ${text}`);
        
        const messageElement = document.createElement('div');
        messageElement.className = `message message-private ${isSelf ? 'message-self' : 'message-other'}`;
        
        const usernameElement = document.createElement('div');
        usernameElement.className = 'username';
        
        // 显示这是私聊消息
        if (isSelf) {
            usernameElement.textContent = `${username} → ${target}`;
        } else {
            usernameElement.textContent = `${username} → 你`;
        }
        
        const textElement = document.createElement('div');
        textElement.className = 'text';
        textElement.textContent = text;
        
        // 添加时间戳
        const timeElement = document.createElement('div');
        timeElement.className = 'timestamp';
        timeElement.textContent = formatTime(timestamp);
        
        messageElement.appendChild(usernameElement);
        messageElement.appendChild(textElement);
        messageElement.appendChild(timeElement);
        
        chatBox.appendChild(messageElement);
        scrollToBottom();
    }
    
    // 显示系统消息
    function displaySystemMessage(text) {
        console.log(`显示系统消息: ${text}`);
        
        const messageElement = document.createElement('div');
        messageElement.className = 'message message-system';
        messageElement.textContent = text;
        
        chatBox.appendChild(messageElement);
        scrollToBottom();
    }
    
    // 更新用户列表
    function updateUserList(userListText) {
        const users = userListText.split(',');
        userList.innerHTML = '';
        
        users.forEach(userInfo => {
            if (!userInfo) return;
            
            const [username, address] = userInfo.split(':');
            
            const userElement = document.createElement('div');
            userElement.className = 'user';
            userElement.dataset.username = username;
            
            // 判断是否是自己
            if (username === usernameInput.value) {
                userElement.innerHTML = `<strong>${username}</strong> <span class="user-address">${address || ''}</span> (我)`;
                userElement.classList.add('self');
            } else {
                userElement.innerHTML = `<strong>${username}</strong> <span class="user-address">${address || ''}</span> <span class="private-chat-btn" title="发送私聊">私聊</span>`;
            }
            
            userList.appendChild(userElement);
        });
    }
    
    // 更新连接状态
    function updateConnectionStatus(status, color) {
        connectionStatus.textContent = status;
        connectionStatus.style.color = color;
    }
    
    // 更新统计信息
    function updateStats() {
        sentCount.textContent = sentMessages;
        receivedCount.textContent = receivedMessages;
    }
    
    // 日志网络事件到网络监视器
    function logNetwork(type, message, className) {
        const logEntry = document.createElement('div');
        logEntry.className = `log-entry ${className}`;
        
        const time = document.createElement('span');
        time.className = 'time';
        time.textContent = new Date().toLocaleTimeString();
        
        logEntry.appendChild(time);
        logEntry.appendChild(document.createTextNode(`[${type}] ${message}`));
        
        networkLog.appendChild(logEntry);
        
        // 限制日志条目数量
        if (networkLog.childElementCount > 100) {
            networkLog.removeChild(networkLog.firstChild);
        }
        
        // 滚动到底部
        networkLog.scrollTop = networkLog.scrollHeight;
    }
    
    // 生成唯一ID
    function generateId() {
        return Date.now().toString(36) + Math.random().toString(36).substring(2);
    }
    
    // 滚动聊天框到底部
    function scrollToBottom() {
        chatBox.scrollTop = chatBox.scrollHeight;
    }
    
    // 格式化时间
    function formatTime(timestamp) {
        const date = new Date(timestamp);
        return date.toLocaleTimeString();
    }
    
    // 启动应用
    init();
    
    // 添加一个测试按钮，用于手动触发显示测试消息
    window.testMessage = function() {
        displayMessage('系统', '这是一条测试消息', false, Date.now());
        displaySystemMessage('测试系统消息显示');
    };
});