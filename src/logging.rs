use axum::{extract::Request, middleware::Next, response::Response};

pub async fn logger(request: Request, next: Next) -> Response {
    println!("Serving {}", request.uri().to_string());
    next.run(request).await
}
