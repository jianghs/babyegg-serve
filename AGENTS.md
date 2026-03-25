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

## 测试规范
- 使用 Rust 内置测试框架，异步场景用 `#[tokio::test]`。
- 集成测试放在 `apps/blog-api/tests/`，文件名按特性命名，如 `auth_rbac_claims.rs`。
- 测试名建议采用行为描述：`xxx_should_xxx`。
- 涉及数据库的测试需要本地 Postgres 可用并执行迁移。

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
