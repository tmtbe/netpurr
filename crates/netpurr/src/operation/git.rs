use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use futures_util::AsyncWriteExt;

use log::error;
use poll_promise::Promise;
use rustygit::Repository;
use rustygit::types::BranchName;

#[derive(Default, Clone)]
pub struct Git {}

impl Git {
    pub fn if_enable_git(&self, repo_path: &PathBuf) -> bool {
        match Repository::new(repo_path).cmd(["status"]) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
    pub fn enable_git(&self, repo_path: &PathBuf) {
        let repo = Repository::init(repo_path);
        if repo.is_err() {
            error!("init git repo failed, path: {:?}", repo_path);
        }
        let gitignore_content = ".DS_Store\nrequests/";
        let gitignore_path = repo_path.join("./.gitignore");
        let mut file = File::create(gitignore_path);
        if file.is_ok() {
            file.unwrap().write_all(gitignore_content.as_bytes());
        }
    }
    pub fn create_branch(
        &self,
        repo_path: &PathBuf,
        branch_name: String,
    ) -> rustygit::types::Result<()> {
        let repo = Repository::new(repo_path);
        let branch = BranchName::from_str(branch_name.as_str())?;
        repo.create_local_branch(&branch)?;
        repo.cmd(["commit", "--allow-empty", "-m", "Init Repo"])
    }

    pub fn update_remote(
        &self,
        repo_path: &PathBuf,
        remote_url: String,
    ) -> rustygit::types::Result<()> {
        let repo = Repository::new(repo_path);
        let remotes = repo.cmd_out(["remote"])?;
        let origin = "origin".to_string();
        if remotes.contains(&origin) {
            repo.cmd(["remote", "set-url", "origin", remote_url.as_str()])
        } else {
            repo.cmd(["remote", "add", "origin", remote_url.as_str()])
        }
    }

    pub fn git_sync_promise(&self, repo_path: PathBuf) -> Promise<rustygit::types::Result<()>> {
        Promise::spawn_thread("git_thread", move || -> rustygit::types::Result<()> {
            let repo = Repository::new(repo_path);
            if let Ok(head) = repo.cmd_out(["branch", "--show-current"]) {
                if let Some(branch_name) = head.get(0) {
                    repo.cmd([
                        "branch",
                        format!("--set-upstream-to=origin/{}", &branch_name).as_str(),
                    ])?;
                    repo.cmd(["add", "."])?;
                    repo.cmd(["rm", "-rf", "--ignore-unmatch","--cached", "requests/*"])?;
                    repo.commit_all("auto commit")?;
                    repo.cmd(["pull", "--rebase"])?;
                    repo.cmd(["push", "--set-upstream", "origin", &branch_name])
                } else {
                    Ok(())
                }
            } else {
                Ok(())
            }
        })
    }
    pub fn git_force_pull_promise(
        &self,
        repo_path: PathBuf,
    ) -> Promise<rustygit::types::Result<()>> {
        Promise::spawn_thread("git_thread", move || -> rustygit::types::Result<()> {
            let repo = Repository::new(repo_path);
            if let Ok(head) = repo.cmd_out(["branch", "--show-current"]) {
                if let Some(branch_name) = head.get(0) {
                    repo.cmd([
                        "reset",
                        "--hard",
                        format!("origin/{}", &branch_name).as_str(),
                    ])?;
                    repo.cmd(["fetch", "origin"])?;
                    repo.cmd(["pull", "origin", branch_name.as_str()])
                } else {
                    Ok(())
                }
            } else {
                Ok(())
            }
        })
    }
    pub fn git_force_push_promise(
        &self,
        repo_path: PathBuf,
    ) -> Promise<rustygit::types::Result<()>> {
        Promise::spawn_thread("git_thread", move || -> rustygit::types::Result<()> {
            let repo = Repository::new(repo_path);
            if let Ok(head) = repo.cmd_out(["branch", "--show-current"]) {
                if let Some(branch_name) = head.get(0) {
                    repo.cmd(["push", "--force", "origin", branch_name.as_str()])
                } else {
                    Ok(())
                }
            } else {
                Ok(())
            }
        })
    }
    pub fn switch_branch(&self, path: &PathBuf, branch_name: String) {
        let repo = Repository::new(path);
        if let Ok(branch) = BranchName::from_str(branch_name.as_str()) {
            repo.switch_branch(&branch);
        }
    }
}
