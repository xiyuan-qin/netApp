<template>
  <div class="side-panel">
    <!-- 房间列表 -->
    <div class="panel-section rooms-section">
      <h2>房间列表 <span class="count">({{ rooms.length }})</span></h2>
      <ul class="rooms-list">
        <li 
          v-for="room in rooms" 
          :key="room"
          :class="{ active: currentRoom === room && !privateTarget }"
          @click="joinRoom(room)"
        >
          # {{ room }}
        </li>
      </ul>
      
      <div class="new-room">
        <input 
          type="text" 
          v-model="newRoomName" 
          @keyup.enter="createRoom"
          placeholder="创建新房间"
        />
        <button @click="createRoom" :disabled="!isValidRoomName">创建</button>
      </div>
    </div>
    
    <!-- 用户列表 -->
    <div class="panel-section users-section">
      <h2>在线用户 <span class="count">({{ users.length }})</span></h2>
      <ul class="users-list">
        <li 
          v-for="user in users" 
          :key="user.username"
          :class="{ active: privateTarget === user.username }"
          @click="startPrivateChat(user.username)"
        >
          <div class="user-item">
            <span class="user-avatar">{{ getUserInitial(user.username) }}</span>
            <span class="user-name">{{ user.username }}</span>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>

<script>
import { ref, computed } from 'vue';

export default {
  name: 'SidePanel',
  props: {
    users: {
      type: Array,
      required: true
    },
    rooms: {
      type: Array,
      required: true
    },
    currentRoom: {
      type: String,
      required: true
    },
    privateTarget: {
      type: String,
      default: null
    }
  },
  emits: ['join-room', 'create-room', 'start-private-chat'],
  
  setup(props, { emit }) {
    const newRoomName = ref('');
    
    // 检查房间名是否有效
    const isValidRoomName = computed(() => {
      return newRoomName.value && newRoomName.value.trim().length >= 2;
    });
    
    // 获取用户名首字母作为头像
    const getUserInitial = (username) => {
      return username ? username.charAt(0).toUpperCase() : '?';
    };
    
    // 加入房间
    const joinRoom = (room) => {
      emit('join-room', room);
    };
    
    // 创建新房间
    const createRoom = () => {
      if (isValidRoomName.value) {
        emit('create-room', newRoomName.value.trim());
        newRoomName.value = '';
      }
    };
    
    // 开始私聊
    const startPrivateChat = (username) => {
      emit('start-private-chat', username);
    };
    
    return {
      newRoomName,
      isValidRoomName,
      getUserInitial,
      joinRoom,
      createRoom,
      startPrivateChat
    };
  }
};
</script>

<style scoped>
.side-panel {
  width: 260px;
  flex-shrink: 0;
  background-color: var(--light-bg);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.panel-section {
  padding: 15px;
}

.panel-section h2 {
  font-size: 1rem;
  margin-bottom: 10px;
  color: var(--text-color);
  display: flex;
  align-items: center;
}

.count {
  font-size: 0.8rem;
  font-weight: normal;
  color: var(--text-light);
  margin-left: 5px;
}

.rooms-section {
  border-bottom: 1px solid var(--border-color);
}

.rooms-list, .users-list {
  list-style: none;
  padding: 0;
  margin: 0;
  overflow-y: auto;
  max-height: 200px;
}

.rooms-list li, .users-list li {
  padding: 8px 12px;
  border-radius: 4px;
  margin-bottom: 2px;
  cursor: pointer;
  transition: background-color 0.3s;
  color: var(--text-color);
}

.rooms-list li:hover, .users-list li:hover {
  background-color: var(--hover-bg);
}

.rooms-list li.active, .users-list li.active {
  background-color: var(--primary-light);
  color: var(--primary-color);
  font-weight: 500;
}

.new-room {
  margin-top: 15px;
  display: flex;
  gap: 5px;
}

.new-room input {
  flex-grow: 1;
  padding: 8px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 0.9rem;
}

.new-room input:focus {
  outline: none;
  border-color: var(--primary-color);
}

.new-room button {
  background-color: var(--primary-color);
  color: white;
  border: none;
  border-radius: 4px;
  padding: 0 10px;
  cursor: pointer;
  transition: background-color 0.3s;
}

.new-room button:hover:not(:disabled) {
  background-color: var(--secondary-color);
}

.new-room button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.user-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.user-avatar {
  width: 24px;
  height: 24px;
  background-color: var(--primary-light);
  color: var(--primary-color);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  font-size: 0.8rem;
  font-weight: bold;
}

.users-section {
  flex-grow: 1;
  display: flex;
  flex-direction: column;
}

.users-list {
  flex-grow: 1;
  max-height: 100%;
}

@media (max-width: 768px) {
  .side-panel {
    width: 100%;
    border-right: none;
    border-bottom: 1px solid var(--border-color);
    max-height: 200px;
  }
  
  .panel-section {
    padding: 10px;
  }
  
  .rooms-section, .users-section {
    flex-basis: 50%;
  }
  
  .rooms-list, .users-list {
    max-height: 120px;
  }
}
</style>