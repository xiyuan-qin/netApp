import { defineStore } from 'pinia'
import { ref } from 'vue'
import wsService from '../services/websocket'

export const useChatStore = defineStore('chat', () => {
  const username = ref('用户' + Math.floor(Math.random() * 1000))
  const messages = ref([])
  const users = ref([])
  const currentPrivateTarget = ref(null)
  const isConnected = ref(false)

  function generateId() {
    return Date.now().toString(36) + Math.random().toString(36).substring(2)
  }

  function initialize() {
    wsService.init({
      onOpen: () => { isConnected.value = true; sendMessage('', 'chat') },
      onMessage: handleMessage,
      onClose: () => { isConnected.value = false }
    })
  }

  function handleMessage(message) {
    if (message.msg_type === 'userlist') {
      users.value = message.text.split(',').map(u => {
        const [name, addr] = u.split(':')
        return { username: name, address: addr }
      })
      return
    }
    messages.value.push(message)
    if (messages.value.length > 200) messages.value.shift()
  }

  function sendMessage(text, type = 'chat', target = null) {
    if (!isConnected.value) return
    const msg = {
      msg_type: type,
      username: username.value,
      room: '大厅',
      text,
      timestamp: Date.now(),
      id: generateId(),
      target
    }
    wsService.send(msg)
  }

  function startPrivateChat(targetUser) {
    currentPrivateTarget.value = targetUser
  }

  function exitPrivateMode() {
    currentPrivateTarget.value = null
  }

  return {
    username, messages, users, currentPrivateTarget, isConnected,
    initialize, sendMessage, startPrivateChat, exitPrivateMode
  }
}) 