use clap::Parser;
use tenex_edge::cli::{self, Cli};

fn main() {
    // rig-core's reqwest pulls a second rustls crypto provider (aws-lc-rs)
    // alongside nostr-sdk's (ring), so rustls 0.23 can't auto-pick one and panics
    // on the first TLS handshake. Install an explicit process-wide default.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let cli = Cli::parse();
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("tenex-edge: failed to start runtime: {e}");
            std::process::exit(1);
        }
    };
    if let Err(e) = rt.block_on(cli::run(cli)) {
        eprintln!("tenex-edge: {e:#}");
        std::process::exit(1);
    }
}
