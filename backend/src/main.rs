mod auth;
mod config;
mod error;
mod handlers;
mod middleware;
mod models;
mod routes;
mod utils;

use std::net::SocketAddr;

use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::{info, Level};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    config::Config,
    routes::create_routes,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        // Auth endpoints
        handlers::auth::register,
        handlers::auth::login,
        handlers::auth::logout,
        handlers::auth::refresh_token,
        handlers::auth::google_auth,
        handlers::auth::google_callback,
        
        // Thread endpoints
        handlers::threads::get_threads,
        handlers::threads::create_thread,
        handlers::threads::get_thread,
        handlers::threads::update_thread,
        handlers::threads::delete_thread,
        
        // Comment endpoints
        handlers::comments::get_comments,
        handlers::comments::create_comment,
        handlers::comments::update_comment,
        handlers::comments::delete_comment,
        
        // User endpoints
        handlers::users::get_current_user,
        handlers::users::update_profile,
    ),
    components(
        schemas(
            // Auth DTOs
            models::auth::RegisterRequest,
            models::auth::LoginRequest,
            models::auth::LogoutResponse,
            models::auth::AuthResponse,
            models::auth::RefreshTokenRequest,
            
            // Thread DTOs
            models::threads::CreateThreadRequest,
            models::threads::UpdateThreadRequest,
            models::threads::ThreadResponse,
            models::threads::ThreadListResponse,
            models::threads::ThreadUser,
            
            // Comment DTOs
            models::comments::CreateCommentRequest,
            models::comments::UpdateCommentRequest,
            models::comments::CommentResponse,
            models::comments::CommentUser,
            models::comments::CommentListResponse,
            
            // User DTOs
            models::users::UserResponse,
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

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(config.cors_origin.parse::<HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .allow_credentials(true);

    // Build the application router
    let app = Router::new()
        .merge(create_routes(pool.clone()))
        .merge(SwaggerUi::new("/swagger-ui").url("/api/openapi.json", ApiDoc::openapi()))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors)
        );

    // Server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    
    info!("🚀 Server starting on {}", addr);
    info!("📚 OpenAPI docs available at http://{}:{}/swagger-ui/", config.host, config.port);
    
    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
} 
