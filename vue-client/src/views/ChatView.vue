<!-- eslint-disable -->
<template>
  <div class="chat-app">
    <!-- 侧边栏 -->
    <div class="sidebar">
      <!-- 连接状态 -->
      <div class="connection-info">
        <div class="status-indicator">
          <span class="status-dot" :class="connectionStatusClass"></span>
          <span>{{ connectionStatus }}</span>
        </div>
        <div class="server-info">
          <div><span>服务器:</span> {{ serverAddress || '--' }}</div>
          <div><span>客户端:</span> {{ clientAddress || '--' }}</div>
          <div><span>延迟:</span> {{ networkLatency }}</div>
        </div>
        <div class="stats">
          <div>发送/接收: {{ sentCount }}/{{ receivedCount }}</div>
          <button @click="sendPing" class="ping-btn">PING</button>
        </div>
      </div>
      
      <!-- 房间列表 -->
      <div class="rooms">
        <div class="section-header">
          <h3>房间列表</h3>
          <button @click="showCreateRoom = true" class="add-btn">+</button>
        </div>
        <ul class="room-list">
          <li v-for="room in rooms" 
              :key="room.name" 
              :class="{ active: currentRoom === room.name }" 
              @click="joinRoom(room.name)">
            # {{ room.name }}
          </li>
        </ul>
        
        <!-- 创建房间对话框 -->
        <div v-if="showCreateRoom" class="modal-backdrop" @click="showCreateRoom = false">
          <div class="modal-content" @click.stop>
            <h3>创建/加入房间</h3>
            <input 
              ref="newRoomInput"
              v-model="newRoomName" 
              @keyup.enter="createNewRoom" 
              placeholder="房间名称"
            />
            <div class="modal-actions">
              <button @click="showCreateRoom = false">取消</button>
              <button @click="createNewRoom" :disabled="!newRoomName.trim()">确认</button>
            </div>
          </div>
        </div>
      </div>
      
      <!-- 用户列表 -->
      <div class="users">
        <div class="section-header">
          <h3>在线用户</h3>
        </div>
        <ul class="user-list">
          <li v-for="user in users" 
              :key="user.username" 
              :class="{ 'is-self': user.username === username }"
              @click="startPrivateChat(user.username)">
            <span class="user-avatar">{{ getUserInitial(user.username) }}</span>
            <span class="user-name">{{ user.username }}</span>
          </li>
        </ul>
      </div>
    </div>
    
    <!-- 聊天区域 -->
    <div class="chat-container">
      <!-- 聊天头部 -->
      <div class="chat-header">
        <h2>
          <template v-if="currentPrivateTarget">
            私聊: {{ currentPrivateTarget }}
            <button class="exit-btn" @click="exitPrivateMode">退出私聊</button>
          </template>
          <template v-else>
            房间: {{ currentRoom }}
          </template>
        </h2>
      </div>
      
      <!-- 消息区域 -->
      <div class="messages-area" ref="messagesArea">
        <div v-if="messages.length === 0" class="empty-message">
          暂无消息，开始聊天吧！
        </div>
        
        <template v-for="(message, index) in messages" :key="index">
          <!-- 系统消息 -->
          <div v-if="message.type === 'system'" class="system-message">
            {{ message.text }}
          </div>
          
          <!-- 自己的消息 -->
          <div v-else-if="message.username === username" class="message-row self-message">
            <div class="message-bubble self-bubble">
              {{ message.text }}
              <div class="message-time">{{ formatTime(message.timestamp) }}</div>
            </div>
          </div>
          
          <!-- 他人的消息 -->
          <div v-else class="message-row other-message">
            <div class="user-avatar">{{ getUserInitial(message.username) }}</div>
            <div class="message-bubble other-bubble">
              <div class="sender-name">{{ message.username }}</div>
              {{ message.text }}
              <div class="message-time">{{ formatTime(message.timestamp) }}</div>
            </div>
          </div>
        </template>
      </div>
      
      <!-- 消息输入区 -->
      <div class="input-area">
        <textarea 
          ref="messageInput"
          v-model="newMessage" 
          placeholder="输入消息..." 
          @keydown.enter.prevent="sendMessage"
          :class="{ 'private-mode': currentPrivateTarget }"
        ></textarea>
        <button @click="sendMessage" class="send-btn">发送</button>
      </div>
    </div>
    
    <!-- 网络监控 -->
    <div class="network-monitor">
      <div class="monitor-header">
        <h3>网络监控</h3>
        <div class="status-indicator" :class="connectionStatusClass">
          {{ connectionStatus }}
        </div>
      </div>
      <div class="log-container" ref="logContainer">
        <div v-for="(log, index) in networkLog" :key="index" class="log-entry" :class="log.className">
          <span class="log-time">{{ log.time }}</span>
          <span class="log-type">{{ log.type }}</span>
          <span class="log-message">{{ log.message }}</span>
        </div>
      </div>
      <div class="network-actions">
        <button @click="manualConnect" class="action-button">重新连接</button>
        <button @click="clearLogs" class="action-button secondary">清空日志</button>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'

export default {
  setup() {
    // 状态变量
    const connectionStatus = ref('未连接')
    const serverAddress = ref('')
    const clientAddress = ref('')
    const networkLatency = ref('--')
    const sentCount = ref(0)
    const receivedCount = ref(0)
    const averageLatency = ref('--')
    const currentRoom = ref('大厅')
    const currentPrivateTarget = ref(null)
    const username = ref('用户' + Math.floor(Math.random() * 1000))
    const rooms = ref([{ name: '大厅', active: true }])
    const users = ref([])
    const messages = ref([])
    const networkLog = ref([])
    const newMessage = ref('')
    const showCreateRoom = ref(false)
    const newRoomName = ref('')
    
    // 引用
    const messagesArea = ref(null)
    const messageInput = ref(null)
    const newRoomInput = ref(null)
    const logContainer = ref(null)
    
    // WebSocket 相关
    let socket = null
    let pingStartTime = 0
    let pingCount = 0
    let totalLatency = 0
    const messageIdMap = new Map()
    let connectionAttempts = 0
    const maxConnectionAttempts = 5
    let reconnectTimeout = null
    
    // 追踪已显示的本地消息
    const displayedMessages = new Set()
    
    // 计算连接状态样式
    const connectionStatusClass = computed(() => {
      if (connectionStatus.value === '已连接') return 'connected'
      if (connectionStatus.value === '正在连接...') return 'connecting'
      return 'disconnected'
    })
    
    // 获取用户名首字母作为头像
    const getUserInitial = (name) => {
      return name ? name.charAt(0).toUpperCase() : '?'
    }
    
    // 格式化时间
    const formatTime = (timestamp) => {
      if (!timestamp) return ''
      
      const date = new Date(timestamp)
      const hours = date.getHours().toString().padStart(2, '0')
      const minutes = date.getMinutes().toString().padStart(2, '0')
      
      return `${hours}:${minutes}`
    }
    
    // 生成消息唯一标识
    const generateMessageKey = (username, text, timestamp) => {
      return `${username}:${text.substring(0, 20)}:${timestamp}`
    }
    
    // 生成唯一ID - 用于消息发送时的跟踪标识
    const generateId = () => {
      return Date.now().toString(36) + Math.random().toString(36).substring(2)
    }
    
    // 检测是否是移动设备
    const isMobileDevice = () => {
      return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent)
    }
    
    // 添加网络日志
    const logNetwork = (type, message, className) => {
      networkLog.value.push({
        time: new Date().toLocaleTimeString(),
        type,
        message,
        className
      })
      
      if (networkLog.value.length > 100) {
        networkLog.value.shift()
      }
      
      nextTick(() => {
        if (logContainer.value) {
          logContainer.value.scrollTop = logContainer.value.scrollHeight
        }
      })
      
      console.log(`[${type}] ${message}`)
    }
    
    // 更新连接状态
    const updateConnectionStatus = (status) => {
      connectionStatus.value = status
    }
    
    // 滚动消息到底部
    const scrollToBottom = () => {
      nextTick(() => {
        if (messagesArea.value) {
          messagesArea.value.scrollTop = messagesArea.value.scrollHeight
        }
      })
    }
    
    // 显示系统消息
    const displaySystemMessage = (text) => {
      messages.value.push({
        type: 'system',
        text,
        timestamp: Date.now()
      })
      
      if (messages.value.length > 200) {
        messages.value.shift()
      }
      
      scrollToBottom()
    }
    
    // 显示普通消息
    const displayMessage = (fromUsername, text, isSelf, timestamp) => {
      // 生成消息唯一标识
      const messageKey = generateMessageKey(fromUsername, text, timestamp)
      
      // 检查是否已显示过
      if (displayedMessages.has(messageKey)) {
        return
      }
      
      displayedMessages.add(messageKey)
      
      // 添加消息
      messages.value.push({
        type: 'chat',
        username: fromUsername,
        text,
        isSelf,
        timestamp
      })
      
      // 限制消息数量
      if (messages.value.length > 200) {
        messages.value.shift()
      }
      
      // 清理显示记录
      cleanupOldDisplayedMessages()
      
      scrollToBottom()
    }
    
    // 清理旧的显示消息记录
    const cleanupOldDisplayedMessages = () => {
      if (displayedMessages.size > 100) {
        const oldEntries = Array.from(displayedMessages).slice(0, 50)
        oldEntries.forEach(key => displayedMessages.delete(key))
      }
    }
    
    // 显示私聊消息
    const displayPrivateMessage = (fromUsername, text, isSelf, timestamp, target) => {
      // 创建私聊消息的唯一键
      const privateMessageKey = generateMessageKey(fromUsername, text, timestamp)
      
      // 检查消息是否已经显示过
      if (displayedMessages.has(privateMessageKey)) {
        return
      } else {
        displayedMessages.add(privateMessageKey)
      }
      
      // 明确标记消息是否为当前用户发送的
      const isCurrentUserMessage = isSelf || fromUsername === username.value
      
      messages.value.push({
        type: 'private',
        username: fromUsername,
        text: text,
        isSelf: isCurrentUserMessage, // 确保这个字段被正确标记
        timestamp: timestamp,
        target: target
      })
      
      // 限制消息数量，避免内存占用过高
      if (messages.value.length > 200) {
        messages.value.shift()
      }
      
      // 定期清理旧的消息记录，避免内存泄漏
      if (displayedMessages.size % 20 === 0) {
        setTimeout(() => {
          cleanupOldDisplayedMessages();
        }, 1000);
      }
      
      scrollToBottom()
    }
    
    // 更新用户列表
    const updateUserList = (userListText) => {
      const userArray = userListText.split(',')
      users.value = userArray
        .filter(Boolean)
        .map(userInfo => {
          const [username, address] = userInfo.split(':')
          return {
            username,
            address: address || '',
            isSelf: username === username.value
          }
        })
    }
    
    // 连接WebSocket服务器
    const connect = () => {
      // 清除可能存在的重连定时器
      if (reconnectTimeout) {
        clearTimeout(reconnectTimeout)
        reconnectTimeout = null
      }
      
      // 如果已经有连接，先关闭
      if (socket) {
        try {
          // 保存引用以便在关闭完成后进行清理
          const oldSocket = socket
          // 将socket置为null，避免重复关闭
          socket = null
          
          // 确保清理所有事件处理器，避免内存泄漏
          oldSocket.onopen = null
          oldSocket.onmessage = null
          oldSocket.onclose = null
          oldSocket.onerror = null
          
          // 记录要关闭的连接状态
          const socketState = oldSocket.readyState
          console.log(`正在关闭旧连接，当前状态: ${socketState}`)
          
          if (socketState === WebSocket.OPEN || socketState === WebSocket.CONNECTING) {
            // 使用标准关闭码和原因
            oldSocket.close(1000, "正常关闭")
            logNetwork('关闭', '手动关闭旧连接', 'info')
          }
        } catch(e) {
          console.error("关闭现有连接出错:", e)
        }
      }
      
      // 等待一小段时间确保旧连接完全关闭
      setTimeout(() => {
        connectionAttempts++
        updateConnectionStatus('正在连接...')
        
        // 检查当前协议是http还是https
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
        
        // 获取当前主机地址
        const host = window.location.host
        let wsUrl
        
        // 针对移动设备使用IP直连方式
        if (isMobileDevice()) {
          // 在移动设备上使用硬编码的IP地址
          wsUrl = `${protocol}//172.20.10.3:8080/ws`
          console.log('移动设备检测: 使用固定IP连接', wsUrl)
        } else {
          // 在桌面设备上使用相对路径
          wsUrl = `${protocol}//${host}/ws`
          console.log('桌面设备: 使用当前主机连接', wsUrl)
        }
        
        try {
          console.log(`尝试创建新WebSocket连接: ${wsUrl}`)
          socket = new WebSocket(wsUrl)
          
          socket.onopen = () => { // 移除 event 参数，因为未使用
            connectionAttempts = 0
            updateConnectionStatus('已连接')
            
            if (isMobileDevice()) {
              serverAddress.value = '172.20.10.3:8080'
            } else {
              serverAddress.value = window.location.host
            }
            
            logNetwork('连接', '连接已建立', 'info')
            
            // 发送初始消息以设置用户名
            sendChatMessage('chat', username.value, currentRoom.value, '')
            
            // 添加系统消息
            displaySystemMessage('已成功连接到服务器')
          }
          
          socket.onmessage = (event) => {
            receivedCount.value++
            
            try {
              const message = JSON.parse(event.data)
              logNetwork('接收', `${message.msg_type}: ${message.text ? (message.text.substring(0, 30) + (message.text.length > 30 ? '...' : '')) : '空消息'}`, 'received')
              
              // 确认消息收到，将消息ID从待确认队列中移除
              if (message.id && messageIdMap.has(message.id)) {
                messageIdMap.delete(message.id)
              }
              
              // 处理不同类型的消息
              switch (message.msg_type) {
                case 'chat':
                  // 确保文本不为空才显示消息
                  if (message.text && message.text.trim() !== '') {
                    // 确定此消息是否为当前用户发送的
                    const isSelfMessage = message.username === username.value
                    console.log(`收到消息 - 用户: ${message.username}, 我的用户名: ${username.value}, 是自己发的: ${isSelfMessage ? 'YES' : 'NO'}`)
                    
                    displayMessage(message.username, message.text, isSelfMessage, message.timestamp)
                  }
                  break
                  
                case 'system':
                  displaySystemMessage(message.text)
                  
                  // 提取服务器信息中的客户端IP
                  if (message.text && message.text.includes('您的IP地址')) {
                    const ipMatch = message.text.match(/您的IP地址: ([^,]+)/)
                    if (ipMatch && ipMatch[1]) {
                      clientAddress.value = ipMatch[1]
                    }
                  }
                  break
                  
                case 'private':
                  // 处理私聊消息
                  if (!currentPrivateTarget.value && message.username !== username.value) {
                    startPrivateChat(message.username)
                  }
                  
                  // 显示私聊消息
                  if (message.text && message.text.trim() !== '') {
                    // 确定私聊消息是否为自己发送的
                    const isSelfMessage = message.username === username.value
                    displayPrivateMessage(message.username, message.text, 
                      isSelfMessage, 
                      message.timestamp, 
                      message.target)
                  }
                  break
                  
                case 'userlist':
                  if (message.text) {
                    updateUserList(message.text)
                  }
                  break
                  
                case 'ping':
                  // 接收到服务器ping，立即回复pong
                  sendChatMessage('pong', username.value, currentRoom.value, message.text || Date.now().toString())
                  
                  // 如果是我们的ping请求，计算延迟
                  if (pingStartTime > 0) {
                    const latency = Date.now() - pingStartTime
                    networkLatency.value = `${latency}ms`
                    
                    // 更新平均延迟
                    pingCount++
                    totalLatency += latency
                    averageLatency.value = `${Math.round(totalLatency / pingCount)}ms`
                    
                    pingStartTime = 0
                  }
                  break
                  
                case 'pong':
                  // 收到服务器的pong响应，计算延迟
                  if (pingStartTime > 0) {
                    const latency = Date.now() - pingStartTime
                    networkLatency.value = `${latency}ms`
                    
                    // 更新平均延迟
                    pingCount++
                    totalLatency += latency
                    averageLatency.value = `${Math.round(totalLatency / pingCount)}ms`
                    
                    pingStartTime = 0
                  }
                  break
                  
                default:
                  console.warn('未知消息类型:', message.msg_type, message)
              }
              
            } catch (e) {
              console.error('无法解析消息:', e, event.data)
              logNetwork('错误', '消息解析失败: ' + e.message, 'error')
              
              // 显示原始消息以便调试
              if (typeof event.data === 'string') {
                logNetwork('调试', `原始消息内容: ${event.data.substring(0, 100)}${event.data.length > 100 ? '...' : ''}`, 'error')
              }
            }
          }
          
          socket.onclose = (event) => {
            updateConnectionStatus('已断开')
            console.log('WebSocket连接关闭:', event.code, event.reason)
            logNetwork('关闭', `WebSocket连接关闭: 代码=${event.code}, 原因=${event.reason || '未指定'}`, 'warning')
            
            // 如果不是手动关闭，尝试重连
            if (socket) { // 只有当socket引用仍然存在(不是由我们手动置null)才尝试重连
              if (connectionAttempts < maxConnectionAttempts) {
                const delay = Math.min(5000 * Math.pow(1.5, connectionAttempts - 1), 30000)
                console.log(`将在${Math.round(delay/1000)}秒后尝试重连(尝试${connectionAttempts}/${maxConnectionAttempts})`)
                
                reconnectTimeout = setTimeout(connect, delay)
              } else {
                console.log(`已达到最大重连次数(${maxConnectionAttempts})，请手动重连`)
              }
            }
            
            // 清理socket引用
            socket = null
          }
          
          socket.onerror = (error) => {
            updateConnectionStatus('连接错误')
            console.error('WebSocket错误:', error)
            logNetwork('错误', 'WebSocket连接错误', 'error')
          }
          
          // 设置连接超时
          setTimeout(() => {
            if (socket && socket.readyState === WebSocket.CONNECTING) {
              console.log('连接超时，正在关闭')
              socket.close(1006, "连接超时")
              // 不要在这里设置socket = null，让onclose事件处理
            }
          }, 10000) // 10秒超时
          
        } catch (e) {
          console.error("创建WebSocket连接出错:", e)
          updateConnectionStatus('连接失败')
        }
      }, 300); // 给旧连接300毫秒关闭时间
    }
    
    // 手动触发连接
    const manualConnect = () => {
      connectionAttempts = 0
      console.log('手动触发连接')
      connect()
    }
    
    // 清除日志
    const clearLogs = () => {
      networkLog.value = []
      console.log('日志已清除')
    }
    
    // 发送ping请求测试延迟
    const sendPing = () => {
      if (!socket || socket.readyState !== WebSocket.OPEN) {
        displaySystemMessage('未连接到服务器，无法发送ping')
        return
      }
      
      pingStartTime = Date.now()
      // 直接使用ping类型，而不是使用command发送/ping
      sendChatMessage('ping', username.value, currentRoom.value, Date.now().toString())
      networkLatency.value = '测量中...'
      logNetwork('ping', '发送ping请求', 'sent')
    }
    
    // 发送消息基础函数
    const sendChatMessage = (type, username, room, text, target = null) => {
      if (!socket || socket.readyState !== WebSocket.OPEN) {
        if (type !== 'pong') { // 不要为pong消息显示错误
          displaySystemMessage('未连接到服务器，无法发送消息')
        }
        return false
      }
      
      const messageId = generateId()
      const message = {
        msg_type: type,
        username: username,
        room: room,
        text: text,
        timestamp: Date.now(),
        id: messageId,
        target: target
      }
      
      try {
        const msgJson = JSON.stringify(message)
        socket.send(msgJson)
        sentCount.value++
        
        if (type !== 'ping' && type !== 'pong') {
          logNetwork('发送', `${type}: ${text.substring(0, 30)}${text.length > 30 ? '...' : ''}`, 'sent')
        }
        
        if ((type === 'chat' || type === 'private') && text) {
          messageIdMap.set(messageId, message)
          
          // 60秒后检查消息是否收到确认
          setTimeout(() => {
            if (messageIdMap.has(messageId)) {
              messageIdMap.delete(messageId)
              displaySystemMessage('消息可能未送达: ' + text.substring(0, 20) + '...')
            }
          }, 60000)
        }
        
        return true
      } catch (e) {
        console.error('发送消息失败:', e)
        logNetwork('错误', '发送失败: ' + e.message, 'error')
        return false
      }
    }
    
    // 发送私聊消息
    const sendPrivateMessage = (targetUser, text) => {
      sendChatMessage('private', username.value, '私聊', text, targetUser)
    }
    
    // 加入房间
    const joinRoom = (roomName) => {
      if (!roomName) return
      
      if (socket && socket.readyState === WebSocket.OPEN) {
        // 更新当前房间
        currentRoom.value = roomName
        
        // 发送房间切换消息
        sendChatMessage('command', username.value, roomName, `/join ${roomName}`)
        
        // 清空当前消息并添加系统消息
        messages.value = []
        displaySystemMessage(`加入房间: ${roomName}`)
        
        // 更新房间列表中的活跃状态
        rooms.value.forEach(r => r.active = (r.name === roomName))
        
        // 如果是新房间，添加到房间列表
        const existingRoom = rooms.value.find(r => r.name === roomName)
        if (!existingRoom) {
          rooms.value.push({ name: roomName, active: true })
        }
        
        // 如果在私聊模式，退出私聊
        if (currentPrivateTarget.value) {
          exitPrivateMode()
        }
      } else {
        displaySystemMessage('无法加入房间: 未连接到服务器')
      }
    }
    
    // 创建新房间
    const createNewRoom = () => {
      const roomName = newRoomName.value.trim()
      if (!roomName) return
      
      // 加入/创建房间
      joinRoom(roomName)
      
      // 关闭对话框并清空输入
      showCreateRoom.value = false
      newRoomName.value = ''
    }
    
    // 开始私聊
    const startPrivateChat = (targetUsername) => {
      if (targetUsername === username.value) {
        displaySystemMessage('无法与自己私聊')
        return
      }
      
      // 设置私聊目标
      currentPrivateTarget.value = targetUsername
      
      // 清空当前消息列表
      messages.value = []
      
      // 添加系统消息
      displaySystemMessage(`开始与 ${targetUsername} 的私聊`)
      
      // 聚焦输入框
      nextTick(() => {
        if (messageInput.value) {
          messageInput.value.focus()
        }
      })
    }
    
    // 退出私聊模式
    const exitPrivateMode = () => {
      // 保存当前私聊对象以便显示退出消息
      const previousTarget = currentPrivateTarget.value
      
      // 清除私聊目标
      currentPrivateTarget.value = null
      
      // 清空消息列表
      messages.value = []
      
      // 显示退出私聊的系统消息
      displaySystemMessage(`退出与 ${previousTarget} 的私聊`)
      
      // 重新加载当前房间消息
      sendChatMessage('command', username.value, currentRoom.value, `/join ${currentRoom.value}`)
    }
    
    // 生命周期钩子
    let pingInterval;
    
    onMounted(() => {
      // 页面加载时连接服务器
      connect()
      
      // 添加欢迎消息
      displaySystemMessage('欢迎使用网络聊天应用！')
      
      // 定期发送ping以保持连接 (60秒一次)
      pingInterval = setInterval(() => {
        if (socket && socket.readyState === WebSocket.OPEN) {
          sendChatMessage('ping', username.value, currentRoom.value, Date.now().toString())
        }
      }, 60000)
    })
    
    onUnmounted(() => {
      clearInterval(pingInterval)
      if (socket) {
        socket.close()
      }
      if (reconnectTimeout) {
        clearTimeout(reconnectTimeout)
      }
    })
    
    return {
      // 状态
      connectionStatus,
      connectionStatusClass,
      serverAddress,
      clientAddress,
      networkLatency,
      sentCount,
      receivedCount,
      averageLatency,
      currentRoom,
      currentPrivateTarget,
      username,
      rooms,
      users,
      messages,
      networkLog,
      newMessage,
      showCreateRoom,
      newRoomName,
      
      // 引用
      messagesArea,
      messageInput,
      newRoomInput,
      logContainer,
      
      // 方法
      getUserInitial,
      formatTime,
      sendMessage,
      sendPing,
      manualConnect,
      clearLogs,
      
      // 添加缺失的方法
      joinRoom,
      createNewRoom,
      startPrivateChat,
      exitPrivateMode,
      cleanupOldDisplayedMessages,
      sendChatMessage,
      sendPrivateMessage
    }
  }
}
</script>

<style scoped>
.chat-app {
  display: flex;
  width: 100%;
  max-width: 1300px;
  height: 85vh;
  background-color: white;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
}

/* 侧边栏 */
.sidebar {
  width: 250px;
  background-color: #f8f9fa;
  border-right: 1px solid #e9ecef;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.connection-info {
  padding: 15px;
  background: linear-gradient(135deg, #f0f4f8, #e6edf5);
  border-bottom: 1px solid #e9ecef;
}

.status-indicator {
  display: flex;
  align-items: center;
  margin-bottom: 8px;
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  margin-right: 8px;
  background-color: #ccc;
}

.status-dot.connected {
  background-color: #4caf50;
}

.status-dot.connecting {
  background-color: #ff9800;
  animation: pulse 1.5s infinite;
}

.status-dot.disconnected {
  background-color: #f44336;
}

.server-info {
  margin: 10px 0;
  font-size: 0.85rem;
  color: #666;
}

.server-info div {
  margin: 4px 0;
}

.server-info span {
  font-weight: bold;
  color: #444;
}

.stats {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 10px;
}

.ping-btn {
  background-color: #4361ee;
  color: white;
  border: none;
  border-radius: 4px;
  padding: 5px 10px;
  cursor: pointer;
}

/* 房间区域 */
.rooms, .users {
  padding: 15px;
  border-bottom: 1px solid #e9ecef;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.section-header h3 {
  font-size: 1rem;
  margin: 0;
  color: #444;
}

.add-btn {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background-color: #4361ee;
  color: white;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.room-list, .user-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.room-list li, .user-list li {
  padding: 8px 10px;
  margin: 2px 0;
  border-radius: 6px;
  cursor: pointer;
}

.room-list li:hover, .user-list li:hover {
  background-color: #f0f4f8;
}

.room-list li.active {
  background-color: #e3f0ff;
  color: #4361ee;
  font-weight: bold;
}

/* 用户列表 */
.users {
  flex: 1;
  overflow-y: auto;
}

.user-list li {
  display: flex;
  align-items: center;
}

.user-avatar {
  width: 30px;
  height: 30px;
  border-radius: 50%;
  background-color: #4361ee;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 10px;
  font-size: 0.9rem;
  font-weight: bold;
}

/* 创建房间对话框 */
.modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background-color: white;
  padding: 20px;
  border-radius: 8px;
  width: 300px;
}

.modal-content h3 {
  margin: 0 0 15px 0;
  text-align: center;
}

.modal-content input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  margin-bottom: 15px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.modal-actions button {
  padding: 6px 12px;
  border-radius: 4px;
  border: none;
  cursor: pointer;
}

.modal-actions button:last-child {
  background-color: #4361ee;
  color: white;
}

/* 聊天区域 */
.chat-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.chat-header {
  padding: 15px;
  background-color: #4361ee;
  color: white;
  display: flex;
  align-items: center;
}

.chat-header h2 {
  font-size: 1.2rem;
  margin: 0;
  display: flex;
  align-items: center;
}

.exit-btn {
  background-color: rgba(255,255,255,0.2);
  border: none;
  color: white;
  padding: 5px 10px;
  margin-left: 10px;
  border-radius: 4px;
  cursor: pointer;
}

/* 消息区域 */
.messages-area {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
  background-color: #f5f7fa;
}

.empty-message {
  text-align: center;
  color: #aaa;
  margin-top: 40px;
}

.message-row {
  margin-bottom: 15px;
  display: flex;
  align-items: flex-start;
  max-width: 85%;
}

/* 自己发送的消息 */
.self-message {
  margin-left: auto;
  justify-content: flex-end;
}

/* 他人发送的消息 */
.other-message {
  margin-right: auto;
  justify-content: flex-start;
}

.message-bubble {
  padding: 10px 15px;
  border-radius: 16px;
  max-width: 100%;
  word-break: break-word;
}

.self-bubble {
  background-color: #4361ee;
  color: white;
  border-bottom-right-radius: 5px;
}

.other-bubble {
  background-color: white;
  color: #333;
  border-bottom-left-radius: 5px;
  box-shadow: 0 1px 2px rgba(0,0,0,0.1);
}

.sender-name {
  font-size: 0.75rem;
  font-weight: bold;
  margin-bottom: 5px;
  color: #555;
}

.message-time {
  font-size: 0.7rem;
  text-align: right;
  margin-top: 5px;
  opacity: 0.7;
}

/* 系统消息 */
.system-message {
  background-color: rgba(0,0,0,0.05);
  color: #666;
  padding: 8px 12px;
  border-radius: 12px;
  font-size: 0.85rem;
  text-align: center;
  margin: 10px auto;
  max-width: 80%;
}

/* 消息输入区 */
.input-area {
  display: flex;
  padding: 15px;
  background-color: white;
  border-top: 1px solid #e9ecef;
}

.input-area textarea {
  flex: 1;
  border: 1px solid #ddd;
  border-radius: 20px;
  padding: 10px 15px;
  resize: none;
  height: 40px;
  outline: none;
}

.input-area textarea:focus {
  border-color: #4361ee;
}

.input-area textarea.private-mode {
  border-color: #8a56ac;
  background-color: rgba(138, 86, 172, 0.05);
}

.send-btn {
  margin-left: 10px;
  background-color: #4361ee;
  color: white;
  border: none;
  border-radius: 20px;
  width: 80px;
  cursor: pointer;
  font-weight: bold;
}

/* 网络监控 */
.network-monitor {
  width: 300px;
  background-color: #f8f9fa;
  border-left: 1px solid #e9ecef;
  display: flex;
  flex-direction: column;
}

.monitor-header {
  padding: 15px;
  background-color: #4361ee;
  color: white;
}

.monitor-header h3 {
  margin: 0 0 10px 0;
  font-size: 1.1rem;
}

.log-container {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
}

.log-entry {
  padding: 6px 10px;
  margin-bottom: 5px;
  border-radius: 6px;
  background-color: white;
  font-size: 0.85rem;
}

.log-time {
  color: #888;
  margin-right: 5px;
  font-size: 0.75rem;
}

.log-type {
  font-weight: bold;
  margin-right: 5px;
}

.log-entry.info {
  background-color: #e3f2fd;
}

.log-entry.sent {
  background-color: #e8f5e9;
}

.log-entry.received {
  background-color: #f1f8e9;
}

.log-entry.warning {
  background-color: #fff3e0;
}

.log-entry.error {
  background-color: #ffebee;
}

.network-actions {
  display: flex;
  justify-content: space-between;
  padding: 10px;
  background-color: white;
  border-top: 1px solid #e9ecef;
}

.action-button {
  padding: 6px 12px;
  border: none;
  border-radius: 4px;
  background-color: #4361ee;
  color: white;
  cursor: pointer;
}

.action-button.secondary {
  background-color: #6c757d;
}

/* 动画 */
@keyframes pulse {
  0% { opacity: 1; }
  50% { opacity: 0.5; }
  100% { opacity: 1; }
}

/* 响应式设计 */
@media (max-width: 1100px) {
  .chat-app {
    flex-direction: column;
    height: 95vh;
  }
  
  .sidebar {
    width: 100%;
    max-height: 200px;
    flex-direction: row;
    overflow-x: auto;
  }
  
  .network-monitor {
    width: 100%;
    max-height: 200px;
  }
}

@media (max-width: 600px) {
  .chat-app {
    border-radius: 0;
    height: 100vh;
  }
}
</style>