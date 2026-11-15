//! Playground video commands (Tauri wrappers).
//!
//! Core logic lives in `droidgear-core`.

use droidgear_core::video::{self, VideoModel, VideoTaskRecord, VideoTaskSnapshot};
use tauri::command;

/// Loads the video models configured for the Playground.
#[command]
#[specta::specta]
pub fn load_video_models() -> Result<Vec<VideoModel>, String> {
    video::load_video_models()
}

/// Persists the video models configured for the Playground.
#[command]
#[specta::specta]
pub fn save_video_models(models: Vec<VideoModel>) -> Result<(), String> {
    video::save_video_models(&models)
}

/// Loads persisted Playground video tasks.
#[command]
#[specta::specta]
pub fn load_video_tasks() -> Result<Vec<VideoTaskRecord>, String> {
    video::load_video_tasks()
}

/// Persists Playground video tasks.
#[command]
#[specta::specta]
pub fn save_video_tasks(tasks: Vec<VideoTaskRecord>) -> Result<(), String> {
    video::save_video_tasks(&tasks)
}

/// Submits a video generation task with the model's configured protocol.
#[command]
#[specta::specta]
pub async fn create_video_task(
    model: VideoModel,
    prompt: String,
    duration: Option<u32>,
    resolution: Option<String>,
    image_url: Option<String>,
    extra_json: String,
) -> Result<VideoTaskSnapshot, String> {
    video::create_video_task(
        &model,
        &prompt,
        duration,
        resolution.as_deref(),
        image_url.as_deref(),
        &extra_json,
    )
    .await
}

/// Queries a video generation task with the model's configured protocol.
#[command]
#[specta::specta]
pub async fn query_video_task(
    model: VideoModel,
    task_id: String,
) -> Result<VideoTaskSnapshot, String> {
    video::query_video_task(&model, &task_id).await
}
