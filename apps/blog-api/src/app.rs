use app_foundation::{middleware::request_id::inject_request_id, web::health};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{modules, state::AppState};

/// 构建业务服务的 HTTP 路由。
///
/// 当前路由装配约定：
/// - `/health` 与认证接口默认公开
/// - `/users/*` 默认挂载认证中间件
/// - `/external/ip` 作为外部服务调用示例，当前保持公开
///
/// 全局中间件顺序上会统一挂载 request id、HTTP trace 和 CORS。
pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let auth_layer =
        middleware::from_fn_with_state(state.clone(), modules::auth::middleware::require_auth);

    let me_route = get(modules::user::handler::me).route_layer(auth_layer.clone());
    let users_route = post(modules::user::handler::create_user)
        .get(modules::user::handler::list_users)
        .route_layer(auth_layer.clone());
    let user_detail_route = get(modules::user::handler::get_user)
        .put(modules::user::handler::update_user)
        .delete(modules::user::handler::delete_user)
        .route_layer(auth_layer.clone());
    let rbac_roles_route = get(modules::rbac::handler::list_roles).route_layer(auth_layer.clone());
    let rbac_permissions_route =
        get(modules::rbac::handler::list_permissions).route_layer(auth_layer.clone());
    let rbac_user_route =
        get(modules::rbac::handler::get_user_access).route_layer(auth_layer.clone());
    let rbac_user_roles_route =
        post(modules::rbac::handler::assign_user_role).route_layer(auth_layer.clone());

    Router::new()
        .route("/health", get(health::health))
        .route("/auth/register", post(modules::auth::handler::register))
        .route("/auth/login", post(modules::auth::handler::login))
        .route("/auth/refresh", post(modules::auth::handler::refresh))
        .route("/auth/logout", post(modules::auth::handler::logout))
        .route(
            "/auth/sessions",
            get(modules::auth::handler::list_sessions).route_layer(auth_layer.clone()),
        )
        .route(
            "/auth/sessions/revoke-all",
            post(modules::auth::handler::revoke_all_sessions).route_layer(auth_layer.clone()),
        )
        .route(
            "/auth/sessions/{id}",
            axum::routing::delete(modules::auth::handler::revoke_session)
                .route_layer(auth_layer.clone()),
        )
        .route("/users/me", me_route)
        .route("/users", users_route)
        .route("/users/{id}", user_detail_route)
        .route("/rbac/roles", rbac_roles_route)
        .route("/rbac/permissions", rbac_permissions_route)
        .route("/rbac/users/{id}", rbac_user_route)
        .route("/rbac/users/{id}/roles", rbac_user_roles_route)
        .route("/external/ip", get(modules::external::handler::fetch_ip))
        .layer(middleware::from_fn(inject_request_id))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
