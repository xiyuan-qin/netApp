<template>
  <div class="sidebar">
    <div class="user-info">
      <div class="avatar">{{ username.charAt(0) }}</div>
      <div class="name">{{ username }}</div>
    </div>
    <div class="user-list">
      <div class="user-list-title">在线用户</div>
      <div v-for="user in users" :key="user.username" class="user-item">
        <span>{{ user.username }}</span>
        <button v-if="user.username !== username" @click="$emit('start-private-chat', user.username)">私聊</button>
        <span v-if="currentPrivateTarget === user.username" class="private-indicator">私聊中</span>
      </div>
    </div>
    <div v-if="currentPrivateTarget" class="private-exit">
      <button @click="$emit('exit-private-mode')">退出私聊</button>
    </div>
  </div>
</template>
<script setup>
defineProps(['users', 'username', 'currentPrivateTarget'])
</script>
<style scoped>
.sidebar {
  width: 220px;
  background: linear-gradient(135deg, #4a76a8 0%, #8a56ac 100%);
  color: #fff;
  display: flex;
  flex-direction: column;
  padding: 24px 12px 12px 12px;
  border-right: 1px solid #e3e8f0;
}
.user-info {
  display: flex;
  align-items: center;
  margin-bottom: 24px;
}
.avatar {
  width: 44px;
  height: 44px;
  background: #fff;
  color: #4a76a8;
  border-radius: 50%;
  font-size: 1.6rem;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 12px;
  font-weight: bold;
}
.name {
  font-size: 1.1rem;
  font-weight: 600;
}
.user-list-title {
  font-size: 1rem;
  margin-bottom: 8px;
  font-weight: 500;
}
.user-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: rgba(255,255,255,0.08);
  border-radius: 8px;
  padding: 6px 10px;
  margin-bottom: 6px;
}
.user-item button {
  background: #fff;
  color: #4a76a8;
  border: none;
  border-radius: 6px;
  padding: 2px 10px;
  font-size: 0.9rem;
  cursor: pointer;
  margin-left: 8px;
  transition: background 0.2s;
}
.user-item button:hover {
  background: #e3e8f0;
}
.private-indicator {
  color: #ffe066;
  font-size: 0.85rem;
  margin-left: 8px;
}
.private-exit {
  margin-top: 18px;
  text-align: center;
}
.private-exit button {
  background: #fff;
  color: #8a56ac;
  border: none;
  border-radius: 8px;
  padding: 6px 18px;
  font-size: 1rem;
  cursor: pointer;
  font-weight: 600;
  transition: background 0.2s;
}
.private-exit button:hover {
  background: #f3e8ff;
}
</style> 