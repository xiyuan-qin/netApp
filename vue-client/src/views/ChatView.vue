<template>
  <div class="app-container">
    <Sidebar 
      :connection-status="connectionStatus" 
      :server-address="serverAddress"
      :client-address="clientAddress"
      :network-latency="networkLatency"
      :sent-count="sentCount"
      :received-count="receivedCount"
      :average-latency="averageLatency"
      :rooms="rooms"
      :users="users"
      :current-room="currentRoom"
      @join-room="joinRoom"
      @create-room="createRoom"
      @start-private-chat="startPrivateChat"
      @ping="sendPing"
    />
    
    <ChatContainer 
      ref="chatContainer"
      :current-room="currentRoom" 
      :current-private-target="currentPrivateTarget"
      :messages="messages"
      :username="username"
      @send-message="sendMessage"
      @exit-private-mode="exitPrivateMode"
    />
    
    <NetworkMonitor 
      :sent-count="sentCount"
      :received-count="receivedCount"
      :average-latency="averageLatency"
      :network-log="networkLog"
    />
  </div>
</template>

<script>
import Sidebar from '@/components/chat/Sidebar.vue'
import ChatContainer from '@/components/chat/ChatContainer.vue'
import NetworkMonitor from '@/components/chat/NetworkMonitor.vue'
import { ref, onMounted, onUnmounted } from 'vue'

export default {
  name: 'ChatView',
  components: {
    Sidebar,
    ChatContainer,
    NetworkMonitor
  },
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
    
    // WebSocket 相关
    let socket = null
    let pingStartTime = 0
    let pingCount = 0
    let totalLatency = 0
    let messageIdMap = new Map()
    let messagesQueue = []
    
    // 连接WebSocket服务器
    const connect = () => {
      updateConnectionStatus('正在连接...', 'orange')
      logNetwork('连接', `正在连接到服务器...`, 'info')
      
      // 检查当前协议是http还是https
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
      const wsUrl = `${protocol}//${window.location.host}/ws`
      
      socket = new WebSocket(wsUrl)
      
      socket.onopen = () => {
        updateConnectionStatus('已连接', 'green')
        serverAddress.value = window.location.host
        logNetwork('连接', '连接已建立', 'info')
        
        // 发送初始消息以设置用户名
        sendChatMessage('chat', username.value, currentRoom.value, '')
        
        // 添加系统消息
        displaySystemMessage('已成功连接到服务器')
      }
      
      socket.onmessage = handleMessage
      
      socket.onclose = (event) => {
        updateConnectionStatus('已断开', 'red')
        logNetwork('断开', `连接已关闭: ${event.code} ${event.reason}`, 'error')
        
        // 5秒后重连
        setTimeout(connect, 5000)
      }
      
      socket.onerror = (error) => {
        updateConnectionStatus('连接错误', 'red')
        logNetwork('错误', '连接错误', 'error')
        console.error('WebSocket错误:', error)
      }
    }
    
    // 处理接收到的消息
    const handleMessage = (event) => {
      receivedCount.value++
      
      try {
        const message = JSON.parse(event.data)
        logNetwork('接收', `${message.msg_type}: ${message.text ? (message.text.substring(0, 30) + (message.text.length > 30 ? '...' : '')) : '空消息'}`, 'received')
        
        // 调试输出
        console.log('收到消息:', message)
        
        switch (message.msg_type) {
          case 'chat':
            // 确保文本不为空才显示消息
            if (message.text && message.text.trim() !== '') {
              displayMessage(message.username, message.text, message.username === username.value, message.timestamp)
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
              // 如果不在私聊模式且收到对方发来的消息，自动进入私聊模式
              startPrivateChat(message.username)
            }
            
            // 显示私聊消息
            if (message.text && message.text.trim() !== '') {
              displayPrivateMessage(message.username, message.text, 
                message.username === username.value, 
                message.timestamp, 
                message.target)
            }
            break
            
          case 'userlist':
            updateUserList(message.text)
            break
            
          case 'ping':
            // 接收到服务器ping，回复pong
            sendChatMessage('pong', username.value, currentRoom.value, message.text)
            
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
        }
        
      } catch (e) {
        console.error('无法解析消息:', e, event.data)
        logNetwork('错误', '消息解析失败: ' + e.message, 'error')
      }
    }
    
    // 更新用户列表
    const updateUserList = (userListText) => {
      const userArray = userListText.split(',')
      users.value = userArray.filter(Boolean).map(userInfo => {
        const [username, address] = userInfo.split(':')
        return {
          username,
          address: address || '',
          isSelf: username === username.value
        }
      })
    }
    
    // 更新房间列表
    const updateRooms = (roomName, isActive = false) => {
      const existingRoom = rooms.value.find(r => r.name === roomName)
      
      if (existingRoom) {
        // 如果房间已存在，只更新active状态
        if (isActive) {
          rooms.value.forEach(r => r.active = (r.name === roomName))
        }
      } else {
        // 如果是新房间，添加到列表
        rooms.value.push({ name: roomName, active: isActive })
      }
    }
    
    // 发送消息基础函数
    const sendChatMessage = (type, username, room, text, target = null) => {
      if (!socket || socket.readyState !== WebSocket.OPEN) {
        displaySystemMessage('未连接到服务器，无法发送消息')
        return
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
        socket.send(JSON.stringify(message))
        sentCount.value++
        
        if ((type === 'chat' || type === 'private') && text) {
          messageIdMap.set(messageId, message)
          messagesQueue.push(messageId)
          
          setTimeout(() => {
            if (messageIdMap.has(messageId)) {
              messageIdMap.delete(messageId)
              displaySystemMessage('消息可能未送达: ' + text.substring(0, 20) + '...')
            }
          }, 60000)
        }
        
        logNetwork('发送', `${type}: ${text.substring(0, 30)}${text.length > 30 ? '...' : ''}`, 'sent')
      } catch (e) {
        console.error('发送消息失败:', e)
        logNetwork('错误', '发送失败: ' + e.message, 'error')
      }
    }
    
    // 发送消息
    const sendMessage = (text) => {
      if (!text) return
      
      // 添加本地显示
      if (!text.startsWith('/')) {
        // 如果是普通消息，先在本地显示，提高响应速度感
        if (currentPrivateTarget.value) {
          displayPrivateMessage(username.value, text, true, Date.now(), currentPrivateTarget.value)
        } else {
          displayMessage(username.value, text, true, Date.now())
        }
      }
      
      // 调试输出
      console.log('发送消息:', text)
      
      // 检查是否为命令
      if (text.startsWith('/')) {
        if (text === '/help') {
          // 客户端处理help命令
          displaySystemMessage(`可用命令:
          /help - 显示帮助
          /rooms - 显示所有房间
          /join <房间名> - 加入指定房间
          /users - 显示当前房间用户
          /msg <用户名> <消息> - 发送私聊消息
          /ping - 测试网络连接
          /stats - 显示网络统计信息`)
        } else if (text.startsWith('/join ')) {
          // 解析房间名
          const roomName = text.substring(6).trim()
          if (roomName) {
            joinRoom(roomName)
          }
        } else if (text.startsWith('/msg ')) {
          // 解析私聊消息
          const parts = text.split(' ')
          if (parts.length >= 3) {
            const targetUser = parts[1]
            const msgText = parts.slice(2).join(' ')
            sendPrivateMessage(targetUser, msgText)
          } else {
            displaySystemMessage('用法: /msg <用户名> <消息内容>')
          }
        } else {
          // 其他命令发送到服务器处理
          sendChatMessage('command', username.value, currentRoom.value, text)
        }
      } else {
        // 普通消息
        if (currentPrivateTarget.value) {
          // 在私聊模式
          sendPrivateMessage(currentPrivateTarget.value, text)
        } else {
          // 房间消息
          sendChatMessage('chat', username.value, currentRoom.value, text)
        }
      }
    }
    
    // 发送私聊消息
    const sendPrivateMessage = (targetUser, text) => {
      sendChatMessage('private', username.value, '私聊', text, targetUser)
    }
    
    // 发送ping请求测试延迟
    const sendPing = () => {
      pingStartTime = Date.now()
      sendChatMessage('command', username.value, currentRoom.value, '/ping')
      networkLatency.value = '测量中...'
    }
    
    // 加入房间
    const joinRoom = (roomName) => {
      if (roomName === currentRoom.value) return
      
      sendChatMessage('join', username.value, roomName, '')
      displaySystemMessage(`正在加入房间: ${roomName}`)
      currentRoom.value = roomName
      
      // 更新房间列表UI
      rooms.value.forEach(room => {
        room.active = (room.name === roomName)
      })
      
      // 如果是新房间，添加到列表
      updateRooms(roomName, true)
      
      // 退出私聊模式
      exitPrivateMode()
    }
    
    // 创建新房间
    const createRoom = (roomName) => {
      if (roomName) {
        joinRoom(roomName)
      }
    }
    
    // 进入私聊模式
    const startPrivateChat = (user) => {
      currentPrivateTarget.value = user
      displaySystemMessage(`已进入与 ${user} 的私聊模式，消息仅对方可见`)
    }
    
    // 退出私聊模式
    const exitPrivateMode = () => {
      if (currentPrivateTarget.value) {
        currentPrivateTarget.value = null
        displaySystemMessage('已退出私聊模式，回到房间聊天')
      }
    }
    
    // 显示消息
    const displayMessage = (fromUsername, text, isSelf, timestamp) => {
      console.log(`显示消息: ${fromUsername}: ${text}`)
      
      messages.value.push({
        type: 'chat',
        username: fromUsername,
        text: text,
        isSelf: isSelf,
        timestamp: timestamp
      })
      
      // 限制消息数量，避免内存占用过高
      if (messages.value.length > 200) {
        messages.value.shift()
      }
    }
    
    // 显示私聊消息
    const displayPrivateMessage = (fromUsername, text, isSelf, timestamp, target) => {
      console.log(`显示私聊消息: ${fromUsername} -> ${target}: ${text}`)
      
      messages.value.push({
        type: 'private',
        username: fromUsername,
        text: text,
        isSelf: isSelf,
        timestamp: timestamp,
        target: target
      })
      
      // 限制消息数量，避免内存占用过高
      if (messages.value.length > 200) {
        messages.value.shift()
      }
    }
    
    // 显示系统消息
    const displaySystemMessage = (text) => {
      console.log(`显示系统消息: ${text}`)
      
      messages.value.push({
        type: 'system',
        text: text,
        timestamp: Date.now()
      })
      
      // 限制消息数量，避免内存占用过高
      if (messages.value.length > 200) {
        messages.value.shift()
      }
    }
    
    // 更新连接状态
    const updateConnectionStatus = (status) => {
      connectionStatus.value = status
    }
    
    // 日志网络事件到网络监视器
    const logNetwork = (type, message, className) => {
      networkLog.value.push({
        time: new Date().toLocaleTimeString(),
        type: type,
        message: message,
        className: className
      })
      
      // 限制日志条目数量
      if (networkLog.value.length > 100) {
        networkLog.value.shift()
      }
    }
    
    // 生成唯一ID
    const generateId = () => {
      return Date.now().toString(36) + Math.random().toString(36).substring(2)
    }
    
    // 生命周期钩子
    onMounted(() => {
      // 页面加载时连接服务器
      connect()
      
      // 添加欢迎消息
      displaySystemMessage('欢迎使用网络实验聊天应用！')
    })
    
    onUnmounted(() => {
      // 组件卸载时关闭WebSocket连接
      if (socket && socket.readyState === WebSocket.OPEN) {
        socket.close()
      }
    })
    
    return {
      // 状态
      connectionStatus,
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
      
      // 方法
      sendMessage,
      sendPing,
      joinRoom,
      createRoom,
      startPrivateChat,
      exitPrivateMode
    }
  }
}
</script>

<style scoped>
.app-container {
  display: flex;
  width: 95%;
  max-width: 1300px;
  height: 85vh;
  background-color: #fff;
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
  position: relative;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(219, 225, 232, 0.7);
}

/* 添加技术感背景装饰 */
.app-container::before {
  content: "";
  position: absolute;
  top: -50%;
  left: -50%;
  width: 200%;
  height: 200%;
  background: radial-gradient(circle at center, rgba(62, 106, 225, 0.03) 0%, transparent 70%);
  z-index: -1;
}

/* 响应式设计 */
@media (max-width: 1000px) {
  .app-container {
    flex-direction: column;
    height: 95vh;
  }
}

@media (max-width: 600px) {
  .app-container {
    width: 100%;
    height: 100vh;
    border-radius: 0;
  }
}
</style>