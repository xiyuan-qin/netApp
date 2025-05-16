<template>
  <div id="app">
    <div class="bg-gradient"></div>
    <router-view />
  </div>
</template>

<script setup>
// 无需逻辑
</script>

<style>
body, #app {
  min-height: 100vh;
  margin: 0;
  font-family: 'PingFang SC', 'Microsoft YaHei', Arial, sans-serif;
  background: #f5f7fa;
}
#app {
  position: relative;
  min-height: 100vh;
  overflow-x: hidden;
}
.bg-gradient {
  position: fixed;
  top: 0; left: 0; right: 0; bottom: 0;
  z-index: -1;
  background: linear-gradient(135deg, #e0e7ff 0%, #f3e8ff 100%);
  opacity: 0.7;
}
</style> 