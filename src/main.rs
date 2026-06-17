use clap::Parser;
use tenex_edge::cli::{self, Cli};

fn main() {
    // rig-core's reqwest pulls a second rustls crypto provider (aws-lc-rs)
    // alongside nostr-sdk's (ring), so rustls 0.23 can't auto-pick one and panics
    // on the first TLS handshake. Install an explicit process-wide default.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let argv = std::env::args().collect::<Vec<_>>();
    let command_log = cli::command_forensics::CommandCallLog::start(&argv);
    let cli = match Cli::try_parse_from(argv.clone()) {
        Ok(cli) => cli,
        Err(err) => {
            command_log.finish_clap_error(&err);
            let code = err.exit_code();
            let _ = err.print();
            std::process::exit(code);
        }
    };
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            command_log.finish_runtime_error(&format!("failed to start runtime: {e}"));
            eprintln!("tenex-edge: failed to start runtime: {e}");
            std::process::exit(1);
        }
    };
    let result = rt.block_on(cli::run(cli));
    command_log.finish_result(&result);
    if let Err(e) = result {
        eprintln!("tenex-edge: {e:#}");
        std::process::exit(1);
    }
}
