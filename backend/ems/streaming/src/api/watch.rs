use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use tracing::info;

use crate::app::{GetUserState, SaveUserState, SaveUserStateError};

use super::dto::{SaveUserStateRequest, UserStateQuery, UserStateResponse};
use super::state::AppState;

pub fn watch_routes(state: AppState) -> Router {
    Router::new()
        .route("/streamer/save-user-state", post(save_user_state))
        .route("/streamer/user-state", get(get_user_state))
        .with_state(state)
}

async fn save_user_state(
    State(state): State<AppState>,
    Json(body): Json<SaveUserStateRequest>,
) -> Result<Json<UserStateResponse>, (StatusCode, Json<Value>)> {
    info!(
        file_id = %body.file_id,
        viewer_id = %body.viewer_id,
        "POST /streamer/save-user-state"
    );
    let out = SaveUserState::new(state.watch.as_ref())
        .execute(body.into())
        .await
        .map_err(map_save_err)?;
    Ok(Json(UserStateResponse::from(out)))
}

async fn get_user_state(
    State(state): State<AppState>,
    Query(q): Query<UserStateQuery>,
) -> Result<Json<UserStateResponse>, (StatusCode, Json<Value>)> {
    info!(file_id = %q.file_id, viewer_id = %q.viewer_id, "GET /streamer/user-state");
    let Some(p) = GetUserState::new(state.watch.as_ref())
        .execute(q.viewer_id, q.file_id)
        .await
        .map_err(|e| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "error": e.to_string() })),
            )
        })?
    else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "no saved watch state" })),
        ));
    };
    Ok(Json(UserStateResponse::from(p)))
}

fn map_save_err(e: SaveUserStateError) -> (StatusCode, Json<Value>) {
    let status = match &e {
        SaveUserStateError::BadPosition | SaveUserStateError::BadDuration => {
            StatusCode::BAD_REQUEST
        }
        SaveUserStateError::Store(_) => StatusCode::SERVICE_UNAVAILABLE,
    };
    (status, Json(json!({ "error": e.to_string() })))
}
