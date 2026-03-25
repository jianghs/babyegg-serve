# rust-platform-template

一个可长期复用的 Rust 后端服务 workspace 模板，当前包含一个可运行的示例业务服务 `blog-api`。

## 1. 项目概览

本仓库拆分为三层：

- `crates/app-foundation`：跨服务复用的基础能力
- `crates/app-testkit`：测试辅助工具
- `apps/blog-api`：示例业务服务（用户 + 认证 + 外部 HTTP 调用）

主要目标：

- 统一服务初始化、日志、错误、响应结构
- 统一中间件与基础 Web 能力
- 通过 workspace 管理依赖与工程规范
- 让业务服务聚焦业务模块本身

## 2. 目录结构

```text
rust-platform-template/
├── crates/
│   ├── app-foundation/
│   └── app-testkit/
├── apps/
│   └── blog-api/
├── docker-compose.yml
├── Cargo.toml
└── README.md
```

## 3. 当前已实现能力

### 3.1 app-foundation

- 基础配置：`APP_HOST` / `APP_PORT` / `DEFAULT_LOCALE`
- 日志初始化：`tracing` + `RUST_LOG`
- 统一错误类型：`AppError`
- 统一成功响应：`ApiResponse<T>`
- 通用分页响应：`PageResponse<T>`
- 请求追踪中间件：自动注入/透传 `x-request-id`
- 健康检查接口：`GET /health`
- 基础国际化（`zh-CN` / `en-US`）

### 3.2 app-testkit

- `get_text`：快速对 Axum Router 发起请求并读取文本响应

### 3.3 blog-api（业务能力）

- 用户注册/登录（Argon2 密码哈希 + JWT）
- 用户 CRUD
- DTO 层统一参数校验入口（`validate()`）
- 当前登录用户接口（Bearer Token）
- 分页用户列表
- 外部 HTTP 调用示例（`/external/ip`）
- PostgreSQL 持久化（`users` 表）
- 启动自动 migration（可通过开发开关跳过）

## 4. API 清单

统一成功响应格式：

```json
{
  "code": 0,
  "message": "ok",
  "data": {}
}
```

统一错误响应格式：

```json
{
  "code": 400,
  "error_code": "INVALID_PARAM",
  "message": "name cannot be empty",
  "details": [
    {
      "field": "name",
      "reason": "required"
    }
  ]
}
```

说明：错误 `message` 会随 `DEFAULT_LOCALE` 切换为中文或英文。
`error_code` 为稳定机器码（示例：`USER_EMAIL_EXISTS`、`AUTH_INVALID_TOKEN`、`NOT_FOUND`）。
`details` 为可选字段，主要用于参数校验失败的结构化信息。

接口列表：

- `GET /health`：健康检查
- `POST /auth/register`：注册
- `POST /auth/login`：登录并返回 Token
- `GET /users/me`：获取当前登录用户（需 `Authorization: Bearer <token>`）
- `POST /users`：创建用户
- `GET /users?page=1&page_size=10&sort=created_at&order=desc&filter=...`：通用列表查询参数（当前业务使用 page/page_size，其他字段预留）
- `GET /users/{id}`：查询单个用户
- `PUT /users/{id}`：更新用户名
- `DELETE /users/{id}`：删除用户
- `GET /external/ip`：调用上游 HTTP 服务获取出口 IP

## 5. 国际化支持说明

当前支持两种语言：

- `zh-CN`
- `en-US`

`DEFAULT_LOCALE` 会影响：

- 启动与基础日志文案
- 业务校验错误文案
- 鉴权错误文案
- `NotFound` / `Internal` 错误返回文案

当前不支持按请求头动态切语言（例如 `Accept-Language`）；语言由服务启动配置统一决定。

## 6. 数据库

当前 migration 创建 `users` 表：

- `id UUID PRIMARY KEY`
- `name TEXT NOT NULL`
- `email TEXT NOT NULL UNIQUE`
- `password_hash TEXT NOT NULL`
- `created_at TIMESTAMPTZ NOT NULL`
- `updated_at TIMESTAMPTZ NOT NULL`

## 7. 快速开始

### 7.1 启动本地 PostgreSQL

```bash
docker compose up -d
```

### 7.2 准备环境变量

可选两种方式：

1. 在仓库根目录使用 `.env.example`：

```bash
cp .env.example .env
```

2. 在服务目录使用 `.env.example`：

```bash
cd apps/blog-api
cp .env.example .env
```

### 7.3 运行服务

```bash
cargo run -p blog-api
```

如果你在开发环境需要跳过自动 migration（例如本地库 migration 校验冲突），可使用：

```bash
SKIP_MIGRATIONS=true cargo run -p blog-api
```

说明：`SKIP_MIGRATIONS=true` 只跳过迁移步骤，不跳过数据库连接。

### 7.4 运行检查与测试

```bash
cargo check --workspace
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## 8. Makefile 命令

- `make up`：启动本地 PostgreSQL
- `make down`：停止本地 PostgreSQL
- `make check`：`cargo check --workspace`
- `make fmt`：`cargo fmt --all`
- `make clippy`：`cargo clippy ... -D warnings`
- `make test`：`cargo test --workspace`
- `make run-blog-api`：`cargo run -p blog-api`

## 9. 环境变量

根目录与 `apps/blog-api/.env.example` 共同涉及以下变量：

- `APP_HOST`：服务监听地址
- `APP_PORT`：服务监听端口
- `RUST_LOG`：日志过滤级别
- `DEFAULT_LOCALE`：默认语言（`zh-CN` / `en-US`）
- `DATABASE_URL`：PostgreSQL 连接串
- `HTTPBIN_BASE_URL`：外部 HTTP 示例服务地址
- `JWT_SECRET`：JWT 签名密钥（建议生产环境强随机）
- `JWT_EXPIRE_SECONDS`：JWT 过期时间（秒）
- `SKIP_MIGRATIONS`：是否跳过启动 migration（`true/1/yes/on`）

## 10. 当前验证状态

当前代码已通过：

- `cargo fmt --all`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`

## 11. 后续可增强方向

- 参数校验与统一校验错误结构
- OpenAPI/Swagger 文档
- 按请求动态国际化（`Accept-Language`）
- 认证授权扩展（角色/权限）
- repository trait 抽象与 mock 支持
- 更完整的 testkit（JSON helper、fixture、测试库隔离）
- 优雅停机与可观测性（metrics/tracing 细化）
