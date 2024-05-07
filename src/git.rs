use std::path::Path;

use git2::{Direction, IndexAddOption, Oid, PushOptions, Refspecs, Repository, Signature};

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
    // println!("result: {:?}", res);
}

pub fn fetch(repo: &Repository) {
    repo.fetchhead_foreach(|name, _, _, _| {
        println!("{}", name);
        true
    })
    .unwrap()
}

pub fn signature() -> Signature<'static> {
    Signature::now("Git-Append", "git-append@git").unwrap()
}

fn push(repo: &Repository) {
    let mut remote = repo.find_remote("origin").unwrap();
    repo.remote_add_push("origin", "refs/heads/master:refs/heads/master")
        .unwrap();

    let mut remote_callbacks = git2::RemoteCallbacks::new();

    let mut push_options = PushOptions::default();
    push_options.remote_callbacks(remote_callbacks);

    println!("push url: {:?} {:?}", remote.name(), remote.pushurl(),);
    remote.connect(Direction::Push).unwrap();
    remote
        .push(
            &["refs/heads/master:refs/heads/master"],
            Some(&mut push_options),
        )
        .unwrap();
}
