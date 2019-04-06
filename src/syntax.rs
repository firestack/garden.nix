/// Return true if `string` is an `$ exec` expression.
pub fn is_exec(string: &str) -> bool {
    string.starts_with("$ ")
}


/// Return true if `string` is a `:garden` expression.
pub fn is_garden(string: &str) -> bool {
    string.starts_with(":")
}


/// Return true if `string` is a `%group` expression.
pub fn is_group(string: &str) -> bool {
    string.starts_with("%")
}


/// Return true if `string` is a `@tree` expression.
pub fn is_tree(string: &str) -> bool {
    string.starts_with("@")
}


/// Return true if `string` is a `graft::value` expression.
pub fn is_graft(string: &str) -> bool {
    string.contains("::")
}


/// Trim garden, group, and tree prefixes
pub fn trim(string: &str) -> String {
    let mut value = string.to_string();
    value.remove(0);

    value
}


/// Trim the prefix from an exec expression
pub fn trim_exec(string: &str) -> String {
    let mut cmd = string.to_string();
    cmd.remove(0);
    cmd.remove(0);

    cmd
}


/// Safely a string into pre and post-split references
pub fn split_string<'a>(string: &'a str, split: &str)
-> (bool, &'a str, &'a str) {
    let end = string.len();
    let split_len = split.len();
    // split offset, everything up to this point is before the split
    let before = string.find(split).unwrap_or(end);

    let after;  // offset after the split
    let ok = before <= (end - split_len);
    if ok {
        after = before + split_len;
    } else {
        after = before;
    }

    (ok, &string[..before], &string[after..])
}
