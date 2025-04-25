<template>
  <div class="login-panel">
    <div class="login-container">
      <h1>WebSocket聊天应用</h1>
      
      <div class="form-group">
        <label for="username">请输入用户名</label>
        <input 
          type="text" 
          id="username" 
          v-model="username" 
          @keyup.enter="login"
          placeholder="请输入您的昵称"
          autocomplete="off"
        />
      </div>
      
      <button 
        class="btn-login" 
        @click="login"
        :disabled="!isValidUsername"
      >
        加入聊天
      </button>
      
      <div class="error-message" v-if="errorMsg">
        {{ errorMsg }}
      </div>
    </div>
  </div>
</template>

<script>
import { ref, computed } from 'vue';

export default {
  name: 'LoginPanel',
  emits: ['login'],
  setup(props, { emit }) {
    const username = ref('');
    const errorMsg = ref('');
    
    // 检查用户名是否有效
    const isValidUsername = computed(() => {
      return username.value && username.value.trim().length >= 2;
    });
    
    // 登录方法
    const login = () => {
      if (!isValidUsername.value) {
        errorMsg.value = '用户名至少需要2个字符';
        return;
      }
      
      emit('login', username.value.trim());
    };
    
    return {
      username,
      errorMsg,
      isValidUsername,
      login
    };
  }
};
</script>

<style scoped>
.login-panel {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 100%;
  height: 100vh;
  background: linear-gradient(135deg, #4a76a8 0%, #263238 100%);
}

.login-container {
  background-color: white;
  border-radius: 10px;
  padding: 40px;
  width: 90%;
  max-width: 400px;
  box-shadow: var(--shadow);
  animation: fadeIn 0.5s ease-in-out;
}

.login-container h1 {
  text-align: center;
  color: var(--primary-color);
  margin-bottom: 30px;
  font-size: 1.8rem;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
  color: var(--text-color);
}

.form-group input {
  width: 100%;
  padding: 12px;
  border: 2px solid var(--border-color);
  border-radius: 6px;
  font-size: 16px;
  transition: border-color 0.3s;
}

.form-group input:focus {
  outline: none;
  border-color: var(--primary-color);
}

.btn-login {
  width: 100%;
  padding: 12px;
  background-color: var(--primary-color);
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 16px;
  cursor: pointer;
  transition: background-color 0.3s;
}

.btn-login:hover:not(:disabled) {
  background-color: var(--secondary-color);
}

.btn-login:disabled {
  background-color: #cccccc;
  cursor: not-allowed;
}

.error-message {
  margin-top: 15px;
  color: #e74c3c;
  text-align: center;
  font-size: 14px;
}
</style>