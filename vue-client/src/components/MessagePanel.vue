<template>
  <div class="message-panel" ref="messageContainer">
    <div class="chat-header">
      <h2>
        {{ chatTitle }}
        <span v-if="typing" class="typing-indicator">{{ typing }} 正在输入...</span>
      </h2>
    </div>
    
    <div class="messages-container" ref="messagesContainer">
      <div v-if="messages.length === 0" class="no-messages">
        <p>暂无消息</p>
        <p class="hint">开始聊天吧！</p>
      </div>
      
      <transition-group name="message-fade">
        <div 
          v-for="(msg, index) in messages" 
          :key="msg.id || index"
          :class="['message', {'my-message': msg.sender === username}]"
        >
          <div class="message-avatar">
            {{ getUserInitial(msg.sender) }}
          </div>
          <div class="message-content">
            <div class="message-header">
              <span class="sender">{{ msg.sender }}</span>
              <span class="time">{{ formatTime(msg.timestamp) }}</span>
            </div>
            <div class="message-text" v-html="formatMessage(msg.content)"></div>
          </div>
        </div>
      </transition-group>
      
      <!-- 滚动到底部指示器 -->
      <div v-if="showScrollToBottom" class="scroll-to-bottom" @click="scrollToBottom">
        <i class="arrow-down">↓</i>
      </div>
    </div>
    
    <div class="message-input">
      <textarea 
        ref="messageInput"
        v-model="newMessage" 
        @keydown.enter.prevent="sendMessage"
        @input="handleTyping"
        placeholder="输入消息..."
        rows="1"
      ></textarea>
      <button @click="sendMessage" :disabled="!newMessage.trim()">
        发送
      </button>
    </div>
  </div>
</template>

<script>
import { ref, computed, onMounted, onUpdated, watch, nextTick } from 'vue';

export default {
  name: 'MessagePanel',
  props: {
    messages: {
      type: Array,
      required: true
    },
    username: {
      type: String,
      required: true
    },
    currentRoom: {
      type: String,
      default: 'general'
    },
    privateTarget: {
      type: String,
      default: null
    },
    typing: {
      type: String,
      default: null
    }
  },
  
  emits: ['send-message', 'typing'],
  
  setup(props, { emit }) {
    const newMessage = ref('');
    const messagesContainer = ref(null);
    const messageInput = ref(null);
    const showScrollToBottom = ref(false);
    const typingTimeout = ref(null);
    
    // 聊天标题显示
    const chatTitle = computed(() => {
      if (props.privateTarget) {
        return `与 ${props.privateTarget} 的私聊`;
      } else {
        return `# ${props.currentRoom}`;
      }
    });
    
    // 获取用户名首字母作为头像
    const getUserInitial = (username) => {
      return username ? username.charAt(0).toUpperCase() : '?';
    };
    
    // 格式化时间
    const formatTime = (timestamp) => {
      if (!timestamp) return '';
      
      const date = new Date(timestamp);
      const hours = date.getHours().toString().padStart(2, '0');
      const minutes = date.getMinutes().toString().padStart(2, '0');
      
      return `${hours}:${minutes}`;
    };
    
    // 格式化消息，处理链接和表情
    const formatMessage = (message) => {
      if (!message) return '';
      
      // 转义HTML特殊字符
      const escaped = message
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#039;');
      
      // 链接识别
      const withLinks = escaped.replace(
        /(https?:\/\/[^\s]+)/g,
        '<a href="$1" target="_blank" rel="noopener">$1</a>'
      );
      
      // TODO: 表情符号处理可以在这里添加
      
      return withLinks;
    };
    
    // 发送消息
    const sendMessage = () => {
      const content = newMessage.value.trim();
      if (!content) return;
      
      emit('send-message', {
        content,
        isPrivate: !!props.privateTarget,
        target: props.privateTarget || props.currentRoom
      });
      
      newMessage.value = '';
      nextTick(() => {
        messageInput.value.focus();
        adjustTextareaHeight();
      });
    };
    
    // 处理输入事件，动态调整输入框高度
    const adjustTextareaHeight = () => {
      if (!messageInput.value) return;
      
      messageInput.value.style.height = 'auto';
      messageInput.value.style.height = `${messageInput.value.scrollHeight}px`;
    };
    
    // 处理正在输入状态
    const handleTyping = () => {
      adjustTextareaHeight();
      
      if (typingTimeout.value) {
        clearTimeout(typingTimeout.value);
      }
      
      emit('typing', true);
      
      typingTimeout.value = setTimeout(() => {
        emit('typing', false);
      }, 2000);
    };
    
    // 检查是否需要显示滚动到底部按钮
    const checkScroll = () => {
      if (!messagesContainer.value) return;
      
      const { scrollTop, scrollHeight, clientHeight } = messagesContainer.value;
      const scrolledUp = scrollHeight - scrollTop - clientHeight > 50;
      
      showScrollToBottom.value = scrolledUp;
    };
    
    // 滚动到底部
    const scrollToBottom = () => {
      nextTick(() => {
        if (messagesContainer.value) {
          messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
        }
      });
    };
    
    // 监听消息变化，自动滚动
    watch(() => props.messages.length, () => {
      scrollToBottom();
    });
    
    // 组件挂载后
    onMounted(() => {
      scrollToBottom();
      if (messageInput.value) {
        messageInput.value.focus();
      }
      
      if (messagesContainer.value) {
        messagesContainer.value.addEventListener('scroll', checkScroll);
      }
    });
    
    // 组件更新后
    onUpdated(() => {
      scrollToBottom();
    });
    
    return {
      newMessage,
      messagesContainer,
      messageInput,
      showScrollToBottom,
      chatTitle,
      getUserInitial,
      formatTime,
      formatMessage,
      sendMessage,
      handleTyping,
      scrollToBottom
    };
  }
};
</script>

<style scoped>
.message-panel {
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.chat-header {
  padding: 15px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.chat-header h2 {
  font-size: 1.2rem;
  margin: 0;
  color: var(--text-color);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.typing-indicator {
  font-size: 0.8rem;
  color: var(--text-light);
  font-weight: normal;
  font-style: italic;
}

.messages-container {
  flex-grow: 1;
  overflow-y: auto;
  padding: 15px;
  position: relative;
}

.no-messages {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-light);
}

.no-messages .hint {
  font-size: 0.9rem;
  margin-top: 5px;
}

.message {
  display: flex;
  margin-bottom: 15px;
  animation: fadeIn 0.3s ease-in-out;
}

.message-fade-enter-active,
.message-fade-leave-active {
  transition: all 0.3s;
}

.message-fade-enter-from,
.message-fade-leave-to {
  opacity: 0;
  transform: translateY(20px);
}

.my-message {
  flex-direction: row-reverse;
}

.message-avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background-color: var(--primary-light);
  color: var(--primary-color);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
  flex-shrink: 0;
}

.my-message .message-avatar {
  background-color: var(--secondary-light);
  color: var(--secondary-color);
}

.message-content {
  max-width: 70%;
  margin: 0 10px;
  padding: 10px 15px;
  background-color: var(--light-bg);
  border-radius: 18px;
  position: relative;
}

.my-message .message-content {
  background-color: var(--primary-light);
}

.message-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 5px;
  font-size: 0.8rem;
}

.sender {
  font-weight: 500;
  color: var(--primary-color);
}

.my-message .sender {
  color: var(--secondary-color);
}

.time {
  color: var(--text-light);
  margin-left: 8px;
}

.message-text {
  word-break: break-word;
  line-height: 1.4;
}

.message-text a {
  color: var(--link-color);
  text-decoration: none;
}

.message-text a:hover {
  text-decoration: underline;
}

.message-input {
  display: flex;
  padding: 15px;
  border-top: 1px solid var(--border-color);
  background-color: var(--light-bg);
  flex-shrink: 0;
  align-items: flex-end;
}

.message-input textarea {
  flex-grow: 1;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  resize: none;
  font-family: inherit;
  font-size: 0.95rem;
  line-height: 1.4;
  max-height: 150px;
  min-height: 40px;
}

.message-input textarea:focus {
  outline: none;
  border-color: var(--primary-color);
}

.message-input button {
  margin-left: 10px;
  background-color: var(--primary-color);
  color: white;
  border: none;
  border-radius: 4px;
  padding: 0 20px;
  height: 40px;
  cursor: pointer;
  transition: background-color 0.3s;
}

.message-input button:hover:not(:disabled) {
  background-color: var(--secondary-color);
}

.message-input button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.scroll-to-bottom {
  position: absolute;
  bottom: 20px;
  right: 20px;
  width: 40px;
  height: 40px;
  background-color: var(--primary-color);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: white;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  transition: transform 0.3s;
}

.scroll-to-bottom:hover {
  transform: translateY(-2px);
}

.arrow-down {
  font-size: 1.2rem;
}

@media (max-width: 768px) {
  .message-content {
    max-width: 85%;
  }
  
  .message-avatar {
    width: 30px;
    height: 30px;
    font-size: 0.8rem;
  }
  
  .message-input {
    padding: 10px;
  }
  
  .message-input textarea {
    padding: 8px;
  }
  
  .message-input button {
    padding: 0 15px;
  }
}

@keyframes fadeIn {
  0% {
    opacity: 0;
    transform: translateY(10px);
  }
  100% {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>