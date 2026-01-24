use std::{collections::HashSet, sync::LazyLock};

pub static KEYWORDS: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    let mut set = HashSet::new();
    set.insert("echo");
    set.insert("exit");
    set.insert("type");
    set
});
