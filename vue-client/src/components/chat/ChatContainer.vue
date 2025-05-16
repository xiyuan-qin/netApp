<template>
  <div class="chat-container">
    <div class="chat-header">
      <span v-if="currentPrivateTarget">与 {{ currentPrivateTarget }} 私聊中</span>
      <span v-else>大厅</span>
    </div>
    <div class="message-list">
      <div v-for="msg in messages" :key="msg.id"
        :class="msg.username === username ? 'user-message' : 'other-message'">
        <div class="msg-meta">
          <span class="msg-username">{{ msg.username }}</span>
          <span class="msg-time">{{ formatTime(msg.timestamp) }}</span>
        </div>
        <div class="msg-content">{{ msg.text }}</div>
      </div>
    </div>
    <div class="input-bar">
      <input v-model="input" @keyup.enter="handleSend" placeholder="输入消息..." />
      <button @click="handleSend">发送</button>
    </div>
  </div>
</template>
<script setup>
import { ref, watch, nextTick } from 'vue'
const props = defineProps(['messages', 'username', 'currentPrivateTarget'])
const emit = defineEmits(['send-message'])
const input = ref('')

function handleSend() {
  if (!input.value.trim()) return
  emit('send-message', input.value.trim())
  input.value = ''
}

function formatTime(ts) {
  const d = new Date(ts)
  return d.getHours().toString().padStart(2, '0') + ':' + d.getMinutes().toString().padStart(2, '0')
}

// 自动滚动到底部
const messageListRef = ref(null)
watch(() => props.messages.length, () => {
  nextTick(() => {
    if (messageListRef.value) {
      messageListRef.value.scrollTop = messageListRef.value.scrollHeight
    }
  })
})
</script>
<style scoped>
.chat-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: linear-gradient(to bottom, #f5f7fa, #f0f4f8);
  position: relative;
}
.chat-header {
  padding: 16px 20px;
  background: linear-gradient(to right, #4a76a8, #8a56ac);
  color: white;
  font-size: 1.1rem;
  font-weight: 600;
  border-bottom: 1px solid rgba(255,255,255,0.1);
}
.message-list {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  flex-direction: column;
}
.user-message {
  align-self: flex-end;
  background: linear-gradient(135deg, #4a76a8, #8a56ac);
  color: #fff;
  border-radius: 12px 12px 4px 12px;
  margin: 8px 0 8px auto;
  max-width: 70%;
  padding: 10px 16px;
  box-shadow: 0 2px 8px rgba(62,106,225,0.08);
}
.other-message {
  align-self: flex-start;
  background: #fff;
  color: #333;
  border-radius: 12px 12px 12px 4px;
  margin: 8px auto 8px 0;
  max-width: 70%;
  padding: 10px 16px;
  box-shadow: 0 2px 8px rgba(62,106,225,0.08);
}
.msg-meta {
  display: flex;
  justify-content: space-between;
  font-size: 0.85rem;
  margin-bottom: 2px;
  color: #888;
}
.input-bar {
  display: flex;
  padding: 16px;
  border-top: 1px solid #e3e8f0;
  background: #fff;
}
.input-bar input {
  flex: 1;
  border: 1.5px solid #e3e8f0;
  border-radius: 8px;
  padding: 10px 14px;
  font-size: 1rem;
  outline: none;
  margin-right: 12px;
  transition: border 0.2s;
}
.input-bar input:focus {
  border-color: #8a56ac;
}
.input-bar button {
  background: linear-gradient(135deg, #4a76a8, #8a56ac);
  color: #fff;
  border: none;
  border-radius: 8px;
  padding: 0 24px;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.2s;
}
.input-bar button:hover {
  background: #4a76a8;
}
</style> 