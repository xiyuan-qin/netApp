<template>
  <div class="app-wrapper">
    <router-view/>
  </div>
</template>

<style>
:root {
  /* 主题色彩方案 - 专业技术感 */
  --primary-color: #3e6ae1;      /* 主色调：深蓝色 */
  --secondary-color: #8a56ac;    /* 次色调：紫色 */
  --accent-color: #4caf50;       /* 强调色：绿色 */
  --warning-color: #ff9800;      /* 警告色：橙色 */
  --error-color: #f44336;        /* 错误色：红色 */
  --success-color: #4caf50;      /* 成功色：绿色 */
  
  /* 中性色 */
  --bg-color: #f7f9fc;           /* 背景色：浅灰蓝 */
  --light-bg: #f0f4f8;           /* 浅背景：更浅的灰 */
  --dark-bg: #1a1a2e;            /* 深背景：深蓝黑 */
  --border-color: #dbe1e8;       /* 边框色：浅灰 */
  
  /* 文本颜色 */
  --text-color: #2d3748;         /* 主文本：深灰 */
  --text-light: #718096;         /* 次要文本：中灰 */
  --text-white: #ffffff;         /* 白色文本 */
  
  /* 阴影 */
  --shadow: 0 4px 20px rgba(0, 0, 0, 0.1);  /* 标准阴影 */
  --shadow-lg: 0 10px 30px rgba(0, 0, 0, 0.15); /* 大阴影 */
  
  /* 字体 */
  --font-sans: 'Inter', 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif;
  --font-mono: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  
  /* 圆角 */
  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-lg: 16px;
  
  /* 动画 */
  --transition: all 0.3s ease;
}

body {
  font-family: var(--font-sans);
  background-color: var(--bg-color);
  color: var(--text-color);
  margin: 0;
  padding: 0;
  height: 100vh;
  display: flex;
  justify-content: center;
  align-items: center;
  overflow: hidden;
}

#app {
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
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

/* 聊天气泡动画 */
@keyframes popIn {
  0% { opacity: 0; transform: scale(0.9) translateY(10px); }
  100% { opacity: 1; transform: scale(1) translateY(0); }
}

/* 渐变按钮和交互元素 */
.gradient-btn {
  background: linear-gradient(135deg, var(--primary-color), var(--secondary-color));
  color: white;
  border: none;
  border-radius: var(--radius-md);
  padding: 8px 16px;
  font-weight: 500;
  cursor: pointer;
  transition: var(--transition);
  box-shadow: 0 2px 10px rgba(62, 106, 225, 0.3);
}

.gradient-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 15px rgba(62, 106, 225, 0.4);
}
</style>