# Auth / Identity / RBAC 重构资产

## 背景
原先登录相关能力集中在 `blog-api/modules/auth`，同时承担了注册、登录、JWT、refresh token、RBAC claims 拼装和鉴权中间件等职责。`modules/user` 里又重复了一套用户创建和密码哈希逻辑，导致边界不清晰、重复较多，也不利于后续扩展。

本轮重构的目标不是拆成独立服务或共享 crate，而是在 `blog-api` 内把身份、认证、授权和测试边界理顺。

## 当前模块边界
### `modules/identity`
- 负责账号生命周期与身份输入规则。
- 包含用户实体/响应模型、用户创建、注册、密码哈希与凭证校验。
- `RegisterRequest` 和 `CreateUserRequest` 的统一领域输入下沉到 `identity::dto::CreateIdentityUser`。

### `modules/auth`
- 只负责认证会话流程。
- 包含 `login` / `refresh` / `logout`、JWT provider、认证中间件和 HTTP DTO/handler。
- 不再负责注册用户，也不再承载授权上下文类型。

### `modules/rbac`
- 负责授权主体、权限判断和 RBAC 约定。
- `AccessContext` 是认证成功后注入请求的授权上下文。
- `authorization` 只做 role/scope 判断。
- `keys` 集中维护角色和权限常量。
- `catalog` 集中维护当前 seed 约定的角色集合、权限集合和角色权限映射。

### `modules/user`
- 只保留用户资源接口与用户资料服务。
- `create_user` 复用 `identity` 内部能力，不再自己做密码哈希和邮箱唯一性校验。

## 关键约定
### 配置
- JWT 相关配置收口到 `AppConfig.auth: AuthConfig`。
- 业务层不再直接依赖根级散落的 JWT 配置字段。

### 数据访问
- `db/auth_repo.rs` 已重命名为 `db/session_repo.rs`，语义上只负责 refresh token/session 持久化。
- `db/rbac_repo.rs` 新增了只读查询接口，用于测试校验 seed 是否与代码约定一致。

### 授权
- `/users`、`/users/{id}`、`/users/me` 均经过认证中间件。
- `users:read` 用于读取用户资源。
- `users:write` 用于创建、更新、删除用户资源。
- `admin` 角色当前通过 `*` 权限获得全部访问能力。

## 测试资产
### `crates/app-testkit`
提供通用 HTTP 测试辅助：
- `get_text`
- `request_json`
- `request_json_with_auth`
- `request_empty_json_with_auth`

### `apps/blog-api/tests/support`
提供 blog-api 专属测试辅助：
- `test_config`
- `setup_app_lazy`
- `setup_app_with_db`
- `register_and_login`
- `refresh_session`
- `delete_users`

## 当前测试覆盖
- `login_and_refresh_should_issue_dynamic_claims_from_rbac`
- `users_routes_should_require_auth_and_enforce_scopes`
- `rbac_seed_should_match_code_keys`
- `health_endpoint_returns_ok`

这些测试共同覆盖了：
- 注册、登录、refresh token 轮换
- RBAC claims 动态刷新
- users 资源接口的读写权限
- RBAC seed 与代码常量/矩阵的一致性

## 后续扩展建议
- 新增权限或角色时，优先同步更新 `modules/rbac/keys.rs` 与 `modules/rbac/catalog.rs`，再修改 migration。
- 若后续出现第二个应用需要复用认证能力，再从当前 `identity/auth/rbac` 边界中抽共享 crate。
- 如果继续补集成测试，优先复用 `tests/support` 和 `app-testkit`，避免重新引入样板逻辑。
