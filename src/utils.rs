use warp::Filter;

pub fn with_clone<T: Clone + Send>(t: &T) -> impl Clone + Filter<Extract = (T, ), Error = std::convert::Infallible> {
    let t = t.clone();
    warp::any().map(move || t.clone())
}

pub fn decode_url(s: &str) -> String {
    let mut res = String::new();
    let mut iter = s.chars();
    while let Some(ch) = iter.next() {
        if ch == '%' {
            let code = format!("{}{}", iter.next().unwrap(), iter.next().unwrap());
            let code = [u8::from_str_radix(&code, 16).unwrap()];
            res.push_str(std::str::from_utf8(&code).unwrap());
        } else {
            res.push(ch);
        }
    }
    res
}

