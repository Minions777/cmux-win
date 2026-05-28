use crate::git::{GitService, GitStatus, BranchInfo};
use std::sync::Arc;
use tauri::State;

/// Get git status for a directory
#[tauri::command]
pub async fn git_get_status(
    path: String,
    git_service: State<'_, Arc<GitService>>,
) -> Result<Option<GitStatus>, String> {
    Ok(git_service.get_status(&path))
}

/// Force refresh git status
#[tauri::command]
pub async fn git_refresh_status(
    path: String,
    git_service: State<'_, Arc<GitService>>,
) -> Result<Option<GitStatus>, String> {
    Ok(git_service.refresh(&path))
}

/// Get list of branches
#[tauri::command]
pub async fn git_get_branches(
    path: String,
    git_service: State<'_, Arc<GitService>>,
) -> Result<Vec<BranchInfo>, String> {
    Ok(git_service.get_branches(&path))
}

/// Clear git status cache
#[tauri::command]
pub async fn git_clear_cache(
    git_service: State<'_, Arc<GitService>>,
) -> Result<(), String> {
    git_service.clear_cache();
    Ok(())
}
