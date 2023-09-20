use std::net::SocketAddr;

use async_trait::async_trait;
use axum::{
    error_handling::HandleErrorLayer, extract::FromRequestParts, response::IntoResponse,
    routing::get, BoxError, Router,
};
use http::{request::Parts, StatusCode};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_sessions::{time::Duration, MemoryStore, Session, SessionManagerLayer};

#[derive(Default, Deserialize, Serialize)]
struct Counter(usize);

#[async_trait]
impl<S> FromRequestParts<S> for Counter
where
    S: Send + Sync,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(req, state).await?;

        let count: usize = session
            .get("count")
            .expect("Could not deserialize.")
            .unwrap_or_default();

        session
            .insert("count", count + 1)
            .expect("Could not serialize.");

        Ok(Counter(count))
    }
}

#[tokio::main]
async fn main() {
    let session_store = MemoryStore::default();
    let session_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: BoxError| async {
            StatusCode::BAD_REQUEST
        }))
        .layer(
            SessionManagerLayer::new(session_store)
                .with_secure(false)
                .with_max_age(Duration::seconds(10)),
        );

    let app = Router::new()
        .route("/", get(handler))
        .layer(session_service);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(counter: Counter) -> impl IntoResponse {
    format!("Current count: {}", counter.0)
}