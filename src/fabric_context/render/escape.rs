pub(super) fn esc_attr(input: &str) -> String {
    esc_text(input).replace('"', "&quot;")
}

pub(super) fn esc_text(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
