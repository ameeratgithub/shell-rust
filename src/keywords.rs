use std::{collections::HashSet, sync::LazyLock};

pub static KEYWORDS: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    let mut set = HashSet::new();
    set.insert("echo");
    set.insert("exit");
    set.insert("pwd");
    set.insert("type");
    set.insert("cd");
    set.insert("history");
    set
});

pub static REDIRECTION_OPERATORS: LazyLock<HashSet<char>> = LazyLock::new(|| {
    let mut set = HashSet::new();
    set.insert('>');
    set.insert('<');
    set
});
