//! Playground image commands (Tauri wrappers).
//!
//! Core logic lives in `droidgear-core`.

use droidgear_core::image::{self, ImageModel, ImageTaskRecord, ImageTaskSnapshot};
use tauri::command;

/// Loads the image models configured for the Playground.
#[command]
#[specta::specta]
pub fn load_image_models() -> Result<Vec<ImageModel>, String> {
    image::load_image_models()
}

/// Persists the image models configured for the Playground.
#[command]
#[specta::specta]
pub fn save_image_models(models: Vec<ImageModel>) -> Result<(), String> {
    image::save_image_models(&models)
}

/// Loads persisted Playground image tasks.
#[command]
#[specta::specta]
pub fn load_image_tasks() -> Result<Vec<ImageTaskRecord>, String> {
    image::load_image_tasks()
}

/// Persists Playground image tasks.
#[command]
#[specta::specta]
pub fn save_image_tasks(tasks: Vec<ImageTaskRecord>) -> Result<(), String> {
    image::save_image_tasks(&tasks)
}

/// Generates images with the model's configured protocol.
#[command]
#[specta::specta]
#[allow(clippy::too_many_arguments)]
pub async fn create_image_task(
    model: ImageModel,
    task_id: String,
    prompt: String,
    size: String,
    quality: String,
    count: u32,
    image_urls: Vec<String>,
    extra_json: String,
) -> Result<ImageTaskSnapshot, String> {
    image::create_image_task(
        &model,
        &task_id,
        &prompt,
        &size,
        &quality,
        count,
        image_urls,
        &extra_json,
    )
    .await
}
