# Form-Messenger
Send submitted forms to predefined email address.
Using axum and lettre

Recommended to use with [Treafik Rate Limit](https://doc.traefik.io/traefik/reference/routing-configuration/http/middlewares/ratelimit/) to prevent to many mails.

## Environment Variables
| Env | Description | Example |
| ---- | ---- | ---- |
| ORIGINS | space separated list of allowed urls for cors | example.com test.org |
| SEND_TO | email to forward form entries to | example@gmail.com |
| SMTP_HOST | host url of your smtp provider | mail.example.com |
| SMTP_PASSWORD | smtp password | password123 |
| SMTP_USER | smtp user name (email) | forms@example.com |

## API
GET /health
	returns 200

POST /mail
	JSON body {subject: String, body: HTML}
	returns 200
	errors 400, 406 (JSON is broken)