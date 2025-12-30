pub fn format_f32(weight: f32) -> String {
    let s = format!("{weight:?}");
    let Some((int_part, frac_part)) = s
        .split_once('.')
        .filter(|(_int_part, frac_part)| frac_part.len() > 3)
    else {
        return s;
    };
    let (parts, tail) = frac_part.as_bytes().as_chunks::<3>();
    let mut result = String::from(int_part);
    result.push('.');
    let mut first = true;
    for part in parts {
        if !first {
            result.push('_');
        }
        result.push_str(str::from_utf8(part).unwrap());
        first = false;
    }
    if !tail.is_empty() {
        if !first {
            result.push('_');
        }
        result.push_str(str::from_utf8(tail).unwrap());
    }
    result
}
