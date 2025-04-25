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
    
    <div class="message-container" ref="messageContainerRef">
      <div v-if="messages.length === 0" class="no-messages">
        暂无消息，开始聊天吧！
      </div>
      
      <div 
        v-for="(message, index) in messages" 
        :key="index" 
        class="message" 
        :class="{
          'system-message': message.type === 'system',
          'user-message': message.type === 'chat' && message.isSelf,
          'other-message': message.type === 'chat' && !message.isSelf,
          'private-message-sent': message.type === 'private' && message.isSelf,
          'private-message-received': message.type === 'private' && !message.isSelf,
        }"
      >
        <div class="message-header" v-if="message.username">
          <span class="username">{{ message.username }}</span>
          <span class="timestamp">{{ formatTime(message.timestamp) }}</span>
        </div>
        <div class="message-content" v-html="formatMessage(message.text)"></div>
      </div>
    </div>
    
    <div class="message-input">
      <textarea 
        ref="messageInputRef"
        v-model="messageText" 
        placeholder="输入消息..." 
        @keyup.enter="sendMessage"
        :class="{ 'private-mode': currentPrivateTarget }"
      ></textarea>
      <button class="send-btn" @click="sendMessage">发送</button>
    </div>
  </div>
</template>

<script>
import { ref, onMounted, watch, nextTick } from 'vue'

export default {
  name: 'ChatContainer',
  props: {
    currentRoom: String,
    currentPrivateTarget: String,
    messages: Array,
    username: String
  },
  
  setup(props, { emit }) {
    const messageText = ref('')
    const messageContainerRef = ref(null)
    const messageInputRef = ref(null)
    
    // 在消息容器中滚动到最底部
    const scrollToBottom = () => {
      nextTick(() => {
        if (messageContainerRef.value) {
          messageContainerRef.value.scrollTop = messageContainerRef.value.scrollHeight
        }
      })
    }
    
    // 格式化消息文本，处理链接和表情
    const formatMessage = (text) => {
      if (!text) return ''
      
      // 将URL转换为链接
      const urlRegex = /(https?:\/\/[^\s]+)/g
      let formattedText = text.replace(urlRegex, '<a href="$1" target="_blank" rel="noopener noreferrer">$1</a>')
      
      // 简单的表情符号替换
      formattedText = formattedText
        .replace(/:\)/g, '😊')
        .replace(/:\(/g, '😢')
        .replace(/:D/g, '😃')
        .replace(/;\)/g, '😉')
        .replace(/:P/g, '😛')
        .replace(/<3/g, '❤️')
      
      return formattedText
    }
    
    // 格式化时间戳
    const formatTime = (timestamp) => {
      if (!timestamp) return ''
      
      const date = new Date(timestamp)
      const hours = date.getHours().toString().padStart(2, '0')
      const minutes = date.getMinutes().toString().padStart(2, '0')
      
      return `${hours}:${minutes}`
    }
    
    // 发送消息
    const sendMessage = () => {
      const text = messageText.value.trim()
      if (text) {
        emit('send-message', text)
        messageText.value = ''
        
        // 聚焦回输入框
        messageInputRef.value.focus()
      }
    }
    
    // 监听消息列表变化，自动滚动到底部
    watch(() => props.messages.length, () => {
      scrollToBottom()
    })
    
    // 监听当前房间变化，清空输入框
    watch(() => props.currentRoom, () => {
      messageText.value = ''
    })
    
    // 监听私聊目标变化，清空输入框
    watch(() => props.currentPrivateTarget, () => {
      messageText.value = ''
      messageInputRef.value.focus()
    })
    
    // 组件挂载后自动滚动到底部
    onMounted(() => {
      scrollToBottom()
      messageInputRef.value.focus()
    })
    
    return {
      messageText,
      messageContainerRef,
      messageInputRef,
      sendMessage,
      formatMessage,
      formatTime
    }
  }
}
</script>

<style scoped>
.chat-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background: linear-gradient(to bottom, #f5f7fa, #f0f4f8);
  position: relative;
}

.chat-header {
  padding: 16px 20px;
  background: linear-gradient(to right, var(--primary-color), var(--secondary-color));
  color: white;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  position: relative;
  z-index: 10;
}

.room-info h2 {
  font-size: 1.3rem;
  font-weight: 600;
  margin: 0;
  color: white;
  display: flex;
  align-items: center;
}

.exit-private-btn {
  margin-left: 15px;
  background-color: rgba(255, 255, 255, 0.2);
  border: none;
  border-radius: var(--radius-md);
  padding: 6px 12px;
  font-size: 0.85rem;
  cursor: pointer;
  color: white;
  transition: var(--transition);
}

.exit-private-btn:hover {
  background-color: rgba(255, 255, 255, 0.3);
  transform: translateY(-2px);
}

.message-container {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  background-color: transparent;
  scroll-behavior: smooth;
  position: relative;
}

/* 消息容器装饰背景 */
.message-container::before {
  content: "";
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: url('data:image/svg+xml;utf8,<svg width="100" height="100" xmlns="http://www.w3.org/2000/svg"><rect width="100" height="100" fill="none"/><path d="M20,10 L80,10 L80,90 L20,90 Z" stroke="%233e6ae1" stroke-width="0.5" fill="none" stroke-opacity="0.03"/><circle cx="50" cy="50" r="30" stroke="%238a56ac" stroke-width="0.5" fill="none" stroke-opacity="0.05"/></svg>') repeat;
  opacity: 0.1;
  z-index: -1;
}

.no-messages {
  text-align: center;
  color: var(--text-light);
  margin-top: 80px;
  font-size: 1rem;
  opacity: 0.6;
}

.message {
  margin-bottom: 20px;
  animation: popIn 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  max-width: 80%;
  word-wrap: break-word;
  position: relative;
}

.system-message {
  background-color: rgba(62, 106, 225, 0.07);
  border-radius: var(--radius-md);
  color: var(--text-light);
  font-size: 0.92rem;
  text-align: center;
  margin: 15px auto;
  max-width: 90%;
  padding: 10px 15px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.03);
  border-left: 3px solid rgba(62, 106, 225, 0.2);
}

.user-message {
  margin-left: auto;
}

.other-message {
  margin-right: auto;
}

.private-message-sent {
  margin-left: auto;
}

.private-message-received {
  margin-right: auto;
}

.message-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 4px;
  padding: 0 2px;
}

.username {
  font-weight: 600;
  font-size: 0.9rem;
  color: var(--text-color);
}

.timestamp {
  font-size: 0.75rem;
  color: var(--text-light);
  margin-left: 10px;
}

.message-content {
  padding: 12px 16px;
  border-radius: var(--radius-md);
  line-height: 1.4;
  position: relative;
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.05);
}

/* 用户消息气泡 */
.user-message .message-content {
  background: linear-gradient(135deg, var(--primary-color), #5175dd);
  color: white;
  border-bottom-right-radius: 4px;
}

.user-message .message-content::after {
  content: '';
  position: absolute;
  bottom: 0;
  right: -8px;
  width: 16px;
  height: 16px;
  background: var(--primary-color);
  border-bottom-left-radius: 16px;
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.05);
  clip-path: polygon(0 0, 0 100%, 100% 100%);
}

/* 其他用户消息气泡 */
.other-message .message-content {
  background-color: white;
  border-bottom-left-radius: 4px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.07);
}

.other-message .message-content::before {
  content: '';
  position: absolute;
  bottom: 0;
  left: -8px;
  width: 16px;
  height: 16px;
  background: white;
  border-bottom-right-radius: 16px;
  clip-path: polygon(100% 0, 0 100%, 100% 100%);
}

/* 私聊消息样式 */
.private-message-sent .message-content {
  background: linear-gradient(135deg, var(--secondary-color), #a173be);
  color: white;
  border-bottom-right-radius: 4px;
}

.private-message-received .message-content {
  background: linear-gradient(135deg, #efe7f5, #f0e3ff);
  border-bottom-left-radius: 4px;
}

.private-message-received .username,
.private-message-sent .username {
  color: var(--secondary-color);
  position: relative;
}

.private-message-received .username::before,
.private-message-sent .username::before {
  content: '🔒';
  font-size: 0.8rem;
  margin-right: 5px;
}

.message-input {
  padding: 18px;
  border-top: 1px solid var(--border-color);
  background-color: white;
  display: flex;
  align-items: center;
  box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.05);
  position: relative;
}

textarea {
  flex: 1;
  padding: 12px 16px;
  border: 2px solid var(--border-color);
  border-radius: var(--radius-md);
  resize: none;
  height: 24px;
  font-family: var(--font-sans);
  font-size: 0.95rem;
  outline: none;
  transition: var(--transition);
  box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.05);
  line-height: 1.4;
}

textarea:focus {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px rgba(62, 106, 225, 0.15);
}

textarea.private-mode {
  border-color: var(--secondary-color);
  background-color: rgba(138, 86, 172, 0.03);
  box-shadow: 0 0 0 3px rgba(138, 86, 172, 0.1);
}

.send-btn {
  background: linear-gradient(135deg, var(--primary-color), var(--secondary-color));
  color: white;
  border: none;
  border-radius: var(--radius-md);
  padding: 0 20px;
  height: 42px;
  margin-left: 12px;
  cursor: pointer;
  font-weight: 600;
  font-size: 0.95rem;
  transition: var(--transition);
  box-shadow: 0 2px 8px rgba(62, 106, 225, 0.2);
}

.send-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(62, 106, 225, 0.3);
}

a {
  color: #0366d6;
  text-decoration: none;
}

a:hover {
  text-decoration: underline;
}

@media (max-width: 1000px) {
  .message {
    max-width: 90%;
  }
  
  .message-input {
    padding: 12px;
  }
}

/* 链接在消息中的样式 */
.user-message .message-content a,
.private-message-sent .message-content a {
  color: #e0f0ff;
  text-decoration: underline;
  text-decoration-color: rgba(255,255,255,0.4);
}

.user-message .message-content a:hover,
.private-message-sent .message-content a:hover {
  text-decoration-color: rgba(255,255,255,0.7);
}
</style>
