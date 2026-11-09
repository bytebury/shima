//! Exercises the real HTTP path against a tiny local server.

use shima::prelude::*;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;

struct Captured {
    request_line: String,
    headers: Vec<String>,
    body: String,
}

/// Serves exactly one request with `status` and `body`, returning what it received.
fn serve_once(status: u16, body: &'static str) -> (String, mpsc::Receiver<Captured>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/v1", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut request_line = String::new();
        reader.read_line(&mut request_line).unwrap();

        let mut headers = Vec::new();
        let mut content_length = 0;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let line = line.trim_end().to_string();
            if line.is_empty() {
                break;
            }
            if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                content_length = v.trim().parse().unwrap();
            }
            headers.push(line);
        }
        let mut buf = vec![0; content_length];
        reader.read_exact(&mut buf).unwrap();

        let response = format!(
            "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
        tx.send(Captured {
            request_line: request_line.trim_end().to_string(),
            headers,
            body: String::from_utf8(buf).unwrap(),
        })
        .unwrap();
    });

    (url, rx)
}

fn decoded(body: &str) -> Vec<(String, String)> {
    body.split('&')
        .map(|pair| {
            let (k, v) = pair.split_once('=').unwrap();
            let decode = |s: &str| {
                let s = s.replace('+', " ");
                let mut out = Vec::new();
                let bytes = s.as_bytes();
                let mut i = 0;
                while i < bytes.len() {
                    if bytes[i] == b'%' {
                        out.push(u8::from_str_radix(&s[i + 1..i + 3], 16).unwrap());
                        i += 3;
                    } else {
                        out.push(bytes[i]);
                        i += 1;
                    }
                }
                String::from_utf8(out).unwrap()
            };
            (decode(k), decode(v))
        })
        .collect()
}

#[tokio::test]
async fn checkout_macro_sends_expected_request() {
    let (url, rx) = serve_once(200, r#"{"id":"cs_test_1","url":"https://checkout.stripe.com/c/1"}"#);
    let client = Client::new("sk_test_abc").with_base_url(url).with_api_version("2025-03-31.basil");
    let customer_id = String::from("cus_42");

    let session = checkout!(
        &client,
        &customer_id,
        "price_pro",
        "https://example.com/ok?session_id={CHECKOUT_SESSION_ID}",
        String::from("https://example.com/pricing"),
        trial_days = 7,
        metadata = [("team_id", 42.to_string())],
    )
    .unwrap();
    assert_eq!(session.id, "cs_test_1");

    let req = rx.recv().unwrap();
    assert_eq!(req.request_line, "POST /v1/checkout/sessions HTTP/1.1");
    // basic auth of "sk_test_abc:"
    assert!(req.headers.iter().any(|h| h == "authorization: Basic c2tfdGVzdF9hYmM6"), "{:?}", req.headers);
    assert!(req.headers.iter().any(|h| h == "stripe-version: 2025-03-31.basil"));

    let form = decoded(&req.body);
    let get = |k: &str| form.iter().find(|(key, _)| key == k).map(|(_, v)| v.as_str());
    assert_eq!(get("customer"), Some("cus_42"));
    assert_eq!(get("line_items[0][price]"), Some("price_pro"));
    assert_eq!(get("success_url"), Some("https://example.com/ok?session_id={CHECKOUT_SESSION_ID}"));
    assert_eq!(get("subscription_data[trial_period_days]"), Some("7"));
    assert_eq!(get("metadata[team_id]"), Some("42"));
}

#[tokio::test]
async fn invalid_ids_error_instead_of_panicking() {
    let client = Client::new("sk_test_abc").with_base_url("http://127.0.0.1:9");
    let err = checkout!(&client, "not_a_customer", "price_1", "https://s", "https://c").unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)), "{err}");

    let err = manage_subscriptions!(&client, "", "https://x").unwrap_err();
    assert!(matches!(err, Error::InvalidInput(_)), "{err}");
}

#[tokio::test]
async fn stripe_errors_are_parsed_with_status() {
    let (url, rx) = serve_once(
        404,
        r#"{"error":{"code":"resource_missing","message":"No such customer: 'cus_x'","param":"id","type":"invalid_request_error"}}"#,
    );
    let client = Client::new("sk_test_abc").with_base_url(url);

    let err = Customer::retrieve(&client, "cus_x").await.unwrap_err();
    assert!(err.is_not_found());
    let stripe = err.stripe_error().unwrap();
    assert_eq!(stripe.status, 404);
    assert_eq!(stripe.message, "No such customer: 'cus_x'");
    assert_eq!(rx.recv().unwrap().request_line, "GET /v1/customers/cus_x HTTP/1.1");
}

#[tokio::test]
async fn find_by_email_uses_query_string() {
    let (url, rx) = serve_once(
        200,
        r#"{"object":"list","has_more":false,"data":[{"id":"cus_1","email":"a+b@example.com","name":null,"description":null,"phone":null,"tax_exempt":null,"metadata":{}}]}"#,
    );
    let client = Client::new("sk_test_abc").with_base_url(url);

    let found = Customer::find_by_email(&client, "a+b@example.com").await.unwrap().unwrap();
    assert_eq!(found.id, "cus_1");
    assert_eq!(
        rx.recv().unwrap().request_line,
        "GET /v1/customers?email=a%2Bb%40example.com&limit=1 HTTP/1.1"
    );
}

#[tokio::test]
async fn report_usage_macro() {
    let (url, rx) = serve_once(200, r#"{"event_name":"tokens","identifier":"r1","timestamp":1,"payload":{"value":"10"}}"#);
    let client = Client::new("sk_test_abc").with_base_url(url);

    report_usage!(&client, "tokens", "cus_1", 10, identifier = "r1").unwrap();
    let req = rx.recv().unwrap();
    assert_eq!(req.request_line, "POST /v1/billing/meter_events HTTP/1.1");
    let form = decoded(&req.body);
    assert!(form.contains(&("payload[stripe_customer_id]".into(), "cus_1".into())));
}

#[test]
fn client_debug_redacts_secrets() {
    let client = Client::new("sk_live_supersecret").with_webhook_secret("whsec_supersecret");
    let debug = format!("{client:?}");
    assert!(!debug.contains("supersecret"), "{debug}");
}
