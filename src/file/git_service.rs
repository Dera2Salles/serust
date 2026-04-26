use crate::common::error::DomainError;
use git2::{Commit, Oid, Repository, Signature, Sort};
use std::path::Path;
use tracing::{debug, info};

pub struct GitService;

impl GitService {
    pub fn new() -> Self {
        Self
    }

    /// Initializes a Git repository in the user's storage root if it doesn't exist.
    pub fn init_repository(&self, user_path: &Path) -> Result<Repository, DomainError> {
        if user_path.join(".git").exists() {
            Repository::open(user_path).map_err(|e| DomainError::Internal(e.to_string()))
        } else {
            info!("Initializing new Git repository at {:?}", user_path);
            Repository::init(user_path).map_err(|e| DomainError::Internal(e.to_string()))
        }
    }

    /// Stages and commits a specific file.
    pub fn commit_file(
        &self,
        user_path: &Path,
        rel_path: &str,
        message: &str,
    ) -> Result<(), DomainError> {
        let repo = self.init_repository(user_path)?;
        let mut index = repo.index().map_err(|e| DomainError::Internal(e.to_string()))?;
        
        index.add_path(Path::new(rel_path)).map_err(|e| DomainError::Internal(e.to_string()))?;
        index.write().map_err(|e| DomainError::Internal(e.to_string()))?;

        let tree_id = index.write_tree().map_err(|e| DomainError::Internal(e.to_string()))?;
        let tree = repo.find_tree(tree_id).map_err(|e| DomainError::Internal(e.to_string()))?;

        let sig = Signature::now("AroSaina Server", "admin@arosaina.io")
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        let parent_commit = match repo.head() {
            Ok(head) => Some(head.peel_to_commit().map_err(|e| DomainError::Internal(e.to_string()))?),
            Err(_) => None,
        };

        let parents = match &parent_commit {
            Some(c) => vec![c],
            None => vec![],
        };

        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            message,
            &tree,
            &parents,
        ).map_err(|e| DomainError::Internal(e.to_string()))?;

        debug!("Git commit successful: {} for {}", message, rel_path);
        Ok(())
    }

    /// Returns a list of commits affecting the specified file.
    pub fn get_history(
        &self,
        user_path: &Path,
        rel_path: &str,
    ) -> Result<Vec<(String, i64, String)>, DomainError> {
        let repo = self.init_repository(user_path)?;
        let mut revwalk = repo.revwalk().map_err(|e| DomainError::Internal(e.to_string()))?;
        revwalk.set_sorting(Sort::TIME).map_err(|e| DomainError::Internal(e.to_string()))?;
        revwalk.push_head().map_err(|e| DomainError::Internal(e.to_string()))?;

        let mut history = Vec::new();
        for oid_result in revwalk {
            let oid = oid_result.map_err(|e| DomainError::Internal(e.to_string()))?;
            let commit = repo.find_commit(oid).map_err(|e| DomainError::Internal(e.to_string()))?;
            
            // Check if this commit affected the file
            if self.commit_affected_path(&repo, &commit, rel_path)? {
                history.push((
                    oid.to_string(),
                    commit.time().seconds(),
                    commit.message().unwrap_or("").to_string(),
                ));
            }
        }

        Ok(history)
    }

    fn commit_affected_path(
        &self,
        repo: &Repository,
        commit: &Commit,
        rel_path: &str,
    ) -> Result<bool, DomainError> {
        let tree = commit.tree().map_err(|e| DomainError::Internal(e.to_string()))?;
        
        if commit.parent_count() == 0 {
            // Initial commit - check if file exists in tree
            return Ok(tree.get_path(Path::new(rel_path)).is_ok());
        }

        for parent in commit.parents() {
            let parent_tree = parent.tree().map_err(|e| DomainError::Internal(e.to_string()))?;
            let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            
            let mut affected = false;
            diff.foreach(
                &mut |delta, _| {
                    if let Some(path) = delta.new_file().path() {
                        if path == Path::new(rel_path) {
                            affected = true;
                        }
                    }
                    if let Some(path) = delta.old_file().path() {
                        if path == Path::new(rel_path) {
                            affected = true;
                        }
                    }
                    true
                },
                None,
                None,
                None,
            ).map_err(|e| DomainError::Internal(e.to_string()))?;

            if affected {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Restores a file to a specific version from history.
    pub fn restore_version(
        &self,
        user_path: &Path,
        rel_path: &str,
        commit_hash: &str,
    ) -> Result<(), DomainError> {
        let repo = self.init_repository(user_path)?;
        let oid = Oid::from_str(commit_hash).map_err(|e| DomainError::Internal(e.to_string()))?;
        let commit = repo.find_commit(oid).map_err(|e| DomainError::Internal(e.to_string()))?;
        let tree = commit.tree().map_err(|e| DomainError::Internal(e.to_string()))?;
        
        let entry = tree.get_path(Path::new(rel_path))
            .map_err(|_| DomainError::FileNotFound)?;
        
        let object = entry.to_object(&repo).map_err(|e| DomainError::Internal(e.to_string()))?;
        let blob = object.as_blob().ok_or_else(|| DomainError::Internal("Not a blob".into()))?;

        std::fs::write(user_path.join(rel_path), blob.content())
            .map_err(|e| DomainError::Io(e))?;

        self.commit_file(user_path, rel_path, &format!("Restored version from {}", commit_hash))?;
        
        Ok(())
    }
}
