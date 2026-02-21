use crate::output::OutputConfig;

pub fn print_pagination(count: usize, total: i64, next: Option<&str>, out: &OutputConfig) {
    if count == 0 {
        return;
    }
    match next {
        Some(token) if !token.is_empty() => {
            let _ = out.writeln(format_args!(
                "\nShowing {count} of {total} results. Use --next \"{token}\" for more."
            ));
        }
        _ => {
            let _ = out.writeln(format_args!("\nShowing {count} of {total} results."));
        }
    }
}
