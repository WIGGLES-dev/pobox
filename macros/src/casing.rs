pub fn pascal(ident: &str) -> String {
    ident
        .split("_")
        .map(|v| {
            let mut out = String::new();
            let mut chars = v.chars();
            if let Some(char) = chars.next() {
                out.push(char.to_ascii_uppercase());
            }
            out.extend(chars);
            out
        })
        .collect()
}

pub fn snake(ident: &str) -> String {
    let mut out = String::new();

    for (i, char) in ident.chars().enumerate() {
        if char.is_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.push(char.to_ascii_lowercase());
        } else {
            out.push(char);
        }
    }

    out
}
