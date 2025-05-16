class WebSocketService {
  constructor() {
    this.socket = null
    this.onOpen = null
    this.onMessage = null
    this.onClose = null
  }

  init({ onOpen, onMessage, onClose }) {
    if (this.socket) this.socket.close()
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const host = window.location.host
    this.socket = new WebSocket(`${protocol}//${host}/ws`)
    this.onOpen = onOpen
    this.onMessage = onMessage
    this.onClose = onClose
    this.socket.onopen = onOpen
    this.socket.onmessage = e => {
      try {
        const msg = JSON.parse(e.data)
        onMessage && onMessage(msg)
      } catch {}
    }
    this.socket.onclose = onClose
  }

  send(msg) {
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify(msg))
    }
  }
}

export default new WebSocketService() 