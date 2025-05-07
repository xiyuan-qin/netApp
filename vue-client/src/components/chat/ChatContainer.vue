<template>
  <div class="chat-container">
    <div class="chat-header">
      <h2>
        {{ currentPrivateTarget ? '私聊: ' + currentPrivateTarget : '房间: ' + currentRoom }}
        <button v-if="currentPrivateTarget" @click="$emit('exit-private-mode')" class="exit-btn">退出</button>
      </h2>
    </div>
    
    <div class="messages-area" ref="messageArea">
      <!-- 空消息提示 -->
      <div v-if="messages.length === 0" class="empty-notice">
        暂无消息，开始聊天吧！
      </div>
      
      <!-- 消息列表 -->
      <div v-for="(msg, index) in messages" :key="index" class="message-wrapper">
        <!-- 系统消息 -->
        <div v-if="msg.type === 'system'" style="width:100%; text-align:center; margin:8px 0;">
          <div style="display:inline-block; background-color:#f2f2f2; padding:8px 15px; border-radius:16px; font-size:12px; color:#666;">
            {{ msg.text }}
          </div>
        </div>
        
        <!-- 自己发送的消息，强制靠右 -->
        <div v-else-if="msg.username === username" style="width:100%; display:flex; justify-content:flex-end; margin:10px 0;">
          <div style="max-width:70%; background-color:#4361ee; color:white; padding:10px 15px; border-radius:18px; border-bottom-right-radius:5px; text-align:left;">
            {{ msg.text }}
            <div style="font-size:10px; text-align:right; margin-top:4px; opacity:0.7;">
              {{ formatTime(msg.timestamp) }}
            </div>
          </div>
        </div>
        
        <!-- 他人发送的消息，强制靠左 -->
        <div v-else style="width:100%; display:flex; justify-content:flex-start; margin:10px 0;">
          <div style="max-width:70%; background-color:#ffffff; color:#333; padding:10px 15px; border-radius:18px; border-bottom-left-radius:5px; box-shadow:0 1px 2px rgba(0,0,0,0.1); text-align:left;">
            <div style="font-size:12px; font-weight:bold; margin-bottom:4px; color:#555;">
              {{ msg.username }}
            </div>
            {{ msg.text }}
            <div style="font-size:10px; text-align:right; margin-top:4px; color:#888;">
              {{ formatTime(msg.timestamp) }}
            </div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- 输入区域 -->
    <div class="input-area">
      <textarea 
        ref="messageInput"
        v-model="inputText"
        placeholder="输入消息..."
        @keydown.enter.prevent="sendMessage"
      ></textarea>
      <button @click="sendMessage" class="send-btn">发送</button>
    </div>
  </div>
</template>

<script>
import { ref, onMounted, watch, nextTick } from 'vue'

export default {
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
    const inputText = ref('')
    const messageArea = ref(null)
    const messageInput = ref(null)
    
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
        if (messageArea.value) {
          messageArea.value.scrollTop = messageArea.value.scrollHeight
        }
      })
    }
    
    // 发送消息
    const sendMessage = () => {
      const text = inputText.value.trim()
      if (text) {
        console.log(`发送消息: ${text}`)
        emit('send-message', text)
        inputText.value = ''
        messageInput.value.focus()
      }
    }
    
    // 监听消息变化，自动滚动
    watch(() => props.messages.length, () => {
      scrollToBottom()
    })
    
    // 组件挂载后
    onMounted(() => {
      scrollToBottom()
      messageInput.value.focus()
    })
    
    return {
      inputText,
      messageArea,
      messageInput,
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
  background-color: #f5f7fa;
}

.chat-header {
  padding: 15px;
  background-color: #4361ee;
  color: white;
}

.chat-header h2 {
  margin: 0;
  font-size: 1.2rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.exit-btn {
  background: rgba(255,255,255,0.2);
  border: none;
  color: white;
  padding: 5px 10px;
  font-size: 12px;
  border-radius: 4px;
  cursor: pointer;
}

.messages-area {
  flex: 1;
  overflow-y: auto;
  padding: 15px;
}

.empty-notice {
  text-align: center;
  color: #999;
  margin-top: 40px;
}

.input-area {
  display: flex;
  padding: 15px;
  background-color: white;
  border-top: 1px solid #eee;
}

textarea {
  flex: 1;
  padding: 10px 15px;
  border: 1px solid #ddd;
  border-radius: 18px;
  resize: none;
  height: 24px;
  outline: none;
}

.send-btn {
  margin-left: 10px;
  background-color: #4361ee;
  color: white;
  border: none;
  width: 70px;
  border-radius: 18px;
  cursor: pointer;
  font-weight: bold;
}

/* 禁用所有可能影响消息布局的全局样式 */
.message-wrapper * {
  box-sizing: border-box !important;
}
</style>
