pub(crate) fn snake_to_pascal(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;

    for ch in s.chars() {
        if ch == '_' {
            // underscore → mark next char for capitalization, skip underscore
            capitalize_next = true;
        } else if ch.is_ascii_alphanumeric() {
            if capitalize_next {
                // uppercase the char
                result.push(ch.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                // just push it (lowercase or original)
                result.push(ch.to_ascii_lowercase());
            }
        } else {
            // any other char (dash, space, punctuation) — treat as word-separator
            capitalize_next = true;
        }
    }

    result
}
