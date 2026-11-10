pub fn strip_comments(input: &str) -> String {
    let mut without_comments = String::with_capacity(input.len());
    let mut in_string = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
                without_comments.push(c);
            }
            continue;
        }

        if in_block_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if in_string {
            without_comments.push(c);
            if c == '\\' {
                if let Some(escaped) = chars.next() {
                    without_comments.push(escaped);
                }
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        if c == '"' {
            in_string = true;
            without_comments.push(c);
        } else if c == '/' && chars.peek() == Some(&'/') {
            chars.next();
            in_line_comment = true;
        } else if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            in_block_comment = true;
        } else {
            without_comments.push(c);
        }
    }
    without_comments
}

pub fn is_trailing_comma(chars: &[char], start: usize) -> bool {
    let mut j = start;
    while j < chars.len() && chars[j].is_whitespace() {
        j += 1;
    }
    j < chars.len() && (chars[j] == '}' || chars[j] == ']')
}

pub fn remove_trailing_commas(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut result = String::with_capacity(chars.len());
    let mut i = 0;
    let mut in_str = false;

    while i < chars.len() {
        let c = chars[i];
        if in_str {
            result.push(c);
            if c == '\\' && i + 1 < chars.len() {
                i += 1;
                result.push(chars[i]);
            } else if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }

        if c == '"' {
            in_str = true;
            result.push(c);
            i += 1;
            continue;
        }

        if c == ',' && is_trailing_comma(&chars, i + 1) {
            i += 1;
            continue;
        }

        result.push(c);
        i += 1;
    }

    result
}

pub fn strip_jsonc_and_trailing_commas(input: &str) -> String {
    let without_comments = strip_comments(input);
    remove_trailing_commas(&without_comments)
}
