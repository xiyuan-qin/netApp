<template>
  <div id="app">
    <router-view/>
  </div>
</template>

<style>
:root {
  /* 主要颜色 */
  --primary-color: #4361ee;
  --secondary-color: #8a56ac;
  --success-color: #4caf50;
  --warning-color: #ff9800;
  --error-color: #f44336;
  --info-color: #2196f3;
  
  /* 背景色 */
  --light-bg: #f5f7fa;
  --dark-bg: #1a1a2e;
  --border-color: #e2e8f0;
  
  /* 文本颜色 */
  --text-color: #333333;
  --text-light: #718096;
  --text-white: #ffffff;
  
  /* 字体 */
  --font-sans: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
  --font-mono: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  
  /* 圆角 */
  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-lg: 16px;
  
  /* 过渡效果 */
  --transition: all 0.3s ease;
  
  /* 阴影 */
  --shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  --shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.1);
}

* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

html, body {
  height: 100%;
  font-family: var(--font-sans);
  background: linear-gradient(135deg, #f5f7fa 0%, #e4e9f2 100%);
  color: var(--text-color);
}

#app {
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 20px;
}

/* 滚动条样式 */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 10px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 0, 0, 0.3);
}

::-webkit-scrollbar-track {
  background: transparent;
}
</style>