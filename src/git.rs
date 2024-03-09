use git2::Repository;

pub fn open(path: &String) -> Repository {
    Repository::open(path).unwrap()
}

pub fn commit() {}

pub fn fetch(repo: &Repository) {
    repo.fetchhead_foreach(|_, _, _, _| true).unwrap()
}
