macro_rules! path {
    ($($x:expr),+) => {{
        let mut path = ::std::path::PathBuf::new();
        $(
            path.push($x);
        )*
        path
    }}
}
