use clap::Parser;
use mosaico::cli::{self, Cli};
use mosaico::command_forensics::CommandCallLog;

fn main() {
    let argv = std::env::args().collect::<Vec<_>>();
    // Harness callbacks are latency-sensitive and explicitly fail open. When
    // no daemon socket exists, return before Clap, Tokio, TLS, hook forensics,
    // or process discovery page in the full application. Require the complete
    // hook shape so malformed/manual invocations still receive normal errors.
    if inactive_hook_fast_path(&argv) {
        return;
    }

    // Select the crypto provider before the first TLS handshake.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let command_log = CommandCallLog::start(&argv);

    // `--help --all` (in any order) prints the full help with hidden subcommands
    // revealed. clap intercepts `--help` before our code runs, so detect it here.
    if argv.len() > 1
        && argv[1..].iter().any(|a| a == "--all")
        && argv[1..].iter().any(|a| a == "--help" || a == "-h")
    {
        cli::print_help_all();
        std::process::exit(0);
    }

    // Bare invocation and top-level `--help` / `-h` (without `--all`) print the
    // same context-sensitive help: operator commands (`who`, `agents`, `launch`)
    // are shown only outside an agent context. Internal/debug commands stay
    // hidden; use `--all` for those. Only intercept top-level help so subcommand
    // help (`mosaico who --help`, etc.) still goes through clap normally.
    if argv.len() == 1 || matches!(argv.get(1).map(String::as_str), Some("--help" | "-h")) {
        cli::print_help_contextual();
        std::process::exit(0);
    }

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
            eprintln!("mosaico: failed to start runtime: {e}");
            std::process::exit(1);
        }
    };
    let result = rt.block_on(cli::run(cli));
    command_log.finish_result(&result);
    if let Err(e) = result {
        eprintln!("mosaico: {e:#}");
        std::process::exit(1);
    }
}

fn inactive_hook_fast_path(argv: &[String]) -> bool {
    let is_hook = argv.get(1).map(String::as_str) == Some("harness")
        && argv.get(2).map(String::as_str) == Some("hook")
        && argv
            .get(3)
            .is_some_and(|host| !host.is_empty() && host != "help")
        && argv
            .windows(2)
            .any(|pair| pair[0] == "--type" && !pair[1].is_empty());
    is_hook && !mosaico::daemon::socket_path().exists()
}
