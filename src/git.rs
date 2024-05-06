use std::path::Path;

use git2::{IndexAddOption, Repository, Signature};

pub fn open(path: &String) -> Repository {
    Repository::open(path).unwrap()
}

pub fn commit(repo: &Repository, sign: &Signature) {
    let obj = repo
        .head()
        .unwrap()
        .resolve()
        .unwrap()
        .peel(git2::ObjectType::Commit)
        .unwrap();
    let parent_commit = obj
        .into_commit()
        .map_err(|_| git2::Error::from_str("Couldn't find commit"))
        .unwrap();
    let mut index = repo.index().unwrap();
    index.add_all(["*"].iter(), IndexAddOption::all(), None);
    let oid = index.write_tree().unwrap();
    let tree = repo.find_tree(oid).unwrap();
    repo.commit(
        Some("HEAD"),
        &sign,
        &sign,
        "chore(append)",
        &tree,
        &[&parent_commit],
    )
    .unwrap();
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

pub fn _push(_repo: &Repository) {
    // repo.remot
    // repo.remote(name, url).unwrap().push(refspecs, opts)
}
