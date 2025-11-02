/// Convert a snake_case string to PascalCase.
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

/// Convert a string to SCREAMING_SNAKE_CASE.
pub(crate) fn to_screaming_snake(s: &str) -> String {
    let mut result =
        String::with_capacity(s.len() + s.chars().filter(|c| c.is_uppercase()).count());
    let mut prev_was_lower = false;

    for ch in s.chars() {
        if ch.is_uppercase() && prev_was_lower && !result.is_empty() {
            result.push('_');
        }
        result.push(ch.to_ascii_uppercase());
        prev_was_lower = ch.is_lowercase();
    }

    result
}
