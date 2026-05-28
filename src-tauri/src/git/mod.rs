use git2::{Repository, Status, StatusOptions, BranchType};
use serde::{Deserialize, Serialize};
use std::path::Path;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Git status for a repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub upstream: Option<String>,
    pub state: RepoState,
    pub ahead: usize,
    pub behind: usize,
    pub staged: usize,
    pub modified: usize,
    pub untracked: usize,
    pub conflicted: usize,
    pub last_commit: Option<CommitInfo>,
    pub remote_url: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RepoState {
    Clean,
    Dirty,
    Ahead,
    Behind,
    AheadAndBehind,
    Conflicted,
    Unknown,
}

/// Git service that monitors repositories
pub struct GitService {
    cache: Arc<RwLock<HashMap<String, GitStatus>>>,
    poll_interval_ms: u64,
}

impl GitService {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            poll_interval_ms: 2000, // Poll every 2 seconds
        }
    }

    /// Get git status for a directory
    pub fn get_status(&self, path: &str) -> Option<GitStatus> {
        // Check cache first
        {
            let cache = self.cache.read();
            if let Some(status) = cache.get(path) {
                return Some(status.clone());
            }
        }

        // Compute fresh status
        let status = self.compute_status(path)?;

        // Update cache
        {
            let mut cache = self.cache.write();
            cache.insert(path.to_string(), status.clone());
        }

        Some(status)
    }

    /// Force refresh git status
    pub fn refresh(&self, path: &str) -> Option<GitStatus> {
        let status = self.compute_status(path)?;
        let mut cache = self.cache.write();
        cache.insert(path.to_string(), status.clone());
        Some(status)
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write();
        cache.clear();
    }

    /// Compute git status from scratch
    fn compute_status(&self, path: &str) -> Option<GitStatus> {
        let repo = Repository::open(path).ok()?;

        let branch = self.get_current_branch(&repo);
        let upstream = self.get_upstream_branch(&repo);
        let (ahead, behind) = self.get_ahead_behind(&repo);
        let (staged, modified, untracked, conflicted) = self.get_file_counts(&repo);
        let last_commit = self.get_last_commit(&repo);
        let remote_url = self.get_remote_url(&repo);
        let tags = self.get_tags_for_head(&repo);

        let state = if conflicted > 0 {
            RepoState::Conflicted
        } else if ahead > 0 && behind > 0 {
            RepoState::AheadAndBehind
        } else if ahead > 0 {
            RepoState::Ahead
        } else if behind > 0 {
            RepoState::Behind
        } else if staged > 0 || modified > 0 || untracked > 0 {
            RepoState::Dirty
        } else {
            RepoState::Clean
        };

        Some(GitStatus {
            branch,
            upstream,
            state,
            ahead,
            behind,
            staged,
            modified,
            untracked,
            conflicted,
            last_commit,
            remote_url,
            tags,
        })
    }

    fn get_current_branch(&self, repo: &Repository) -> String {
        repo.head()
            .ok()
            .and_then(|head| head.shorthand().map(String::from))
            .unwrap_or_else(|| "detached HEAD".to_string())
    }

    fn get_upstream_branch(&self, repo: &Repository) -> Option<String> {
        let head = repo.head().ok()?;
        let branch = git2::Branch::wrap(head);
        let upstream = branch.upstream().ok()?;
        upstream.name().ok().flatten().map(String::from)
    }

    fn get_ahead_behind(&self, repo: &Repository) -> (usize, usize) {
        let head = match repo.head() {
            Ok(h) => h,
            Err(_) => return (0, 0),
        };

        let local_oid = match head.target() {
            Some(oid) => oid,
            None => return (0, 0),
        };

        let upstream_oid = match repo.revparse_ext("@{upstream}") {
            Ok((obj, _)) => obj.id(),
            Err(_) => return (0, 0),
        };

        repo.graph_ahead_behind(local_oid, upstream_oid)
            .unwrap_or((0, 0))
    }

    fn get_file_counts(&self, repo: &Repository) -> (usize, usize, usize, usize) {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false);

        let statuses = match repo.statuses(Some(&mut opts)) {
            Ok(s) => s,
            Err(_) => return (0, 0, 0, 0),
        };

        let mut staged = 0;
        let mut modified = 0;
        let mut untracked = 0;
        let mut conflicted = 0;

        for entry in statuses.iter() {
            let s = entry.status();
            if s.contains(Status::CONFLICTED) {
                conflicted += 1;
            }
            if s.contains(Status::INDEX_NEW)
                || s.contains(Status::INDEX_MODIFIED)
                || s.contains(Status::INDEX_DELETED)
                || s.contains(Status::INDEX_RENAMED)
                || s.contains(Status::INDEX_TYPECHANGE)
            {
                staged += 1;
            }
            if s.contains(Status::WT_MODIFIED)
                || s.contains(Status::WT_DELETED)
                || s.contains(Status::WT_RENAMED)
                || s.contains(Status::WT_TYPECHANGE)
            {
                modified += 1;
            }
            if s.contains(Status::WT_NEW) {
                untracked += 1;
            }
        }

        (staged, modified, untracked, conflicted)
    }

    fn get_last_commit(&self, repo: &Repository) -> Option<CommitInfo> {
        let head = repo.head().ok()?;
        let commit = head.peel_to_commit().ok()?;
        let oid = commit.id();
        let hash = oid.to_string();
        let short_hash = hash[..7].to_string();
        let message = commit.message()
            .unwrap_or("")
            .lines()
            .next()
            .unwrap_or("")
            .to_string();
        let author = commit.author().name()
            .unwrap_or("Unknown")
            .to_string();
        let timestamp = commit.time().seconds();

        Some(CommitInfo {
            hash,
            short_hash,
            message,
            author,
            timestamp,
        })
    }

    fn get_remote_url(&self, repo: &Repository) -> Option<String> {
        let remote = repo.find_remote("origin").ok()?;
        remote.url().map(String::from)
    }

    fn get_tags_for_head(&self, repo: &Repository) -> Vec<String> {
        let head = match repo.head().ok().and_then(|h| h.target()) {
            Some(oid) => oid,
            None => return Vec::new(),
        };

        let mut tags = Vec::new();
        if let Ok(tag_iter) = repo.tag_names(None) {
            for tag in tag_iter.iter().flatten() {
                if let Ok(obj) = repo.revparse_single(tag) {
                    if obj.id() == head {
                        tags.push(tag.to_string());
                    }
                }
            }
        }

        tags
    }

    /// Get branch list
    pub fn get_branches(&self, path: &str) -> Vec<BranchInfo> {
        let repo = match Repository::open(path) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        let mut branches = Vec::new();
        if let Ok(branch_iter) = repo.branches(Some(BranchType::Local)) {
            for branch_result in branch_iter {
                if let Ok((branch, _)) = branch_result {
                    if let Ok(name) = branch.name() {
                        if let Some(name) = name {
                            branches.push(BranchInfo {
                                name: name.to_string(),
                                is_current: branch.is_head(),
                                upstream: branch.upstream()
                                    .ok()
                                    .and_then(|u| u.name().ok().flatten())
                                    .map(String::from),
                            });
                        }
                    }
                }
            }
        }

        branches
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub upstream: Option<String>,
}

/// Start background git polling
pub fn start_git_poller(git_service: Arc<GitService>, paths: Vec<String>) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(
                git_service.poll_interval_ms,
            ))
            .await;

            for path in &paths {
                git_service.refresh(path);
            }
        }
    });
}
