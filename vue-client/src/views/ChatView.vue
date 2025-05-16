<template>
  <div class="app-container">
    <Sidebar
      :users="users"
      :username="username"
      :current-private-target="currentPrivateTarget"
      @start-private-chat="startPrivateChat"
      @exit-private-mode="exitPrivateMode"
    />
    <ChatContainer
      :messages="messages"
      :username="username"
      :current-private-target="currentPrivateTarget"
      @send-message="sendMessage"
      @exit-private-mode="exitPrivateMode"
    />
    <NetworkMonitor :is-connected="isConnected" />
  </div>
</template>

<script setup>
import { useChatStore } from '../stores/chat'
import Sidebar from '../components/chat/Sidebar.vue'
import ChatContainer from '../components/chat/ChatContainer.vue'
import NetworkMonitor from '../components/chat/NetworkMonitor.vue'
import { onMounted } from 'vue'

const chat = useChatStore()
const { users, username, messages, currentPrivateTarget, isConnected, initialize, sendMessage, startPrivateChat, exitPrivateMode } = chat

onMounted(() => {
  initialize()
})
</script>

<style scoped>
.app-container {
  display: flex;
  width: 95vw;
  max-width: 1300px;
  height: 85vh;
  background: #fff;
  border-radius: 18px;
  box-shadow: 0 4px 32px rgba(62,106,225,0.08);
  overflow: hidden;
  position: relative;
  margin: 30px auto;
  border: 1px solid rgba(219,225,232,0.7);
}
@media (max-width: 1000px) {
  .app-container {
    flex-direction: column;
    height: 95vh;
  }
}
@media (max-width: 600px) {
  .app-container {
    width: 100vw;
    height: 100vh;
    border-radius: 0;
  }
}
</style> 