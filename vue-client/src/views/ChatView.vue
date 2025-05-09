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
      :ws-connection-status="connectionStatus"
      @manual-connect="manualConnect"
      @clear-logs="clearLogs"
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
    let connectionAttempts = 0
    let maxConnectionAttempts = 5
    let reconnectTimeout = null
    
    // 用于跟踪已在本地显示的消息，避免重复显示
    let displayedLocalMessages = new Set()
    
    // 检测是否是移动设备
    const isMobileDevice = () => {
      return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent);
    }
    
    // 生成消息唯一标识
    const generateMessageKey = (username, text, timestamp) => {
      return `${username}:${text.substring(0, 20)}:${timestamp}`
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
              
              // 移动设备上增加额外日志
              if (isMobileDevice()) {
                console.log('移动设备接收消息:', message)
              }
              
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
    
    // 更新连接状态
    const updateConnectionStatus = (status) => {
      connectionStatus.value = status
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
          messagesQueue.push(messageId)
          
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
      
      // 打印到控制台，便于调试
      console.log(`[${type}] ${message}`)
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
      console.log(`显示消息: ${fromUsername}: ${text}, isSelf: ${isSelf}, 当前用户: ${username.value}`)
      
      // 创建消息唯一标识
      const messageKey = generateMessageKey(fromUsername, text, timestamp)
      
      // 检查消息是否已经显示过
      if (displayedLocalMessages.has(messageKey)) {
        console.log('跳过已显示的消息:', messageKey)
        return
      } else {
        // 添加到已显示消息集合
        displayedLocalMessages.add(messageKey)
      }
      
      // 确保正确识别自己发送的消息
      // 通过比较用户名或检查isSelf标志
      const isCurrentUserMessage = isSelf || fromUsername === username.value
      
      messages.value.push({
        type: 'chat',
        username: fromUsername,
        text: text,
        isSelf: isCurrentUserMessage,
        timestamp: timestamp
      })
      
      // 限制消息数量，避免内存占用过高
      if (messages.value.length > 200) {
        messages.value.shift()
      }
      
      // 定期清理已显示消息记录
      if (displayedLocalMessages.size % 20 === 0) {
        setTimeout(() => {
          cleanupOldDisplayedMessages();
        }, 1000);
      }
    }
    
    // 显示私聊消息
    const displayPrivateMessage = (fromUsername, text, isSelf, timestamp, target) => {
      console.log(`显示私聊消息: ${fromUsername} -> ${target}: ${text}`)
      
      // 创建私聊消息的唯一键
      const privateMessageKey = generateMessageKey(fromUsername, text, timestamp)
      
      // 检查消息是否已经显示过
      if (displayedLocalMessages.has(privateMessageKey)) {
        console.log('跳过已显示的私聊消息:', privateMessageKey)
        return
      } else {
        displayedLocalMessages.add(privateMessageKey)
      }
      
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
      
      // 定期清理旧的消息记录，避免内存泄漏
      // 每添加20条消息清理一次
      if (displayedLocalMessages.size % 20 === 0) {
        setTimeout(() => {
          cleanupOldDisplayedMessages();
        }, 1000);
      }
    }
    
    // 清理过旧的已显示消息记录
    const cleanupOldDisplayedMessages = () => {
      // 如果记录太多，进行清理
      if (displayedLocalMessages.size > 100) {
        console.log('清理过期的消息记录');
        displayedLocalMessages.clear();
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
    
    // 生成唯一ID
    const generateId = () => {
      return Date.now().toString(36) + Math.random().toString(36).substring(2)
    }
    
    // 发送消息
    const sendMessage = (text) => {
      if (!text) return
      
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
        } else if (text === '/ping') {
          // 直接调用ping函数，不通过服务器命令
          sendPing()
          displaySystemMessage('发送ping请求...')
        } else {
          // 其他命令发送到服务器处理
          sendChatMessage('command', username.value, currentRoom.value, text)
        }
      } else {
        // 普通消息 - 不再在本地先显示，等服务器返回后再显示
        // 避免用户名不一致导致的重复消息问题
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
    
    // 生命周期钩子
    let pingInterval;
    
    onMounted(() => {
      // 页面加载时连接服务器
      connect()
      
      // 添加欢迎消息
      displaySystemMessage('欢迎使用网络实验聊天应用！')
      
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
      exitPrivateMode,
      manualConnect,
      clearLogs
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