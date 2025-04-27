<template>
  <div class="network-monitor">
    <div class="monitor-header">
      <h3>网络监控</h3>
      <div class="connection-status" :class="wsStatusClass">
        WebSocket: {{ wsConnectionStatus }}
      </div>
      <div class="stats">
        <div class="stat-item">
          <span class="label">已发送:</span>
          <span class="value">{{ sentCount }}</span>
        </div>
        <div class="stat-item">
          <span class="label">已接收:</span>
          <span class="value">{{ receivedCount }}</span>
        </div>
        <div class="stat-item">
          <span class="label">平均延迟:</span>
          <span class="value">{{ averageLatency }}</span>
        </div>
      </div>
    </div>
    <div class="log-container">
      <div 
        v-for="(log, index) in networkLog" 
        :key="index" 
        class="log-entry"
        :class="log.className"
      >
        <span class="log-time">{{ log.time }}</span>
        <span class="log-type">{{ log.type }}</span>
        <span class="log-message">{{ log.message }}</span>
      </div>
    </div>
    <div class="network-actions">
      <button @click="manualConnect" class="action-button">
        手动连接
      </button>
      <button @click="clearLogs" class="action-button secondary">
        清除日志
      </button>
    </div>
  </div>
</template>

<script>
export default {
  name: 'NetworkMonitor',
  props: {
    sentCount: Number,
    receivedCount: Number,
    averageLatency: String,
    networkLog: Array,
    wsConnectionStatus: {
      type: String,
      default: '未连接'
    }
  },
  computed: {
    wsStatusClass() {
      switch(this.wsConnectionStatus) {
        case '已连接': return 'connected';
        case '正在连接...': return 'connecting';
        case '连接错误': return 'error';
        case '已断开': return 'error';
        default: return 'disconnected';
      }
    }
  },
  methods: {
    manualConnect() {
      this.$emit('manual-connect');
    },
    clearLogs() {
      this.$emit('clear-logs');
    }
  }
}
</script>

<style scoped>
.network-monitor {
  width: 300px;
  border-left: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  background: linear-gradient(to bottom, #ffffff, #f8fafc);
  box-shadow: -2px 0 10px rgba(0, 0, 0, 0.03);
}

.monitor-header {
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
  background: linear-gradient(to right, var(--primary-color), var(--secondary-color));
  color: white;
}

.monitor-header h3 {
  font-size: 1.1rem;
  font-weight: 600;
  margin: 0 0 12px 0;
  color: white;
  display: flex;
  align-items: center;
}

.monitor-header h3::before {
  content: '📊';
  margin-right: 8px;
  font-size: 1.2rem;
}

.stats {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.stat-item {
  font-size: 0.85rem;
  background-color: rgba(255, 255, 255, 0.2);
  padding: 5px 10px;
  border-radius: var(--radius-md);
  backdrop-filter: blur(5px);
  transition: var(--transition);
}

.stat-item:hover {
  background-color: rgba(255, 255, 255, 0.3);
  transform: translateY(-2px);
}

.stat-item .label {
  color: rgba(255, 255, 255, 0.8);
  margin-right: 6px;
}

.stat-item .value {
  font-weight: 600;
  color: white;
}

.log-container {
  flex: 1;
  overflow-y: auto;
  padding: 15px;
  font-family: var(--font-mono);
  font-size: 0.85rem;
  background-color: #f8fafc;
  position: relative;
}

.log-container::before {
  content: "";
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: url('data:image/svg+xml;utf8,<svg width="20" height="20" xmlns="http://www.w3.org/2000/svg"><rect width="20" height="20" fill="none"/><path d="M0,0 L20,20" stroke="%233e6ae1" stroke-width="0.5" opacity="0.03"/></svg>') repeat;
  z-index: -1;
}

.log-entry {
  padding: 6px 10px;
  margin-bottom: 8px;
  border-radius: var(--radius-md);
  display: flex;
  flex-wrap: wrap;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
  transition: var(--transition);
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(5px); }
  to { opacity: 1; transform: translateY(0); }
}

.log-entry:hover {
  box-shadow: 0 3px 6px rgba(0, 0, 0, 0.08);
}

.log-time {
  color: var(--text-light);
  margin-right: 8px;
  font-size: 0.75rem;
  opacity: 0.8;
}

.log-type {
  font-weight: bold;
  margin-right: 8px;
  min-width: 50px;
  text-transform: uppercase;
  font-size: 0.7rem;
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  display: inline-block;
  text-align: center;
}

.log-message {
  word-break: break-word;
  line-height: 1.4;
  margin-top: 2px;
  width: 100%;
}

.info {
  background-color: rgba(62, 106, 225, 0.1);
}

.info .log-type {
  background-color: rgba(62, 106, 225, 0.2);
  color: var(--primary-color);
}

.sent {
  background-color: rgba(76, 175, 80, 0.1);
}

.sent .log-type {
  background-color: rgba(76, 175, 80, 0.2);
  color: var(--success-color);
}

.received {
  background-color: rgba(108, 117, 125, 0.1);
}

.received .log-type {
  background-color: rgba(108, 117, 125, 0.2);
  color: var(--text-color);
}

.error {
  background-color: rgba(244, 67, 54, 0.1);
}

.error .log-type {
  background-color: rgba(244, 67, 54, 0.2);
  color: var(--error-color);
}

@media (max-width: 1000px) {
  .network-monitor {
    width: 100%;
    height: 200px;
    border-left: none;
    border-top: 1px solid var(--border-color);
  }
  
  .monitor-header {
    padding: 10px 15px;
  }
  
  .log-container {
    padding: 10px;
  }
}

.connection-status {
  margin: 8px 0;
  padding: 4px 10px;
  border-radius: var(--radius-md);
  font-size: 0.9rem;
  font-weight: 500;
  display: inline-block;
  transition: all 0.3s;
}

.connection-status.connected {
  background: rgba(76, 175, 80, 0.2);
  color: #2e7d32;
}

.connection-status.connecting {
  background: rgba(255, 193, 7, 0.2);
  color: #f57f17;
  animation: pulse 1.5s infinite;
}

.connection-status.disconnected,
.connection-status.error {
  background: rgba(244, 67, 54, 0.2);
  color: #d50000;
}

@keyframes pulse {
  0% { opacity: 1; }
  50% { opacity: 0.6; }
  100% { opacity: 1; }
}

.network-actions {
  padding: 10px;
  display: flex;
  justify-content: space-between;
  border-top: 1px solid var(--border-color);
  background-color: #f8fafc;
}

.action-button {
  padding: 6px 12px;
  border-radius: var(--radius-md);
  border: none;
  background: linear-gradient(to right, var(--primary-color), var(--secondary-color));
  color: white;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 2px 4px rgba(62, 106, 225, 0.2);
}

.action-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(62, 106, 225, 0.3);
}

.action-button.secondary {
  background: linear-gradient(to right, #6c757d, #495057);
  box-shadow: 0 2px 4px rgba(108, 117, 125, 0.2);
}

.action-button.secondary:hover {
  box-shadow: 0 4px 8px rgba(108, 117, 125, 0.3);
}
</style>