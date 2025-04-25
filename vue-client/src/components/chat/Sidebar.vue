<template>
  <div class="sidebar">
    <div class="connection-info">
      <div class="connection-status">
        <span class="status-dot" :class="{ 
          'connected': connectionStatus === '已连接',
          'connecting': connectionStatus === '正在连接...',
          'disconnected': connectionStatus === '已断开' || connectionStatus === '连接错误'
        }"></span>
        <span class="status-text">{{ connectionStatus }}</span>
      </div>
      
      <div class="server-info">
        <div class="info-item">
          <span class="info-label">服务器:</span>
          <span class="info-value">{{ serverAddress || '--' }}</span>
        </div>
        <div class="info-item">
          <span class="info-label">客户端:</span>
          <span class="info-value">{{ clientAddress || '--' }}</span>
        </div>
      </div>
      
      <div class="network-info">
        <div class="info-item">
          <span class="info-label">延迟:</span>
          <span class="info-value">{{ networkLatency }}</span>
          <button class="ping-btn" @click="$emit('ping')">PING</button>
        </div>
        <div class="info-item">
          <span class="info-label">发送/接收:</span>
          <span class="info-value">{{ sentCount }}/{{ receivedCount }}</span>
        </div>
        <div class="info-item">
          <span class="info-label">平均延迟:</span>
          <span class="info-value">{{ averageLatency }}</span>
        </div>
      </div>
    </div>
    
    <div class="rooms-section">
      <div class="section-header">
        <h3>房间</h3>
        <button class="add-btn" @click="showCreateRoomDialog">+</button>
      </div>
      
      <ul class="room-list">
        <li v-for="room in rooms" 
            :key="room.name" 
            :class="{ active: room.active }" 
            @click="$emit('join-room', room.name)"
        >
          <span class="room-name">{{ room.name }}</span>
          <span class="room-count" v-if="room.userCount">{{ room.userCount }}</span>
        </li>
      </ul>
    </div>
    
    <div class="users-section">
      <div class="section-header">
        <h3>用户</h3>
      </div>
      
      <ul class="user-list">
        <li v-for="user in users" 
            :key="user.username" 
            :class="{ 'is-self': user.isSelf }"
            @click="startPrivateChat(user)"
        >
          <span class="user-name">{{ user.username }}</span>
          <span class="user-address" v-if="user.address">{{ user.address }}</span>
        </li>
      </ul>
    </div>
    
    <!-- 创建房间对话框 -->
    <div class="modal-overlay" v-if="isCreateRoomDialogVisible" @click="cancelCreateRoom">
      <div class="modal-content" @click.stop>
        <h3>创建或加入房间</h3>
        <input 
          ref="roomNameInput"
          type="text" 
          v-model="newRoomName" 
          placeholder="输入房间名称"
          @keyup.enter="createRoom"
        />
        <div class="modal-actions">
          <button @click="cancelCreateRoom">取消</button>
          <button class="primary" @click="createRoom">确定</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, nextTick } from 'vue'

export default {
  name: 'ChatSidebar',
  props: {
    connectionStatus: String,
    serverAddress: String,
    clientAddress: String,
    networkLatency: String,
    sentCount: Number,
    receivedCount: Number,
    averageLatency: String,
    rooms: Array,
    users: Array,
    currentRoom: String
  },
  
  setup(props, { emit }) {
    // 创建房间对话框状态
    const isCreateRoomDialogVisible = ref(false)
    const newRoomName = ref('')
    const roomNameInput = ref(null)
    
    const showCreateRoomDialog = () => {
      isCreateRoomDialogVisible.value = true
      newRoomName.value = ''
      
      // 对话框显示后聚焦输入框
      nextTick(() => {
        roomNameInput.value.focus()
      })
    }
    
    const cancelCreateRoom = () => {
      isCreateRoomDialogVisible.value = false
    }
    
    const createRoom = () => {
      if (newRoomName.value.trim()) {
        emit('create-room', newRoomName.value.trim())
        isCreateRoomDialogVisible.value = false
      }
    }
    
    const startPrivateChat = (user) => {
      if (!user.isSelf) {
        emit('start-private-chat', user.username)
      }
    }
    
    return {
      isCreateRoomDialogVisible,
      newRoomName,
      roomNameInput,
      showCreateRoomDialog,
      cancelCreateRoom,
      createRoom,
      startPrivateChat
    }
  }
}
</script>

<style scoped>
.sidebar {
  width: 280px;
  min-width: 280px;
  background: linear-gradient(to bottom, #ffffff, #f8fafc);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow-y: auto;
  box-shadow: 2px 0 10px rgba(0, 0, 0, 0.03);
}

.connection-info {
  padding: 20px;
  border-bottom: 1px solid var(--border-color);
  background: linear-gradient(135deg, rgba(62, 106, 225, 0.05), rgba(138, 86, 172, 0.05));
}

.connection-status {
  display: flex;
  align-items: center;
  margin-bottom: 15px;
}

.status-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  margin-right: 10px;
  background-color: #ccc;
  position: relative;
  box-shadow: 0 0 5px rgba(0, 0, 0, 0.1);
}

.status-dot.connected {
  background-color: var(--success-color);
}

.status-dot.connected::after {
  content: '';
  position: absolute;
  top: -4px;
  left: -4px;
  right: -4px;
  bottom: -4px;
  border-radius: 50%;
  border: 2px solid rgba(76, 175, 80, 0.3);
  animation: pulse 1.5s infinite;
}

.status-dot.connecting {
  background-color: var(--warning-color);
  animation: blink 1s infinite;
}

.status-dot.disconnected {
  background-color: var(--error-color);
}

@keyframes pulse {
  0% { transform: scale(1); opacity: 1; }
  70% { transform: scale(1.3); opacity: 0; }
  100% { transform: scale(1); opacity: 0; }
}

@keyframes blink {
  0% { opacity: 0.5; }
  50% { opacity: 1; }
  100% { opacity: 0.5; }
}

.status-text {
  font-weight: 600;
  font-size: 1rem;
  color: var(--text-color);
}

.server-info, .network-info {
  margin-bottom: 15px;
  background-color: rgba(255, 255, 255, 0.7);
  border-radius: var(--radius-md);
  padding: 10px 12px;
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.03);
}

.info-item {
  display: flex;
  margin: 8px 0;
  font-size: 0.88rem;
  align-items: center;
}

.info-label {
  width: 80px;
  color: var(--text-light);
  font-weight: 500;
}

.info-value {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: var(--font-mono);
  font-size: 0.85rem;
  color: var(--text-color);
}

.ping-btn {
  background: linear-gradient(135deg, var(--primary-color), var(--secondary-color));
  color: white;
  border: none;
  border-radius: var(--radius-sm);
  padding: 3px 10px;
  font-size: 0.7rem;
  font-weight: 600;
  cursor: pointer;
  margin-left: 8px;
  transition: var(--transition);
  box-shadow: 0 2px 5px rgba(62, 106, 225, 0.2);
}

.ping-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 3px 8px rgba(62, 106, 225, 0.3);
}

/* 房间和用户列表标题 */
.section-header {
  padding: 12px 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid var(--border-color);
  background: linear-gradient(to right, rgba(62, 106, 225, 0.05), rgba(138, 86, 172, 0.05));
}

.section-header h3 {
  font-size: 1rem;
  font-weight: 600;
  color: var(--primary-color);
  margin: 0;
  display: flex;
  align-items: center;
}

/* 图标 */
.section-header h3::before {
  margin-right: 8px;
  font-size: 1.1rem;
}

.rooms-section .section-header h3::before {
  content: '🏠';
}

.users-section .section-header h3::before {
  content: '👥';
}

.add-btn {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--primary-color), var(--secondary-color));
  color: white;
  border: none;
  font-size: 18px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: var(--transition);
  box-shadow: 0 2px 5px rgba(62, 106, 225, 0.2);
}

.add-btn:hover {
  transform: rotate(90deg) scale(1.1);
  box-shadow: 0 3px 8px rgba(62, 106, 225, 0.3);
}

.room-list, .user-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.room-list li, .user-list li {
  padding: 10px 20px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.03);
  cursor: pointer;
  display: flex;
  justify-content: space-between;
  align-items: center;
  transition: var(--transition);
}

.room-list li:hover, .user-list li:hover {
  background-color: rgba(62, 106, 225, 0.05);
}

.room-list li.active {
  background-color: rgba(62, 106, 225, 0.1);
  border-left: 3px solid var(--primary-color);
  font-weight: 600;
}

.room-name {
  display: flex;
  align-items: center;
}

.room-name::before {
  content: '#';
  margin-right: 5px;
  color: var(--text-light);
  font-weight: normal;
}

.room-count {
  background: linear-gradient(135deg, var(--primary-color), var(--secondary-color));
  color: white;
  border-radius: 12px;
  padding: 2px 8px;
  font-size: 0.75rem;
  font-weight: 600;
}

.user-list li {
  position: relative;
}

.user-list li:hover::after {
  content: '私聊';
  position: absolute;
  right: 15px;
  font-size: 0.75rem;
  background-color: rgba(138, 86, 172, 0.1);
  color: var(--secondary-color);
  padding: 2px 8px;
  border-radius: 10px;
}

.user-list li.is-self {
  color: var(--primary-color);
  font-weight: 600;
}

.user-list li.is-self:hover::after {
  content: '(你自己)';
  background: none;
  color: var(--text-light);
}

.user-name {
  display: flex;
  align-items: center;
}

.user-address {
  font-size: 0.75rem;
  color: var(--text-light);
  margin-left: 5px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100px;
}

.rooms-section, .users-section {
  display: flex;
  flex-direction: column;
  border-bottom: 1px solid var(--border-color);
}

.rooms-section {
  flex: 0 0 auto;
}

.users-section {
  flex: 1;
}

/* 创建房间对话框 */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(3px);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.modal-content {
  background-color: white;
  padding: 25px;
  border-radius: var(--radius-lg);
  width: 90%;
  max-width: 320px;
  box-shadow: var(--shadow-lg);
  animation: scaleIn 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

@keyframes scaleIn {
  from { transform: scale(0.8); opacity: 0; }
  to { transform: scale(1); opacity: 1; }
}

.modal-content h3 {
  margin: 0 0 20px 0;
  color: var(--text-color);
  font-size: 1.2rem;
  text-align: center;
}

.modal-content input {
  width: 100%;
  padding: 12px 15px;
  border: 2px solid var(--border-color);
  border-radius: var(--radius-md);
  margin-bottom: 20px;
  font-size: 0.95rem;
  transition: var(--transition);
  box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.05);
}

.modal-content input:focus {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px rgba(62, 106, 225, 0.15);
  outline: none;
}

.modal-actions {
  display: flex;
  justify-content: center;
  gap: 12px;
}

.modal-actions button {
  flex: 1;
  padding: 10px 16px;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 600;
  transition: var(--transition);
}

.modal-actions button:not(.primary) {
  background-color: #f1f5f9;
  color: var(--text-color);
}

.modal-actions button:not(.primary):hover {
  background-color: #e2e8f0;
}

.modal-actions button.primary {
  background: linear-gradient(135deg, var(--primary-color), var(--secondary-color));
  color: white;
  box-shadow: 0 2px 8px rgba(62, 106, 225, 0.2);
}

.modal-actions button.primary:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(62, 106, 225, 0.3);
}

@media (max-width: 1000px) {
  .sidebar {
    width: 100%;
    min-width: auto;
    height: auto;
    max-height: 35vh;
    border-right: none;
    border-bottom: 1px solid var(--border-color);
  }
  
  .connection-info {
    padding: 10px 15px;
  }
  
  .server-info, .network-info {
    margin-bottom: 8px;
  }
}
</style>