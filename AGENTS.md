# Repository Guidelines

## 项目结构与模块组织
本仓库是 Rust workspace，核心目录如下：
- `apps/blog-api/`：可运行的 Web 服务（Axum），包含业务代码与数据库迁移。
- `crates/app-foundation/`：通用基础能力（错误模型、响应、i18n、中间件、查询参数等）。
- `crates/app-testkit/`：测试辅助工具。

`apps/blog-api/src/` 采用按领域分层：
- `modules/<domain>/`：`dto.rs`、`handler.rs`、`service.rs`、`model.rs`
- `db/*_repo.rs`：数据访问层
- `migrations/*.sql`：数据库迁移脚本

认证与授权相关边界约定：
- `modules/identity/`：身份域，负责注册、用户创建、密码哈希、凭证校验与用户模型。
- `modules/auth/`：认证会话域，负责 `login` / `refresh` / `logout`、JWT 与认证中间件。
- `modules/rbac/`：授权域，负责 `AccessContext`、权限判断、角色/权限常量与 RBAC 矩阵。
- `modules/user/`：用户资源接口，只处理用户资料与资源操作，不重复实现认证逻辑。
- `db/session_repo.rs`：只负责 refresh token/session 持久化，不承载业务判断。

## 构建、测试与开发命令
优先使用 Makefile：
- `make up` / `make down`：启动/停止本地 PostgreSQL（Docker Compose）
- `make fmt`：格式化全仓库（`cargo fmt --all`）
- `make check`：编译检查（`cargo check --workspace`）
- `make clippy`：静态检查，警告即失败
- `make test`：运行所有测试
- `make run-blog-api`：启动示例服务

开发时可跳过自动迁移：
- `SKIP_MIGRATIONS=true cargo run -p blog-api`

## 代码风格与命名规范
- 使用 `rustfmt` 默认风格（4 空格缩进）。
- 文件/函数使用 `snake_case`，结构体/枚举使用 `PascalCase`。
- 按模块边界组织代码，避免在 `handler` 写业务逻辑，业务逻辑放 `service`。
- 面向 API 的机器可读字段优先使用枚举，避免自由文本。
- 不要把注册、密码哈希、凭证校验重新塞回 `modules/auth` 或 `modules/user`，统一走 `modules/identity`。
- 不要在 handler 中直接写角色/权限判断，统一走 `modules/rbac/authorization.rs`。
- 不要在业务代码中散落角色/权限魔法字符串，统一使用 `modules/rbac/keys.rs` 与 `modules/rbac/catalog.rs`。

## 测试规范
- 使用 Rust 内置测试框架，异步场景用 `#[tokio::test]`。
- 集成测试放在 `apps/blog-api/tests/`，文件名按特性命名，如 `auth_rbac_claims.rs`。
- 测试名建议采用行为描述：`xxx_should_xxx`。
- 涉及数据库的测试需要本地 Postgres 可用并执行迁移。
- blog-api 集成测试优先复用 `apps/blog-api/tests/support/`。
- HTTP 请求测试优先复用 `crates/app-testkit/`，不要重复手写 JSON 请求、Bearer 请求和响应解析样板。
- 修改 RBAC seed、角色或权限模型后，至少运行 `cargo test -p blog-api --test auth_rbac_claims`。

## 提交与 PR 规范
提交信息遵循 Conventional Commits（与当前历史一致）：
- `feat(auth): ...`
- `refactor(validation): ...`
- `docs(readme): ...`

PR 建议包含：
- 变更目的与影响范围
- 是否涉及迁移/配置变更
- 本地验证结果（至少 `make fmt && make check && make test`）
- 接口变更时附请求/响应示例

## 安全与配置说明
- 不要提交真实密钥，使用本地 `.env`。
- 关键环境变量：`DATABASE_URL`、`JWT_SECRET`、`JWT_EXPIRE_SECONDS`、`JWT_REFRESH_EXPIRE_SECONDS`、`DEFAULT_LOCALE`。
- 生产环境不要使用 `SKIP_MIGRATIONS=true`。

## 认证与 RBAC 维护提示
- 新增角色或权限时，先同步更新 `apps/blog-api/src/modules/rbac/keys.rs` 与 `apps/blog-api/src/modules/rbac/catalog.rs`，再修改 migration/seed。
- 新增受保护资源接口时，默认显式接入认证中间件，并在 handler 中补齐 scope 校验。
- 更完整的重构背景与约定见 `apps/blog-api/docs/auth-rbac-refactor.md`。
