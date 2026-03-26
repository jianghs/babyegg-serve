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
        .route_layer(auth_layer);

    Router::new()
        .route("/health", get(health::health))
        .route("/auth/register", post(modules::auth::handler::register))
        .route("/auth/login", post(modules::auth::handler::login))
        .route("/auth/refresh", post(modules::auth::handler::refresh))
        .route("/auth/logout", post(modules::auth::handler::logout))
        .route("/users/me", me_route)
        .route("/users", users_route)
        .route("/users/{id}", user_detail_route)
        .route("/external/ip", get(modules::external::handler::fetch_ip))
        .layer(middleware::from_fn(inject_request_id))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
