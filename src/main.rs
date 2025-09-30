use axum::{
	http::{header, HeaderValue, Method, Request, StatusCode},
	middleware::{self, Next},
	response::{IntoResponse, Response},
	routing::{get, post},
	extract::State,
	body::Body,
	Router,
};
use lettre::{
	message::header::ContentType,
	transport::smtp::authentication::Credentials,
	Message,
	AsyncSmtpTransport,
	Tokio1Executor,
	AsyncTransport,
};

use std::{
	net::SocketAddr,
	env::var
};
use tower_http::cors::CorsLayer;

use serde_json;
use tokio;


#[tokio::main]
async fn main() {
	let origins_string = var("ORIGINS").expect("Missing ORIGINS env var");
	let origins: Vec<HeaderValue> = origins_string.split_whitespace().filter_map(|item| HeaderValue::from_str(item).ok()).collect();

	let app = Router::new()
		.route("/mail", post(send))
		.route_layer(middleware::from_fn_with_state(origins.clone(), require_origin))
		.layer(
			CorsLayer::new()
				.allow_origin(origins)
				.allow_headers([header::CONTENT_TYPE])
				.allow_methods([Method::POST])
		);

	let health =  Router::new().route("/health", get(health));


	let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
	let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
	axum::serve(listener, app.merge(health)).await.unwrap();
}

async fn require_origin(State(origins): State<Vec<HeaderValue>>, req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
	let origin = req.headers().get("origin");
	return match origin {
		Some(origin) if origins.contains(origin) => {
			Ok(next.run(req).await)
		}
		Some(origin) => { // else
			print!("{origin:?} tried to fetch");
			Err(StatusCode::FORBIDDEN)
		}
		None => {
			print!("Unknown Origin tried to fetch");
			Err(StatusCode::FORBIDDEN)
		}
	};
}

async fn send(body: String) -> Result<Response, Response> {
	// Read .env
	let smpt_user = var("SMTP_USER").expect("Missing SMTP_USER env var");
	let send_to = var("SEND_TO").expect("Missing SEND_TO env var");
	let smtp_host = var("SMTP_HOST").expect("Missing SMTP_HOST env var");
	let smtp_password = var("SMTP_PASSWORD").expect("Missing SMTP_PASSWORD env var");

	// parse JSON
	let json_body: serde_json::Value = serde_json::from_str(&body)
		.map_err(|e| {
			println!("Json: {e}");
			return (StatusCode::NOT_ACCEPTABLE, format!("Failed to read body: {e}")).into_response();
		})?;

	let subject = json_body["subject"].as_str().unwrap_or("");
	let body= json_body["body"].as_str().unwrap_or("");

	// create email
	let mail = Message::builder()
		.from(smpt_user.parse().unwrap())
		.to(send_to.parse().unwrap())
		.subject(subject)
		.header(ContentType::TEXT_HTML)
		.body(body.to_string())
		.map_err(|e| {
			println!("Create: {e}");
			return (StatusCode::BAD_REQUEST, format!("Failed to create email: {e}")).into_response();
		})?;

	// send mail
	let _ = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)
		.map_err(|e| {
			println!("SMTP: {e}");
			return (StatusCode::BAD_REQUEST, format!("Failed to send email: {e}")).into_response();
		})?
		.credentials(Credentials::new(smpt_user.to_owned(), smtp_password.to_owned()))
		.build()
		.send(mail)
		.await
		.map_err(|e| {
			println!("Send: {e}");
			return (StatusCode::BAD_REQUEST, format!("Failed to send email: {e}")).into_response();
		})?;

	return Ok((StatusCode::OK, format!("Email sent")).into_response());
}

async fn health() -> StatusCode {
	return StatusCode::OK;
}