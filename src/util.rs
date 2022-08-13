pub fn format_number(n:&f64)->String{
    let s = n.to_string();
    s.trim_end_matches(".0").to_string()
}
