# ZeroMQTT

[English](README.md) | 中文文档

一个强大的 ZeroMQ 和 MQTT 协议双向桥接系统，支持高性能、高可靠性的消息转发，配备现代化 Web 管理面板。

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)
![Status](https://img.shields.io/badge/status-beta-yellow.svg)

## 功能特性

- **双向桥接**: ZeroMQ（PUB/SUB、XPUB/XSUB）与 MQTT 协议之间的无缝消息转发
- **Web 管理面板**: 基于 Vue 3 的现代化仪表板，支持实时监控和配置管理
- **用户管理**: 多用户支持，密码使用 bcrypt 安全存储
- **RESTful API**: 完整的 HTTP API 配置管理接口
- **动态配置**: 热更新主题映射规则，无需重启服务
- **主题映射**: 灵活的主题转换规则，支持 MQTT 通配符（`+`、`#`）
- **自动重连**: MQTT 和 ZeroMQ 连接内置断线自动重连机制
- **SQLite 存储**: 持久化配置存储
- **Docker 支持**: 生产级 Docker 和 docker-compose 部署方案

## 系统架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        ZeroMQTT 桥接器                          │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    │
│  │ MQTT 客户端 │───▶│   桥接核心   │───▶│  ZMQ 发布者     │    │
│  │  (订阅者)   │    │              │    │  (PUB/XPUB)     │    │
│  └─────────────┘    │   主题映射   │    └─────────────────┘    │
│                     │              │                            │
│  ┌─────────────┐    │   统计收集   │    ┌─────────────────┐    │
│  │ MQTT 客户端 │◀───│              │◀───│  ZMQ 订阅者     │    │
│  │  (发布者)   │    └──────────────┘    │  (SUB/XSUB)     │    │
│  └─────────────┘                        └─────────────────┘    │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              Web 管理面板 (Vue 3 + Vite)                │   │
│  │  • 实时统计数据          • 配置管理                    │   │
│  │  • 连接状态监控          • 主题映射编辑器              │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## 快速开始

### 环境要求

- Rust 1.75+
- Node.js 18+（用于开发管理面板）
- MQTT Broker（或使用公共 Broker 如 `broker.emqx.io`）

### 编译运行

```bash
# 克隆仓库
git clone https://github.com/yourusername/zeromqtt.git
cd zeromqtt

# 编译项目
cargo build --release

# 运行桥接器
cargo run
```

启动后的服务端点：
- **Web 管理面板**: http://localhost:3000
- **API 接口**: http://localhost:3000/api
- **ZMQ PUB Socket**: tcp://*:5555
- **ZMQ SUB Socket**: tcp://localhost:5556

### Docker 部署

```bash
# 使用 docker-compose
docker-compose up -d

# 或手动构建
docker build -t zeromqtt .
docker run -p 3000:3000 -p 5555:5555 zeromqtt
```

## 配置说明

### 默认端点配置

| 类型 | Socket | 端点 |
|------|--------|------|
| MQTT | - | broker.emqx.io:1883 |
| ZMQ PUB | 绑定 | tcp://*:5555 |
| ZMQ SUB | 连接 | tcp://localhost:5556 |

### 主题映射配置

通过 Web 管理面板或 API 配置映射规则：

```json
{
  "source_endpoint_type": "mqtt",
  "source_endpoint_id": 1,
  "target_endpoint_type": "zmq",
  "target_endpoint_id": 3,
  "source_topic": "sensors/+/temperature",
  "target_topic": "zmq/sensors/temperature",
  "direction": "mqtt_to_zmq",
  "enabled": true
}
```

### 通配符支持

| 模式 | 说明 | 示例 |
|------|------|------|
| `+` | 单层通配符 | `sensors/+/temp` 匹配 `sensors/room1/temp` |
| `#` | 多层通配符 | `sensors/#` 匹配 `sensors/a/b/c` |

## API 参考

### 状态查询

```bash
# 获取桥接器状态
curl http://localhost:3000/api/status

# 获取消息统计
curl http://localhost:3000/api/status/stats
```

### 配置管理

```bash
# 列出 MQTT Broker
curl http://localhost:3000/api/config/mqtt

# 列出 ZMQ 端点
curl http://localhost:3000/api/config/zmq

# 列出主题映射
curl http://localhost:3000/api/config/mappings

# 添加新映射
curl -X POST http://localhost:3000/api/config/mappings \
  -H "Content-Type: application/json" \
  -d '{"source_endpoint_type":"mqtt","source_endpoint_id":1,...}'
```

### 桥接控制

```bash
# 启动桥接
curl -X POST http://localhost:3000/api/bridge/start

# 停止桥接
curl -X POST http://localhost:3000/api/bridge/stop

# 重启桥接
curl -X POST http://localhost:3000/api/bridge/restart
```

## 测试

### 单元测试

```bash
cargo test
```

### 端到端测试

E2E 测试套件验证实际消息流转：

```bash
# 终端 1: 启动桥接器
cargo run

# 终端 2: 运行 E2E 测试
cargo run --bin e2e_tests
```

E2E 测试覆盖：
- MQTT → ZMQ 消息桥接
- ZMQ → MQTT 消息桥接
- 双向桥接
- 热更新/动态配置
- 主题转换
- 启用/禁用映射
- 多映射并发

## 支持的 ZMQ 模式

| 模式 | 说明 | 使用场景 |
|------|------|----------|
| PUB/SUB | 发布-订阅 | 标准桥接场景 |
| XPUB/XSUB | 扩展发布-订阅 | 代理场景，支持订阅消息转发 |

## 项目结构

```
zeromqtt/
├── src/
│   ├── main.rs          # 应用入口
│   ├── lib.rs           # 库导出
│   ├── api/             # REST API 处理器
│   ├── bridge/          # 核心桥接逻辑
│   │   ├── core.rs      # 桥接编排
│   │   ├── worker.rs    # MQTT/ZMQ 工作线程
│   │   └── mapper.rs    # 主题映射
│   ├── db/              # 数据库层
│   ├── models/          # 数据结构
│   └── telemetry/       # 日志 & 指标
├── dashboard/           # Vue 3 Web 管理面板
├── tests/               # 集成测试 & E2E 测试
├── Dockerfile
└── docker-compose.yml
```

## 贡献

欢迎贡献代码！请随时提交 Pull Request。

## 许可证

MIT License - 详见 [LICENSE](LICENSE)

## 致谢

- [paho-mqtt](https://github.com/eclipse/paho.mqtt.rust) - MQTT 客户端
- [zmq](https://github.com/erickt/rust-zmq) - ZeroMQ 绑定
- [Axum](https://github.com/tokio-rs/axum) - Web 框架
- [Vue.js](https://vuejs.org/) - 前端框架
