<template>
  <div class="network-monitor">
    <div class="monitor-header">
      <h3>网络监控</h3>
      <div class="connection-status" :class="isConnected ? 'connected' : 'disconnected'">
        WebSocket: {{ isConnected ? '已连接' : '未连接' }}
      </div>
    </div>
    <div class="monitor-body">
      <div class="stat-item">
        <span class="label">状态:</span>
        <span class="value">{{ isConnected ? '在线' : '离线' }}</span>
      </div>
    </div>
  </div>
</template>
<script setup>
defineProps(['isConnected'])
</script>
<style scoped>
.network-monitor {
  width: 260px;
  border-left: 1px solid #e3e8f0;
  background: linear-gradient(to bottom, #fff, #f8fafc);
  display: flex;
  flex-direction: column;
  box-shadow: -2px 0 10px rgba(0,0,0,0.03);
}
.monitor-header {
  padding: 16px 20px;
  border-bottom: 1px solid #e3e8f0;
  background: linear-gradient(to right, #4a76a8, #8a56ac);
  color: white;
}
.monitor-header h3 {
  font-size: 1.1rem;
  font-weight: 600;
  margin: 0 0 12px 0;
  color: white;
}
.connection-status {
  font-size: 1rem;
  font-weight: 500;
  margin-bottom: 8px;
}
.connection-status.connected {
  color: #4caf50;
}
.connection-status.disconnected {
  color: #f44336;
}
.monitor-body {
  padding: 18px 20px;
}
.stat-item {
  font-size: 1rem;
  margin-bottom: 10px;
  color: #333;
}
.label {
  color: #888;
  margin-right: 8px;
}
.value {
  font-weight: 600;
}
</style> 