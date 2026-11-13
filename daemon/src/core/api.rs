//! HTTP routes of `api/openapi.yaml`, version 0.

use std::convert::Infallible;
use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;
use tokio_stream::{Stream, StreamExt};

use crate::core::status::{ApStatus, ErrorBody, Section, Status, UplinkStatus};

/// Interval of the `: keep-alive` comment in the event stream.
const KEEP_ALIVE: Duration = Duration::from_secs(30);

/// Latest value of every section. Cloning is cheap; every request gets its own copy.
#[derive(Clone)]
pub struct Sections {
    pub ap: watch::Receiver<ApStatus>,
    pub uplink: watch::Receiver<UplinkStatus>,
}

pub fn router(sections: Sections) -> Router {
    Router::new()
        .route("/v0/status", get(get_status))
        .route("/v0/events", get(get_events))
        .fallback(not_found)
        .method_not_allowed_fallback(method_not_allowed)
        .with_state(sections)
}

async fn get_status(State(sections): State<Sections>) -> Json<Status> {
    Json(Status {
        ap: sections.ap.borrow().clone(),
        uplink: sections.uplink.borrow().clone(),
    })
}

/// Server-Sent Events. `WatchStream::new` yields the current value first and then every
/// change, so a client gets the whole state right after connecting.
async fn get_events(
    State(sections): State<Sections>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let ap = WatchStream::new(sections.ap).map(to_event);
    let uplink = WatchStream::new(sections.uplink).map(to_event);
    Sse::new(ap.merge(uplink)).keep_alive(KeepAlive::new().interval(KEEP_ALIVE).text(" keep-alive"))
}

fn to_event<T: Section>(section: T) -> Result<Event, Infallible> {
    let event = Event::default()
        .event(T::NAME)
        .json_data(&section)
        .unwrap_or_else(|error| {
            tracing::error!(section = T::NAME, %error, "could not serialize section");
            Event::default().comment(" serialization failed")
        });
    Ok(event)
}

async fn not_found() -> Response {
    error(StatusCode::NOT_FOUND, "not_found", "No such endpoint")
}

async fn method_not_allowed() -> Response {
    error(
        StatusCode::METHOD_NOT_ALLOWED,
        "method_not_allowed",
        "Method not allowed for this endpoint",
    )
}

fn error(status: StatusCode, code: &'static str, message: &'static str) -> Response {
    (status, Json(ErrorBody { code, message })).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::reporter::channel;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn test_router() -> (Router, crate::core::reporter::Reporter<ApStatus>) {
        let (ap_reporter, ap) = channel::<ApStatus>();
        let (_uplink_reporter, uplink) = channel::<UplinkStatus>();
        (router(Sections { ap, uplink }), ap_reporter)
    }

    async fn body_json(response: Response) -> serde_json::Value {
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn status_shows_reported_values() {
        let (app, ap_reporter) = test_router();
        ap_reporter.set(ApStatus {
            running: true,
            ..ApStatus::initial()
        });

        let request = Request::get("/v0/status").body(Body::empty()).unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = body_json(response).await;
        assert_eq!(body["ap"]["running"], true);
        assert_eq!(body["uplink"]["state"], "unavailable");
    }

    #[tokio::test]
    async fn unknown_path_gives_error_body() {
        let (app, _) = test_router();
        let request = Request::get("/v0/nothing").body(Body::empty()).unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(body_json(response).await["code"], "not_found");
    }

    #[tokio::test]
    async fn wrong_method_gives_error_body() {
        let (app, _) = test_router();
        let request = Request::post("/v0/status").body(Body::empty()).unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(body_json(response).await["code"], "method_not_allowed");
    }

    #[tokio::test]
    async fn events_start_with_every_section() {
        let (app, _) = test_router();
        let request = Request::get("/v0/events").body(Body::empty()).unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.headers()["content-type"], "text/event-stream");

        let mut body = response.into_body();
        let mut text = String::new();
        while !(text.contains("event: ap") && text.contains("event: uplink")) {
            let frame = body.frame().await.unwrap().unwrap();
            text.push_str(std::str::from_utf8(frame.data_ref().unwrap()).unwrap());
        }
        assert!(text.contains(r#""state":"unavailable""#));
    }
}
