use axum::{
	http::{HeaderValue, Method, StatusCode},
	response::IntoResponse,
	routing::post,
	Router
};
use lettre::{
	Message, AsyncSmtpTransport, Tokio1Executor, AsyncTransport,
	message::header::ContentType,
	transport::smtp::{
		Error,
		response::Response,
		authentication::Credentials
	}
};

use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

use dotenv;
use serde_json;
use http::header;
use tokio;


#[tokio::main]
async fn main() {
	dotenv::dotenv().ok();

	// let port = dotenv::var("PORT").unwrap_or("3000".to_owned());
	let origins_string = dotenv::var("ORIGINS").expect("Missing ORIGINS env var");
	let origins: Vec<HeaderValue> = origins_string.split_whitespace().filter_map(|item| HeaderValue::from_str(item).ok()).collect();

	let app = Router::new()
		.route("/form-mail", post(send))
		.layer(
			CorsLayer::new()
				.allow_origin(origins)
				.allow_headers([header::CONTENT_TYPE])
				.allow_methods([Method::POST])
		);

	// for docker use 0, 0, 0, 0
	// let addr = SocketAddr::from(([127, 0, 0, 1], port.parse::<u16>().unwrap()));
	let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
	let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
	axum::serve(listener, app).await.unwrap();
}

async fn send(body: String) -> impl IntoResponse {
	// parse JSON
	let json_body: serde_json::Value = serde_json::from_str(&body).expect("JSON is broken");
	let mail_subject;
	let mail;

	// generate mails from different contact froms
	if json_body["type"] == "contact" {
		let site = json_body["site"].as_str().unwrap_or("");
		let subject = json_body["subject"].as_str().unwrap_or("");
		let email = json_body["email"].as_str().unwrap_or("");
		let description = json_body["description"].as_str().unwrap_or("");

		mail_subject = format!("{} Contact ", site);
		mail = format!("{}\nEmail: {}\n\n{}", subject, email, description);
	}
	else if json_body["type"] == "code" {
		let subject = json_body["subject"].as_str().unwrap_or("");
		let metadata = json_body["metadata"].as_str().unwrap_or("");
		let code = json_body["code"].as_str().unwrap_or("");

		mail_subject = subject.to_string();
		mail = format!("{}\n\n{}", metadata, code);
	}
	else {
		mail_subject = "Unknown form".to_string();
		mail = json_body.to_string();
	}

	// send mail
	let result: Result<Response, Error> = send_mail(mail_subject, mail).await;

	return match result {
		Ok(_) => (StatusCode::OK, format!("Email sent: {:?}", result)),
		Err(e) => (StatusCode::NOT_FOUND, format!("Failed to send email: {:?}", e)),
	};
}

async fn send_mail(subject: String, mail: String) -> Result<Response, Error> {
	// Read .env
	let smpt_user = dotenv::var("SMTP_USER").expect("Missing SMTP_USER env var");
	let send_to = dotenv::var("SEND_TO").expect("Missing SEND_TO env var");
	let smtp_host = dotenv::var("SMTP_HOST").expect("Missing SMTP_HOST env var");
	let smtp_password = dotenv::var("SMTP_PASSWORD").expect("Missing SMTP_PASSWORD env var");

	// create email
	let email = Message::builder()
		.from(smpt_user.parse().unwrap())
		.to(send_to.parse().unwrap())
		.subject(&subject)
		.header(ContentType::TEXT_PLAIN)
		.body(mail)
		.unwrap();

	// send mail
	let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)
		.unwrap()
		.credentials(Credentials::new(smpt_user.to_owned(), smtp_password.to_owned()))
		.build();

	return mailer.send(email).await;
}