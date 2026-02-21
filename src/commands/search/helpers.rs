pub fn print_pagination(count: usize, total: i64, next: Option<&str>) {
    if count == 0 {
        return;
    }
    match next {
        Some(token) if !token.is_empty() => {
            println!("\nShowing {count} of {total} results. Use --next \"{token}\" for more.");
        }
        _ => {
            println!("\nShowing {count} of {total} results.");
        }
    }
}
