# Authix

一个基于 Rust 和 Axum 框架构建的高性能身份认证服务，支持多种登录方式、JWT 令牌管理和用户会话管理。

## 功能特性

- 🔐 **多种登录方式**：支持用户名密码、短信验证码、邮箱验证码登录
- 🎫 **JWT 令牌管理**：支持访问令牌和刷新令牌，自动令牌刷新
- 👥 **用户管理**：用户注册、用户信息查询、用户删除
- 📱 **会话管理**：基于 Redis 的用户会话存储和在线用户统计
- 🛡️ **安全特性**：Argon2 密码加密、输入验证、令牌过期管理
- 🌍 **国际化支持**：支持多租户架构
- 📊 **监控支持**：集成 Prometheus 监控指标

## 技术栈

- **Web 框架**: Axum 0.7
- **运行时**: Tokio
- **数据库**: MySQL (通过 SQLx)
- **缓存**: Redis (通过 deadpool-redis)
- **认证**: JWT + Argon2 密码加密
- **日志**: Tracing
- **配置**: dotenvy + config

## 快速开始

### 环境要求

- Rust 1.70+
- MySQL 8.0+
- Redis 6.0+

### 安装

1. 克隆项目：
```bash
git clone <repository-url>
cd authix
```

2. 安装依赖：
```bash
cargo build
```

3. 配置环境变量：
```bash
cp .env.example .env
# 编辑 .env 文件，配置数据库和 Redis 连接信息
```

4. 创建数据库表：
```sql
-- 创建用户表
CREATE TABLE users (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    tenant_id BIGINT NOT NULL DEFAULT 0,
    username VARCHAR(50) UNIQUE,
    phone VARCHAR(20) UNIQUE,
    email VARCHAR(100) UNIQUE,
    password VARCHAR(255) NOT NULL,
    last_login TIMESTAMP NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

5. 启动服务：
```bash
cargo run
```

服务将在 `http://localhost:3000` 启动。

## API 文档

### 认证相关

#### 用户登录
```http
POST /login
Content-Type: application/json

{
    "login_type": "password",  // "password" | "sms" | "email"
    "identifier": "username",  // 用户名/手机号/邮箱
    "credential": "password"   // 密码/验证码
}
```

#### 刷新令牌
```http
GET /token/refresh
Authorization: Bearer <refresh_token>
```

#### 获取令牌信息
```http
GET /token/get
X-Tenant-Id: <tenant_id>
X-Uid: <user_id>
```

#### 用户登出
```http
POST /logout
X-Uid: <user_id>
```

### 用户管理

#### 用户注册
```http
POST /register
Content-Type: application/json

{
    "register_type": "password",  // "password" | "sms" | "email"
    "identifier": "username",     // 用户名/手机号/邮箱
    "credential": "password"      // 密码
}
```

#### 验证验证码
```http
POST /verify_code
Content-Type: application/json

{
    "identifier": "phone_or_email",
    "credential": "verification_code"
}
```

#### 获取用户信息
```http
GET /user/profile
X-Uid: <user_id>
```

#### 删除用户
```http
DELETE /user/delete
X-Uid: <user_id>
```

### 在线用户管理

#### 获取在线用户数量
```http
GET /online/count
```

#### 获取在线用户列表
```http
GET /online/users?page=1&page_size=10
```

## 配置说明

### 环境变量

| 变量名 | 描述 | 默认值 |
|--------|------|--------|
| `DATABASE_URL` | MySQL 数据库连接字符串 | - |
| `REDIS_URL` | Redis 连接字符串 | - |
| `JWT_SECRET` | JWT 签名密钥 | - |
| `JWT_ACCESS_EXP` | 访问令牌过期时间（秒） | 3600 |
| `JWT_REFRESH_EXP` | 刷新令牌过期时间（秒） | 604800 |
| `SERVER_PORT` | 服务器端口 | 3000 |

### 数据库配置

确保 MySQL 数据库已创建并配置正确的连接字符串：
```
DATABASE_URL=mysql://username:password@localhost:3306/database_name
```

### Redis 配置

确保 Redis 服务运行并配置连接字符串：
```
REDIS_URL=redis://localhost:6379
```

## 开发指南

### 项目结构

```
src/
├── main.rs              # 应用入口
├── auth_handler.rs      # 认证处理器
├── cache.rs            # Redis 缓存操作
├── common.rs           # 通用结构和响应
├── errors.rs           # 错误定义
├── user.rs             # 用户相关功能
├── enums/              # 枚举定义
├── provider/           # 登录和注册提供者
│   ├── email.rs        # 邮箱登录/注册
│   ├── password.rs     # 密码登录/注册
│   ├── sms.rs          # 短信登录/注册
│   └── register.rs     # 注册服务
└── utils/              # 工具函数
    ├── database.rs     # 数据库连接
    ├── jwt.rs          # JWT 处理
    ├── redis.rs        # Redis 连接
    ├── regex.rs        # 正则验证
    └── uuid.rs         # UUID 生成
```

### 添加新的登录方式

1. 在 `src/provider/` 目录下创建新的提供者文件
2. 实现 `LoginProvider` 和 `RegisterProvider` trait
3. 在 `LoginService` 和 `RegisterService` 中注册新的提供者

### 自定义验证规则

在 `src/utils/regex.rs` 中修改验证函数：
- `is_valid_username()` - 用户名验证
- `is_valid_password()` - 密码验证
- `is_valid_phone()` - 手机号验证
- `is_valid_email()` - 邮箱验证

## 部署

### Docker 部署

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/authix /usr/local/bin/authix
EXPOSE 3000
CMD ["authix"]
```

### 生产环境配置

1. 设置环境变量
2. 配置反向代理（如 Nginx）
3. 设置 SSL 证书
4. 配置日志收集
5. 设置监控和告警

## 安全注意事项

- 确保 JWT 密钥足够复杂且保密
- 定期更新依赖包
- 使用 HTTPS 传输
- 配置适当的 CORS 策略
- 实施速率限制
- 定期备份数据库

## 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开 Pull Request

## 许可证

本项目采用 Apache 2.0 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 联系方式

如有问题或建议，请通过以下方式联系：

- 提交 Issue
- 发送邮件至 [your-email@example.com]

## 更新日志

### v0.1.0
- 初始版本发布
- 支持多种登录方式
- JWT 令牌管理
- 用户会话管理
- Redis 缓存支持
