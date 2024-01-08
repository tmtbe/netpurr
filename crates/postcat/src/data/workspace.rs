use std::path::PathBuf;

use rustygit::Repository;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Workspace {
    pub name: String,
    pub path: PathBuf,
    pub enable_git: Option<bool>,
    pub remote_url: Option<String>,
}

impl Workspace {
    pub fn if_enable_git(&mut self) -> bool {
        if let Some(git) = self.enable_git {
            return git;
        }
        let repo = Repository::new(self.path.clone());
        match repo.cmd(["status"]) {
            Ok(_) => {
                self.enable_git = Some(true);
                true
            }
            Err(_) => false,
        }
    }

    pub fn has_remote_url(&mut self) -> bool {
        if let Some(url) = &self.remote_url {
            return true;
        }
        let repo = Repository::new(self.path.clone());
        if let Ok(remote) = repo.cmd_out(["remote", "get-url", "origin"]) {
            if remote.len() > 0 {
                self.remote_url = Some(remote[0].clone());
                return true;
            } else {
                return false;
            }
        }
        false
    }
}
