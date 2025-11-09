use std::path::Path;

use git2::{
    Cred, DiffFormat, Direction, FetchOptions, Index, IndexAddOption, Oid, PushOptions,
    RemoteCallbacks, Repository, Signature,
};

pub fn open(path: &String) -> Repository {
    Repository::open(path).unwrap()
}

pub fn commit_and_push(repo: &Repository, credentials: Option<(String, String)>, sign: &Signature) {
    if let Some(_oid) = commit(repo, sign) {
        push(repo, credentials);
    }
}

fn commit(repo: &Repository, sign: &Signature) -> Option<Oid> {
    let parent_commit = repo
        .head()
        .unwrap()
        .resolve()
        .unwrap()
        .peel(git2::ObjectType::Commit)
        .unwrap()
        .into_commit()
        .unwrap();

    let mut index: Index = repo.index().unwrap();

    let _ = repo
        .diff_index_to_workdir(Some(&index), None)
        .unwrap()
        .print(DiffFormat::Raw, |d, h, l| {
            println!(
                "{:?} {:?} {:?}",
                d,
                h,
                String::from_utf8(l.content().to_vec()).unwrap()
            );
            true
        });

    index
        .add_all(["*"].iter(), IndexAddOption::FORCE, None)
        .unwrap();
    if index.is_empty() {
        None
    } else {
        let oid = index.write_tree().unwrap();
        println!("oid: {:?}", oid);
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
        .ok()
    }
}

fn fetch(
    repo: &Repository,
    credentials: Option<(String, String)>,
    branch: String,
) -> Option<git2::AnnotatedCommit> {
    let mut remote = repo.find_remote("http-origin").unwrap();
    let mut fetch_options = FetchOptions::default();
    fetch_options.remote_callbacks(create_callbacks(credentials.clone()));
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
        println!("{} : {}", name, merge);
        if merge {
            return true;
        }
        true
    })
    .unwrap();
    let fetch_head = repo.find_reference("FETCH_HEAD").unwrap();
    repo.reference_to_annotated_commit(&fetch_head).ok()
}

fn fast_forward(
    repo: &Repository,
    lb: &mut git2::Reference,
    rc: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
    println!("{}", msg);
    lb.set_target(rc.id(), &msg)?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default()
            // For some reason the force is required to make the working directory actually get updated
            // I suspect we should be adding some logic to handle dirty working directory states
            // but this is just an example so maybe not.
            .force(),
    ))?;
    Ok(())
}

pub fn pull(repo: &Repository, credentials: Option<(String, String)>, branch: String) {
    let fetch_commit = fetch(repo, credentials, branch.clone()).unwrap();
    let mut r = repo
        .find_reference(&format!("refs/heads/{}", branch))
        .unwrap();
    fast_forward(repo, &mut r, &fetch_commit).unwrap();
}

pub fn signature() -> Signature<'static> {
    Signature::now("Git-Append", "git-append@git").unwrap()
}

fn push(repo: &Repository, credentials: Option<(String, String)>) {
    let mut remote = repo.find_remote("http-origin").unwrap();
    println!("URL: {:?}", remote.url());
    // repo.remote_add_push("origin", "refs/heads/master:refs/heads/master")
    //     .unwrap();

    remote
        .connect_auth(
            Direction::Push,
            Some(create_callbacks(credentials.clone())),
            None,
        )
        .unwrap();
    // repo.remote_add_push("origin", "refs/heads/master:refs/heads/master")
    //     .unwrap();
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

pub fn get_blob_from_head(repo: &Repository, path: &String, branch_name: String) -> Vec<u8> {
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
