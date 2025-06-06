mod auth;
mod config;
mod email;
mod error;
mod handlers;
mod middleware;
mod models;
mod routes;
mod test_utils;
mod utils;
mod validations;

use std::net::SocketAddr;

use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    Router,
};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, Level};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::{config::Config, routes::create_routes};

#[derive(OpenApi)]
#[openapi(
    paths(
        // Auth endpoints
        handlers::auth::register::register,
        handlers::auth::login::login,
        handlers::auth::logout::logout,
        handlers::auth::refresh_token::refresh_token,
        handlers::auth::google_auth::google_auth,
        handlers::auth::google_callback::google_callback,
        handlers::auth::change_password::change_password,
        handlers::auth::verify_email::verify_email,
        handlers::auth::verify_email::resend_verification,
        handlers::auth::request_password_reset::request_password_reset,
        handlers::auth::reset_password::reset_password,

        // Thread endpoints
        handlers::threads::list::get_threads,
        handlers::threads::create::create_thread,
        handlers::threads::detail::get_thread,
        handlers::threads::update::update_thread,
        handlers::threads::delete::delete_thread,

        // Comment endpoints
        handlers::comments::get_comments,
        handlers::comments::create_comment,
        handlers::comments::update_comment,
        handlers::comments::delete_comment,

        // User endpoints
        handlers::users::current_user::get_current_user,
        handlers::users::update_profile::update_profile,
        handlers::users::update_email::update_email,
        handlers::users::detail::get_user_by_username,
        handlers::users::delete::delete_user,
        handlers::users::threads::get_user_threads,
        handlers::users::comments::get_user_comments,
    ),
    components(
        schemas(
            // Auth DTOs
            models::auth::RegisterRequest,
            models::auth::LoginRequest,
            models::auth::LogoutResponse,
            models::auth::AuthResponse,
            models::auth::RefreshTokenRequest,
            models::auth::RequestPasswordResetRequest,
            models::auth::ResetPasswordRequest,
            models::auth::MessageResponse,

            // Thread DTOs
            models::threads::CreateThreadRequest,
            models::threads::UpdateThreadRequest,
            models::threads::ThreadResponse,
            models::threads::ThreadListResponse,
            models::threads::ThreadUser,
            models::common::PaginatedResponse<models::threads::ThreadResponse>,

            // Comment DTOs
            models::comments::CreateCommentRequest,
            models::comments::UpdateCommentRequest,
            models::comments::CommentResponse,
            models::comments::CommentUser,
            models::comments::CommentListResponse,

            // User DTOs
            models::users::UserResponse,
            models::users::PublicUserResponse,
            models::users::UpdateProfileRequest,

            // Common DTOs
            models::common::ErrorResponse,
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "threads", description = "Thread management"),
        (name = "comments", description = "Comment management"),
        (name = "users", description = "User management")
    ),
    info(
        title = "Reddit Clone API",
        version = "1.0.0",
        description = "A Reddit-like discussion platform API built with Rust and axum"
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe { backtrace_on_stack_overflow::enable() };

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // Load configuration
    let config = Config::from_env()?;

    info!("Starting Reddit Clone API server");
    info!("Database URL: {}", config.database_url);
    info!("Server will run on {}:{}", config.host, config.port);

    // Database connection
    let pool = sqlx::PgPool::connect(&config.database_url).await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    info!("Database connected and migrations applied");

    // Write OpenAPI documentation to file
    let openapi_json = serde_json::to_string_pretty(&ApiDoc::openapi())?;

    // Create directory if it doesn't exist
    let openapi_dir = std::path::Path::new("./static");
    if !openapi_dir.exists() {
        std::fs::create_dir_all(openapi_dir)?;
    }

    // Write the OpenAPI JSON to a file
    std::fs::write("./static/openapi.json", openapi_json)?;
    info!("📄 OpenAPI JSON file written to ./static/openapi.json");

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(config.cors_origin.parse::<HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .allow_credentials(true);

    let (router, _api) = OpenApiRouter::with_openapi(ApiDoc::openapi()).split_for_parts();

    // Serve static files
    let static_files_service = tower_http::services::ServeDir::new("./static");
    let static_files_router = Router::new().nest_service("/static", static_files_service);

    // Build the application router
    let router = router
        .merge(create_routes(pool.clone()))
        .merge(static_files_router)
        .merge(SwaggerUi::new("/swagger-ui").url("/static/openapi.json", ApiDoc::openapi()))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors),
        );

    // Server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    info!("🚀 Server starting on {}", addr);
    info!(
        "📚 OpenAPI docs available at http://{}:{}/swagger-ui/",
        config.host, config.port
    );

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router.into_make_service()).await?;

    Ok(())
}
