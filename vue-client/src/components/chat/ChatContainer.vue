<template>
  <div class="chat-container">
    <div class="chat-header">
      <div class="room-info">
        <h2>
          <template v-if="currentPrivateTarget">
            私聊: {{ currentPrivateTarget }}
            <button class="exit-private-btn" @click="$emit('exit-private-mode')">退出</button>
          </template>
          <template v-else>
            房间: {{ currentRoom }}
          </template>
        </h2>
      </div>
    </div>
    
    <div class="messages-container" ref="messagesContainer">
      <div v-if="messages.length === 0" class="no-messages">
        暂无消息，开始聊天吧！
      </div>
      
      <div class="messages-list">
        <template v-for="(message, index) in messages" :key="index">
          <!-- 系统消息 -->
          <div v-if="message.type === 'system'" class="message-item system-message">
            <div class="system-content">{{ message.text }}</div>
          </div>
          
          <!-- 自己发送的消息，右侧显示 -->
          <div v-else-if="isMyMessage(message)" class="message-item right-message">
            <div class="message-avatar self-avatar">
              {{ getInitial(username) }}
            </div>
            <div class="message-content self-message">
              <div class="message-text">{{ message.text }}</div>
              <div class="message-time">{{ formatTime(message.timestamp) }}</div>
            </div>
          </div>
          
          <!-- 他人发送的消息，左侧显示 -->
          <div v-else class="message-item left-message">
            <div class="message-avatar other-avatar">
              {{ getInitial(message.username) }}
            </div>
            <div class="message-content other-message">
              <div class="sender-name">{{ message.username }}</div>
              <div class="message-text">{{ message.text }}</div>
              <div class="message-time">{{ formatTime(message.timestamp) }}</div>
            </div>
          </div>
        </template>
      </div>
    </div>
    
    <div class="input-container">
      <textarea 
        ref="messageInput"
        v-model="inputMessage" 
        @keydown.enter.prevent="sendMessage"
        :placeholder="currentPrivateTarget ? `发送给 ${currentPrivateTarget} 的私聊消息...` : '发送消息...'"
        :class="{'private-mode': currentPrivateTarget}"
      ></textarea>
      <button @click="sendMessage" class="send-button">
        发送
      </button>
    </div>
  </div>
</template>

<script>
import { ref, onMounted, watch, nextTick } from 'vue'

export default {
  name: 'ChatContainer',
  props: {
    currentRoom: {
      type: String,
      required: true
    },
    currentPrivateTarget: {
      type: String,
      default: null
    },
    messages: {
      type: Array,
      required: true
    },
    username: {
      type: String,
      required: true
    }
  },
  
  setup(props, { emit }) {
    const inputMessage = ref('')
    const messagesContainer = ref(null)
    const messageInput = ref(null)
    
    // 判断消息是否为当前用户发送的
    const isMyMessage = (message) => {
      console.log(`判断消息归属: ${message.username} vs ${props.username}, isSelf=${message.isSelf}`)
      return message.username === props.username || message.isSelf === true
    }
    
    // 获取用户名首字母
    const getInitial = (username) => {
      return username ? username.charAt(0).toUpperCase() : '?'
    }
    
    // 格式化时间
    const formatTime = (timestamp) => {
      if (!timestamp) return ''
      
      const date = new Date(timestamp)
      const hours = date.getHours().toString().padStart(2, '0')
      const minutes = date.getMinutes().toString().padStart(2, '0')
      
      return `${hours}:${minutes}`
    }
    
    // 滚动到底部
    const scrollToBottom = () => {
      nextTick(() => {
        if (messagesContainer.value) {
          messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
        }
      })
    }
    
    // 发送消息
    const sendMessage = () => {
      const text = inputMessage.value.trim()
      if (text) {
        console.log(`发送消息: ${text}, 用户: ${props.username}`)
        emit('send-message', text)
        inputMessage.value = ''
        
        // 聚焦回输入框
        messageInput.value.focus()
      }
    }
    
    // 监听消息变化，自动滚动
    watch(() => props.messages.length, () => {
      scrollToBottom()
    })
    
    // 监听房间或私聊对象变化
    watch([() => props.currentRoom, () => props.currentPrivateTarget], () => {
      inputMessage.value = ''
      nextTick(() => {
        messageInput.value.focus()
      })
    })
    
    // 组件挂载后
    onMounted(() => {
      console.log(`ChatContainer 已挂载，当前用户: ${props.username}`)
      scrollToBottom()
      messageInput.value.focus()
    })
    
    return {
      inputMessage,
      messagesContainer,
      messageInput,
      isMyMessage,
      getInitial,
      formatTime,
      sendMessage
    }
  }
}
</script>

<style scoped>
.chat-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  flex: 1;
  overflow: hidden;
  background-color: #f5f7fa;
  position: relative;
}

.chat-header {
  padding: 15px 20px;
  background: linear-gradient(135deg, #4361ee, #3f37c9);
  color: white;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  z-index: 10;
}

.room-info h2 {
  margin: 0;
  font-size: 1.3rem;
  display: flex;
  align-items: center;
}

.exit-private-btn {
  margin-left: 10px;
  padding: 4px 8px;
  background: rgba(255, 255, 255, 0.2);
  border: none;
  border-radius: 4px;
  color: white;
  font-size: 0.8rem;
  cursor: pointer;
}

.exit-private-btn:hover {
  background: rgba(255, 255, 255, 0.3);
}

.messages-container {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  scroll-behavior: smooth;
}

.no-messages {
  text-align: center;
  color: #a0aec0;
  margin-top: 40px;
  font-size: 0.9rem;
}

.messages-list {
  display: flex;
  flex-direction: column;
}

.message-item {
  margin-bottom: 16px;
  display: flex;
  align-items: flex-start;
  max-width: 85%;
}

/* 系统消息居中 */
.system-message {
  align-self: center;
  justify-content: center;
  background-color: rgba(0, 0, 0, 0.05);
  padding: 8px 16px;
  border-radius: 16px;
  margin: 8px 0;
}

.system-content {
  color: #718096;
  font-size: 0.85rem;
  text-align: center;
}

/* 右侧消息（我发送的） */
.right-message {
  align-self: flex-end;
  flex-direction: row-reverse;
  margin-left: auto;
}

/* 左侧消息（他人发送的） */
.left-message {
  align-self: flex-start;
  margin-right: auto;
}

.message-avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
  flex-shrink: 0;
  margin: 0 8px;
  color: white;
}

.self-avatar {
  background-color: #4361ee;
}

.other-avatar {
  background-color: #718096;
}

.message-content {
  padding: 10px 15px;
  border-radius: 18px;
  position: relative;
  word-break: break-word;
  max-width: calc(100% - 60px);
}

.self-message {
  background: linear-gradient(135deg, #4361ee, #3a0ca3);
  color: white;
  border-bottom-right-radius: 4px;
}

.other-message {
  background-color: white;
  color: #4a5568;
  border-top-left-radius: 4px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.sender-name {
  font-size: 0.75rem;
  font-weight: 600;
  margin-bottom: 4px;
  color: #4a5568;
}

.message-text {
  font-size: 0.95rem;
  line-height: 1.4;
}

.message-time {
  font-size: 0.7rem;
  text-align: right;
  margin-top: 4px;
  opacity: 0.8;
}

.self-message .message-time {
  color: rgba(255, 255, 255, 0.8);
}

.input-container {
  padding: 15px;
  border-top: 1px solid #e2e8f0;
  background-color: white;
  display: flex;
  align-items: center;
}

textarea {
  flex: 1;
  border: 1px solid #e2e8f0;
  border-radius: 20px;
  padding: 10px 15px;
  font-size: 0.95rem;
  max-height: 100px;
  min-height: 20px;
  resize: none;
  transition: border-color 0.3s, box-shadow 0.3s;
}

textarea:focus {
  outline: none;
  border-color: #4361ee;
  box-shadow: 0 0 0 2px rgba(67, 97, 238, 0.2);
}

textarea.private-mode {
  border-color: #8a56ac;
  background-color: rgba(138, 86, 172, 0.05);
}

.send-button {
  margin-left: 10px;
  background: #4361ee;
  color: white;
  border: none;
  width: 80px;
  height: 40px;
  border-radius: 20px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.send-button:hover {
  background: #3a0ca3;
  transform: translateY(-2px);
}

@media (max-width: 768px) {
  .message-item {
    max-width: 90%;
  }
}
</style>
