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

    let me_route = get(modules::user::handler::me).route_layer(middleware::from_fn_with_state(
        state.clone(),
        modules::auth::middleware::require_auth,
    ));

    Router::new()
        .route("/health", get(health::health))
        .route("/auth/register", post(modules::auth::handler::register))
        .route("/auth/login", post(modules::auth::handler::login))
        .route("/users/me", me_route)
        .route(
            "/users",
            post(modules::user::handler::create_user).get(modules::user::handler::list_users),
        )
        .route(
            "/users/{id}",
            get(modules::user::handler::get_user)
                .put(modules::user::handler::update_user)
                .delete(modules::user::handler::delete_user),
        )
        .route("/external/ip", get(modules::external::handler::fetch_ip))
        .layer(middleware::from_fn(inject_request_id))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
