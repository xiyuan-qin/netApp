# 实时通讯应用

这是一个基于WebSocket的实时通讯应用，专为计算机网络实验设计，用于演示网络通信原理和协议特性。应用使用Rust作为服务端语言，前端使用HTML、CSS和JavaScript实现用户界面。


## 功能特点

### 1. 网络信息显示
- 显示服务器地址和客户端IP地址
- 实时网络延迟测量
- 连接状态监控

### 2. 房间/频道功能
- 支持创建和加入不同的聊天房间
- 模拟不同子网之间的通信
- 房间用户列表自动更新

### 3. 网络状态监控
- 发送和接收消息统计
- 平均网络延迟统计
- 实时网络事件日志
- 协议信息展示

### 4. 消息类型支持
- 普通文本消息
- 系统消息
- 命令消息
- 网络诊断消息
- 私聊消息

### 5. 网络可靠性实现
- 消息确认机制
- 心跳检测保持连接
- 自动重连机制
- 每条消息都有唯一ID

### 6. 命令系统
- `/help` - 显示命令帮助
- `/rooms` - 查看所有可用房间
- `/join <房间名>` - 加入特定房间
- `/users` - 显示当前房间用户列表
- `/msg <用户名> <消息>` - 发送私聊消息
- `/ping` - 测试网络连接延迟
- `/stats` - 显示网络统计信息

## 技术架构

### 服务端
- 使用Rust语言和Actix Web框架
- WebSocket实现全双工通信
- 多线程处理客户端连接
- 使用Mutex实现共享状态安全访问

### 客户端
- 纯前端实现，无需额外插件
- 使用原生WebSocket API
- 实时更新界面
- 网络日志和统计功能

## 如何运行

### 安装依赖
确保已安装Rust和Cargo:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 克隆并运行项目

```bash
# 克隆项目（如果是从Git仓库获取）
git clone git@github.com:xiyuan-qin/netApp.git
cd net_app

# 编译并运行
cargo run
```

服务器默认在 `0.0.0.0:8080` 上启动，可以通过任意设备在局域网内访问。

### 使用方法

1. 在浏览器中访问 `http://localhost:8080`
2. 输入您的用户名
3. 发送消息或使用命令
4. 可以创建或加入房间与其他用户聊天
5. 使用网络监视器查看连接状态和统计信息
6. 点击用户名可进行私聊

## 分发与共享应用

### 方法1: 局域网内分享（最简单）

在局域网内部署是最简单的方式，适用于在同一网络环境（如家庭网络、学校网络、公司内网）中的用户。

#### 步骤:

1. **编译应用**
   ```bash
   cargo build --release
   ```

2. **启动服务器**
   ```bash
   ./target/release/net_app
   ```
   或Windows上:
   ```bash
   target\release\net_app.exe
   ```

3. **查找服务器IP地址**
   - 在Linux/macOS上:
     ```bash
     ifconfig | grep "inet "
     ```
   - 在Windows上:
     ```
     ipconfig
     ```
   记录下你的本地IP地址（通常是192.168.x.x或10.0.x.x格式）

4. **让其他用户连接**
   - 告知其他用户在浏览器中输入: `http://[你的IP地址]:8080`
   - 例如: `http://192.168.1.15:8080`
   - 查找你的IP地址的方法:
     - 在macOS上: 
       - WiFi连接: `ipconfig getifaddr en0` 
       - 有线连接: `ipconfig getifaddr en1`
       - 或者使用: `ifconfig | grep "inet " | grep -v 127.0.0.1`
       - 也可以在"系统设置 > 网络"中查看
     - 在Windows上: 打开命令提示符，输入 `ipconfig`，查找"IPv4 地址"
     - 在Linux上: 打开终端，输入 `ip addr show | grep "inet " | grep -v 127.0.0.1`
     - 通常你需要找到类似 `192.168.x.x` 或 `10.0.x.x` 格式的地址，这是你的局域网IP

5. **注意事项**
   - 确保防火墙允许8080端口通信
   - 所有用户必须在同一个局域网内

### 方法2: 互联网部署（通过云服务）

要让互联网上的任何人都能使用您的应用，需要将其部署到云服务器上。

#### 选项A: 使用云服务器（如阿里云, 腾讯云, AWS等）

1. **租用云服务器**
   - 创建一个Linux服务器实例（Ubuntu或CentOS等）
   - 确保开放8080端口（在云控制台的安全组/防火墙设置中）

2. **安装Rust环境**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

3. **上传代码到服务器**
   ```bash
   # 如果是Git仓库
   git clone git@github.com:xiyuan-qin/netApp.git ~/net_app
   # 或者直接上传本地文件
   scp -r /path/to/net_app user@your-server-ip:~
   ```

4. **在服务器上编译和运行**
   ```bash
   cd ~/net_app
   # 安装可能需要的依赖（以Ubuntu为例）
   sudo apt update
   sudo apt install -y build-essential pkg-config libssl-dev

   # 编译应用
   cargo build --release
   
   # 后台运行服务（保持在退出SSH后继续运行）
   nohup ./target/release/net_app > app.log 2>&1 &
   
   # 查看运行状态
   ps aux | grep net_app
   
   # 查看日志
   tail -f app.log
   ```

5. **服务管理（可选）**
   ```bash
   # 停止服务
   pkill -f "net_app"
   
   # 设置开机自启动（使用systemd，适用于大多数现代Linux发行版）
   sudo tee /etc/systemd/system/netapp.service > /dev/null <<EOT
   [Unit]
   Description=Net App Chat Server
   After=network.target

   [Service]
   Type=simple
   User=$(whoami)
   WorkingDirectory=$(realpath ~/net_app)
   ExecStart=$(realpath ~/net_app/target/release/net_app)
   Restart=on-failure
   RestartSec=5s

   [Install]
   WantedBy=multi-user.target
   EOT
   
   # 启用并启动服务
   sudo systemctl daemon-reload
   sudo systemctl enable netapp
   sudo systemctl start netapp
   ```

6. **分享访问地址**
   - 告知用户访问: `http://[你的服务器IP地址]:8080`
   - 或者如果你有域名: `http://your-domain.com:8080`
   - 如果想使用HTTPS，建议配置Nginx作为反向代理并使用Let's Encrypt免费证书

#### 选项B: 使用Docker容器部署

1. **创建Dockerfile**
   在项目根目录创建`Dockerfile`:
   ```dockerfile
   FROM rust:1.72 as builder
   WORKDIR /usr/src/net_app
   COPY . .
   RUN cargo build --release

   FROM debian:bullseye-slim
   WORKDIR /app
   COPY --from=builder /usr/src/net_app/target/release/net_app /app/
   COPY --from=builder /usr/src/net_app/static /app/static
   EXPOSE 8080
   CMD ["./net_app"]
   ```

2. **创建Docker镜像**
   ```bash
   docker build -t net_app .
   ```

3. **运行Docker容器**
   ```bash
   docker run -d -p 8080:8080 --name chat_app net_app
   ```

### 方法3: 编译为可执行文件分发（无需安装Rust）

对于不想安装Rust的用户，可以直接分发编译好的可执行文件。

1. **跨平台编译**
   - 对于Windows:
     ```bash
     rustup target add x86_64-pc-windows-gnu
     cargo build --release --target=x86_64-pc-windows-gnu
     ```
   - 对于macOS:
     ```bash
     rustup target add x86_64-apple-darwin
     cargo build --release --target=x86_64-apple-darwin
     ```
   - 对于Linux:
     ```bash
     rustup target add x86_64-unknown-linux-gnu
     cargo build --release --target=x86_64-unknown-linux-gnu
     ```

2. **创建分发包**
   - 将编译的可执行文件和static目录打包到一起
   - 示例结构:
     ```
     net_app-windows/
     ├── net_app.exe
     └── static/
         ├── index.html
         ├── css/
         └── js/
     ```

3. **分享给其他用户**
   - 用户只需解压文件并运行可执行文件
   - 确保他们知道如何开放防火墙端口

## 高级配置

### 修改端口号
如果需要修改默认的8080端口，编辑`src/main.rs`文件，找到以下行:
```rust
.bind("0.0.0.0:8080")?
```
将8080修改为您想要的端口，然后重新编译应用。

### 设置TLS/SSL（HTTPS）
要启用安全连接，需要进行以下修改:

1. 安装依赖:
   ```bash
   cargo add rustls
   cargo add rustls-pemfile
   ```

2. 获取SSL证书（可使用Let's Encrypt等服务）

3. 修改代码，启用TLS支持（需要额外的代码修改）

## 网络实验参考

本应用可用于以下计算机网络实验内容：

1. **WebSocket协议分析**：观察全双工通信的实现
2. **网络延迟测量**：使用ping命令测量实时延迟
3. **连接状态监控**：观察TCP连接的建立和维护
4. **子网通信模拟**：通过不同房间模拟子网间通信
5. **网络可靠性分析**：观察消息传递的可靠性机制
6. **网络流量分析**：使用浏览器开发工具分析网络通信

## 故障排除

### 常见问题:

1. **无法连接到服务器**
   - 检查服务器是否正在运行
   - 确认使用的是正确的IP地址和端口
   - 检查防火墙设置

2. **连接断开问题**
   - 应用有自动重连机制，通常会在5秒后尝试重新连接
   - 如果持续断开，可能是网络问题或服务器已关闭

3. **消息未送达**
   - 应用有消息确认机制，未确认的消息会在60秒后提示可能发送失败
   - 检查网络连接状态

4. **界面显示问题**
   - 如果消息不显示，请检查浏览器控制台是否有错误信息
   - 尝试刷新页面或清除浏览器缓存

## 技术栈

- **后端**: Rust, Actix Web, WebSocket
- **前端**: HTML5, CSS3, JavaScript
- **通信**: WebSocket (RFC 6455)
- **传输层**: TCP
- **序列化**: JSON

## 贡献

欢迎提交问题报告和改进建议。如果您想为项目做出贡献，请遵循以下步骤：

1. Fork 仓库
2. 创建您的功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交您的更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 打开一个 Pull Request

## 许可证

MIT许可证