import { createStore } from 'vuex'

export default createStore({
  state: {
    // 网络连接相关状态
    connected: false,
    socket: null,
    serverInfo: {
      ping: 0,
      status: '未连接',
      address: ''
    },
    // 用户相关状态
    username: '',
    currentRoom: '大厅',
    privateTarget: null,
    // 消息和房间相关
    messages: [],
    rooms: ['大厅'],
    users: [],
    // 网络日志
    networkLogs: []
  },
  
  getters: {
    isConnected: state => state.connected,
    currentUsername: state => state.username,
    currentRoom: state => state.currentRoom,
    usersList: state => state.users,
    roomsList: state => state.rooms,
    messagesList: state => state.messages,
    networkLogs: state => state.networkLogs,
    serverInfo: state => state.serverInfo,
    isInPrivateChat: state => !!state.privateTarget,
    privateTarget: state => state.privateTarget
  },
  
  mutations: {
    // 连接状态
    setConnected(state, status) {
      state.connected = status;
    },
    setSocket(state, socket) {
      state.socket = socket;
    },
    // 用户信息
    setUsername(state, name) {
      state.username = name;
    },
    setCurrentRoom(state, room) {
      state.currentRoom = room;
    },
    setPrivateTarget(state, target) {
      state.privateTarget = target;
    },
    // 消息处理
    addMessage(state, message) {
      state.messages.push(message);
      // 限制消息历史记录数量，避免内存溢出
      if (state.messages.length > 200) {
        state.messages.shift();
      }
    },
    clearMessages(state) {
      state.messages = [];
    },
    // 房间处理
    setRooms(state, rooms) {
      state.rooms = rooms;
    },
    addRoom(state, room) {
      if (!state.rooms.includes(room)) {
        state.rooms.push(room);
      }
    },
    // 用户列表
    setUsers(state, users) {
      state.users = users;
    },
    // 网络日志
    addNetworkLog(state, log) {
      state.networkLogs.push({
        id: Date.now(),
        time: new Date().toLocaleTimeString(),
        ...log
      });
      // 限制日志数量
      if (state.networkLogs.length > 100) {
        state.networkLogs.shift();
      }
    },
    // 服务器信息
    updateServerInfo(state, info) {
      state.serverInfo = {...state.serverInfo, ...info};
    },
    updatePing(state, ping) {
      state.serverInfo.ping = ping;
    }
  },
  
  actions: {
    // WebSocket连接管理
    initConnection({commit, dispatch, state}) {
      // 确保不重复连接
      if (state.socket && state.connected) {
        return;
      }
      
      // 创建WebSocket连接
      const socket = new WebSocket(`ws://${window.location.host}/ws`);
      
      socket.onopen = () => {
        commit('setConnected', true);
        commit('updateServerInfo', { status: '已连接' });
        commit('addNetworkLog', { type: 'info', message: '已连接到服务器' });
        
        // 如果有用户名，发送初始化消息
        if (state.username) {
          dispatch('sendInitMessage');
        }
      };
      
      socket.onclose = () => {
        commit('setConnected', false);
        commit('updateServerInfo', { status: '已断开' });
        commit('addNetworkLog', { type: 'error', message: '连接已关闭' });
        
        // 尝试重新连接
        setTimeout(() => {
          dispatch('initConnection');
        }, 3000);
      };
      
      socket.onerror = (error) => {
        commit('addNetworkLog', { type: 'error', message: '连接错误: ' + error });
      };
      
      socket.onmessage = (event) => {
        dispatch('handleMessage', event.data);
      };
      
      commit('setSocket', socket);
    },
    
    // 发送初始化消息（用户名和房间）
    sendInitMessage({commit, state}) {
      if (!state.connected) return;
      
      const message = {
        msg_type: 'chat',
        username: state.username,
        room: state.currentRoom,
        text: '',
        timestamp: Date.now(),
        id: crypto.randomUUID()
      };
      
      state.socket.send(JSON.stringify(message));
      commit('addNetworkLog', { type: 'sent', message: '发送初始化消息' });
    },
    
    // 发送消息
    sendMessage({commit, state}, text) {
      if (!state.connected) return;
      
      const msgType = state.privateTarget ? 'private' : 'chat';
      const messageObj = {
        msg_type: msgType,
        username: state.username,
        room: state.currentRoom,
        text: text,
        timestamp: Date.now(),
        id: crypto.randomUUID()
      };
      
      // 添加私聊目标
      if (state.privateTarget) {
        messageObj.target = state.privateTarget;
      }
      
      // 添加本地消息（乐观更新）
      commit('addMessage', {
        id: messageObj.id,
        type: msgType,
        username: state.username,
        text: text,
        timestamp: messageObj.timestamp,
        isSelf: true,
        room: state.currentRoom,
        target: state.privateTarget
      });
      
      state.socket.send(JSON.stringify(messageObj));
      commit('addNetworkLog', { 
        type: 'sent', 
        message: `发送${state.privateTarget ? '私聊' : ''}消息: ${text.substring(0, 30)}${text.length > 30 ? '...' : ''}` 
      });
    },
    
    // 加入房间
    joinRoom({commit, state}, room) {
      if (!state.connected || !room) return;
      
      const message = {
        msg_type: 'join',
        username: state.username,
        room: room,
        text: '',
        timestamp: Date.now(),
        id: crypto.randomUUID()
      };
      
      state.socket.send(JSON.stringify(message));
      commit('setCurrentRoom', room);
      commit('addNetworkLog', { type: 'info', message: `加入房间: ${room}` });
    },
    
    // 创建新房间
    createRoom({dispatch}, room) {
      dispatch('joinRoom', room);
    },
    
    // 发送Ping测试
    sendPing({commit, state}) {
      if (!state.connected) return;
      
      const startTime = Date.now();
      const pingMessage = {
        msg_type: 'command',
        username: state.username,
        room: state.currentRoom,
        text: '/ping',
        timestamp: startTime,
        id: crypto.randomUUID()
      };
      
      state.socket.send(JSON.stringify(pingMessage));
      commit('addNetworkLog', { type: 'info', message: '发送Ping请求' });
    },
    
    // 开始私聊
    startPrivateChat({commit}, username) {
      commit('setPrivateTarget', username);
    },
    
    // 结束私聊
    endPrivateChat({commit}) {
      commit('setPrivateTarget', null);
    },
    
    // 处理收到的消息
    handleMessage({commit}, data) {
      try {
        const message = JSON.parse(data);
        
        switch(message.msg_type) {
          case 'chat':
            commit('addMessage', {
              id: message.id,
              type: 'chat',
              username: message.username,
              text: message.text,
              timestamp: message.timestamp,
              isSelf: message.username === this.state.username,
              room: message.room
            });
            commit('addNetworkLog', { type: 'received', message: `收到消息: ${message.text.substring(0, 30)}${message.text.length > 30 ? '...' : ''}` });
            break;
            
          case 'system':
            commit('addMessage', {
              id: message.id,
              type: 'system',
              text: message.text,
              timestamp: message.timestamp,
              room: message.room
            });
            commit('addNetworkLog', { type: 'info', message: `系统消息: ${message.text.substring(0, 30)}${message.text.length > 30 ? '...' : ''}` });
            break;
            
          case 'private':
            commit('addMessage', {
              id: message.id,
              type: 'private',
              username: message.username, 
              text: message.text,
              timestamp: message.timestamp,
              isSelf: message.username === this.state.username,
              target: message.target
            });
            commit('addNetworkLog', { 
              type: 'received', 
              message: `私聊消息(${message.username === this.state.username ? '发送给 ' + message.target : '来自 ' + message.username}): ${message.text.substring(0, 30)}${message.text.length > 30 ? '...' : ''}` 
            });
            break;
            
          case 'userlist': {
            const users = message.text.split(',')
              .filter(u => u.trim() !== '')
              .map(u => {
                const [username, address] = u.split(':');
                return { username, address };
              });
            commit('setUsers', users);
            commit('addNetworkLog', { type: 'info', message: `更新用户列表: ${users.length}个用户` });
            break;
          }
            
          case 'ping': {
            const ping = Date.now() - parseInt(message.timestamp);
            commit('updatePing', ping);
            
            // 回复pong
            if (this.state.socket) {
              const pongMessage = {
                msg_type: 'pong',
                username: this.state.username,
                room: this.state.currentRoom,
                text: '',
                timestamp: Date.now(),
                id: crypto.randomUUID()
              };
              this.state.socket.send(JSON.stringify(pongMessage));
            }
            break;
          }
            
          default:
            commit('addNetworkLog', { type: 'info', message: `收到未知类型消息: ${message.msg_type}` });
        }
      } catch (e) {
        commit('addNetworkLog', { type: 'error', message: `消息解析错误: ${e.message}` });
      }
    }
  }
})