//! Escaping primitives used by the canonical serializer.

pub(super) fn attr(input: &str) -> String {
    text(input).replace('"', "&quot;").replace('\'', "&apos;")
}

pub(super) fn text(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
