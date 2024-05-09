use std::{env, path::Path};

use git2::{
    Cred, Direction, FetchOptions, IndexAddOption, Oid, PushOptions, Refspecs, RemoteCallbacks,
    Repository, Signature,
};

pub fn open(path: &String) -> Repository {
    Repository::open(path).unwrap()
}

pub fn commit_and_push(repo: &Repository, sign: &Signature, files: Vec<String>) {
    let _oid = commit(repo, sign, files);
    push(repo);
}

fn commit(repo: &Repository, sign: &Signature, files: Vec<String>) -> Oid {
    //  println!("{:?}", files);
    let parent_commit = repo
        .head()
        .unwrap()
        .resolve()
        .unwrap()
        .peel(git2::ObjectType::Commit)
        .unwrap()
        .into_commit()
        .unwrap();

    // println!("parent: {:?}", parent_commit);
    let mut index = repo.index().unwrap();
    index
        .add_all(["*"].iter(), IndexAddOption::FORCE, None)
        .unwrap();
    let oid = index.write_tree().unwrap();
    // println!("oid: {:?}", oid);
    index.write().unwrap();
    let tree = repo.find_tree(oid).unwrap();
    println!("tree: {:?}", tree);
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

pub fn fetch(repo: &Repository) {
    let mut remote = repo.find_remote("http-origin").unwrap();
    let mut fetch_options = FetchOptions::default();
    fetch_options.remote_callbacks(create_callbacks());
    remote
        .connect_auth(Direction::Fetch, Some(create_callbacks()), None)
        .unwrap();
    repo.remote_add_fetch("origin", "refs/heads/master:refs/heads/master")
        .unwrap();
    remote
        .fetch(&["master"], Some(&mut fetch_options), None)
        .unwrap();
    repo.fetchhead_foreach(|name, _, _, merge| {
        println!("{} : {}", name, merge);
        if merge {
            return true;
        }
        true
    })
    .unwrap()
}

pub fn pull(repo: &Repository) {
    fetch(repo);
    // repo.merge(annotated_commits, merge_opts, checkout_opts)
}

pub fn signature() -> Signature<'static> {
    Signature::now("Git-Append", "git-append@git").unwrap()
}

fn push(repo: &Repository) {
    let mut remote = repo.find_remote("http-origin").unwrap();
    println!("URL: {:?}", remote.url());
    repo.remote_add_push("origin", "refs/heads/master:refs/heads/master")
        .unwrap();

    remote
        .connect_auth(Direction::Push, Some(create_callbacks()), None)
        .unwrap();
    repo.remote_add_push("origin", "refs/heads/master:refs/heads/master")
        .unwrap();
    let mut push_options = PushOptions::default();
    let callbacks = create_callbacks();
    push_options.remote_callbacks(callbacks);

    remote
        .push(
            &["refs/heads/master:refs/heads/master"],
            Some(&mut push_options),
        )
        .unwrap();
}

fn create_callbacks<'a>() -> RemoteCallbacks<'a> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_str, _str_opt, _cred_type| Cred::userpass_plaintext("", ""));
    callbacks
}
