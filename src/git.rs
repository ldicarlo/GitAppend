use std::path::Path;

use git2::{
    Cred, Direction, FetchOptions, IndexAddOption, Oid, PushOptions, RemoteCallbacks, Repository,
    Signature,
};

pub fn open(path: &String) -> Repository {
    Repository::open(path).unwrap()
}

pub fn commit_and_push(
    repo: &Repository,
    credentials: Option<(String, String)>,
    sign: &Signature,
    files: Vec<String>,
) {
    let _oid = commit(repo, sign, files);
    push(repo, credentials);
}

fn commit(repo: &Repository, sign: &Signature, _files: Vec<String>) -> Oid {
    let parent_commit = repo
        .head()
        .unwrap()
        .resolve()
        .unwrap()
        .peel(git2::ObjectType::Commit)
        .unwrap()
        .into_commit()
        .unwrap();

    let mut index = repo.index().unwrap();
    index
        .add_all(["*"].iter(), IndexAddOption::FORCE, None)
        .unwrap();
    let oid = index.write_tree().unwrap();
    log::debug!("oid: {:?}", oid);
    index.write().unwrap();
    let tree = repo.find_tree(oid).unwrap();
    log::debug!("tree: {:?}", tree);
    repo.commit(
        Some("HEAD"),
        &sign,
        &sign,
        "chore(append)",
        &tree,
        &[&parent_commit],
    )
    .unwrap()
}

pub fn fetch(repo: &Repository, credentials: Option<(String, String)>, branch: String) {
    let mut remote = repo.find_remote("http-origin").unwrap();
    let mut fetch_options = FetchOptions::default();
    fetch_options.remote_callbacks(create_callbacks(credentials.clone()));
    log::debug!("{:?}", credentials);
    remote
        .connect_auth(Direction::Fetch, Some(create_callbacks(credentials)), None)
        .unwrap();
    repo.remote_add_fetch(
        "http-origin",
        &format!("refs/heads/{}:refs/heads/{}", branch, branch),
    )
    .unwrap();
    remote
        .fetch(&[&branch], Some(&mut fetch_options), None)
        .unwrap();
    repo.fetchhead_foreach(|name, _, _, merge| {
        log::debug!("{} : {}", name, merge);
        if merge {
            return true;
        }
        true
    })
    .unwrap()
}

// pub fn pull(repo: &Repository, credentials: Option<(String, String)>) {
//     fetch(repo, credentials);
//     //repo.f
// }

pub fn signature() -> Signature<'static> {
    Signature::now("Git-Append", "git-append@git").unwrap()
}

fn push(repo: &Repository, credentials: Option<(String, String)>) {
    let mut remote = repo.find_remote("http-origin").unwrap();
    log::debug!("URL: {:?}", remote.url());
    repo.remote_add_push("origin", "refs/heads/master:refs/heads/master")
        .unwrap();

    remote
        .connect_auth(
            Direction::Push,
            Some(create_callbacks(credentials.clone())),
            None,
        )
        .unwrap();
    repo.remote_add_push("origin", "refs/heads/master:refs/heads/master")
        .unwrap();
    let mut push_options = PushOptions::default();
    let callbacks = create_callbacks(credentials.clone());
    push_options.remote_callbacks(callbacks);

    remote
        .push(
            &["refs/heads/master:refs/heads/master"],
            Some(&mut push_options),
        )
        .unwrap();
}

fn create_callbacks<'a>(credentials: Option<(String, String)>) -> RemoteCallbacks<'a> {
    if let Some((username, token)) = credentials.clone() {
        create_callbacks_with_creds(username, token)
    } else {
        RemoteCallbacks::new()
    }
}

fn create_callbacks_with_creds<'a>(username: String, token: String) -> RemoteCallbacks<'a> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks
        .credentials(move |_str, _str_opt, _cred_type| Cred::userpass_plaintext(&username, &token));
    callbacks
}

pub fn get_blob_from_head(repo: &Repository, path: String, branch_name: String) -> Vec<u8> {
    let parent_commit = repo
        .find_branch(&branch_name, git2::BranchType::Remote)
        .unwrap();
    let head_commit = parent_commit.into_reference().peel_to_commit().unwrap();
    let path = Path::new(&path);
    let maybe_path = head_commit
        .as_object()
        .clone()
        .into_commit()
        .unwrap()
        .tree()
        .unwrap()
        .get_path(path.into())
        .ok();
    if let Some(tree) = maybe_path {
        tree.to_object(repo)
            .unwrap()
            .into_blob()
            .unwrap()
            .content()
            .into()
    } else {
        Vec::new()
    }
}
