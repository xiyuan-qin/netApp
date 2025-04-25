<template>
  <div class="about">
    <h1>关于本应用</h1>
    <p>这是一个使用 Vue 3 和 Rust 构建的网络聊天应用。</p>
    <p>该应用展示了前后端通信以及实时消息传递的基本功能。</p>
    <div class="features">
      <h2>功能特点</h2>
      <ul>
        <li>多房间聊天</li>
        <li>私聊功能</li>
        <li>网络监控</li>
        <li>消息延迟计算</li>
      </ul>
    </div>
    <p class="version">版本: 0.1.0</p>
  </div>
</template>

<style scoped>
.about {
  max-width: 800px;
  margin: 0 auto;
  padding: 20px;
}

h1 {
  margin-bottom: 20px;
  color: var(--primary-color, #4361ee);
}

.features {
  margin-top: 30px;
  background-color: #f8f9fa;
  padding: 20px;
  border-radius: 8px;
}

.features h2 {
  font-size: 1.3rem;
  margin-bottom: 15px;
}

.features ul {
  padding-left: 20px;
}

.features li {
  margin-bottom: 8px;
}

.version {
  margin-top: 40px;
  color: #6c757d;
  font-size: 0.9rem;
}
</style>