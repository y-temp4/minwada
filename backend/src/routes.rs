use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;

use crate::{handlers, middleware::auth_middleware};

pub fn create_routes(pool: PgPool) -> Router {
    Router::new()
        // API prefix
        .nest("/api", api_routes(pool))
}

fn api_routes(pool: PgPool) -> Router {
    Router::new()
        .nest("/auth", auth_routes(pool.clone()))
        .nest("/threads", thread_routes(pool.clone()))
        .nest("/comments", comment_routes(pool.clone()))
        .nest("/users", user_routes(pool.clone()))
        .with_state(pool)
}

fn auth_routes(pool: PgPool) -> Router<PgPool> {
    let auth_protected_routes = Router::new()
        .route("/change-password", post(handlers::auth::change_password))
        .route(
            "/resend-verification",
            post(handlers::auth::resend_verification),
        )
        .route_layer(middleware::from_fn_with_state(
            pool.clone(),
            auth_middleware,
        ));

    Router::new()
        .route("/register", post(handlers::auth::register))
        .route("/login", post(handlers::auth::login))
        .route("/logout", post(handlers::auth::logout))
        .route("/refresh", post(handlers::auth::refresh_token))
        .route("/google", get(handlers::auth::google_auth))
        .route("/google/callback", get(handlers::auth::google_callback))
        .route("/verify-email/{token}", post(handlers::auth::verify_email))
        .route("/password-reset/request", post(handlers::auth::request_password_reset))
        .route("/password-reset/{token}", post(handlers::auth::reset_password))
        .merge(auth_protected_routes)
}

fn thread_routes(pool: PgPool) -> Router<PgPool> {
    // 認証不要のルート
    let public_routes = Router::new()
        .route("/", get(handlers::threads::get_threads))
        .route("/{id}", get(handlers::threads::get_thread))
        .route(
            "/{thread_id}/comments",
            get(handlers::comments::get_comments),
        );

    // 認証が必要なルート
    let auth_routes = Router::new()
        .route("/", post(handlers::threads::create_thread))
        .route("/{id}", put(handlers::threads::update_thread))
        .route("/{id}", delete(handlers::threads::delete_thread))
        .route(
            "/{thread_id}/comments",
            post(handlers::comments::create_comment),
        )
        .route_layer(middleware::from_fn_with_state(
            pool.clone(),
            auth_middleware,
        ));

    // マージして返す
    public_routes.merge(auth_routes)
}

fn comment_routes(pool: PgPool) -> Router<PgPool> {
    Router::new()
        .route("/{id}", put(handlers::comments::update_comment))
        .route("/{id}", delete(handlers::comments::delete_comment))
        .route_layer(middleware::from_fn_with_state(
            pool.clone(),
            auth_middleware,
        ))
}

fn user_routes(pool: PgPool) -> Router<PgPool> {
    // 認証が必要なルート
    let auth_routes = Router::new()
        .route("/me", get(handlers::users::get_current_user))
        .route("/me", put(handlers::users::update_profile))
        .route("/me", delete(handlers::users::delete_user))
        .route("/me/email", put(handlers::users::update_email))
        .route_layer(middleware::from_fn_with_state(
            pool.clone(),
            auth_middleware,
        ));

    // 認証不要のルート
    let public_routes = Router::new()
        .route("/{username}", get(handlers::users::get_user_by_username))
        .route("/{user_id}/threads", get(handlers::users::get_user_threads))
        .route(
            "/{user_id}/comments",
            get(handlers::users::get_user_comments),
        );

    // マージして返す
    auth_routes.merge(public_routes)
}
