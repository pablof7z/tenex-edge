use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub(super) enum MyAction {
    /// Declare your current broad work topic.
    Status(MyStatusArgs),
}

#[derive(Args)]
pub(super) struct MyStatusArgs {
    /// A rare, broad work topic of at most 15 words.
    #[arg(long, value_name = "TOPIC")]
    topic: String,
}

pub(super) fn my(action: MyAction) -> Result<()> {
    match action {
        MyAction::Status(args) => status(args),
    }
}

fn status(args: MyStatusArgs) -> Result<()> {
    let topic = crate::work_topic::normalize(&args.topic)?;
    crate::daemon::blocking::call(
        "my_status",
        crate::cli::rpc_params(serde_json::json!({ "topic": topic })),
    )?;
    println!("Visible work topic set: \"{topic}\" (automatic distillation paused for 30 minutes)");
    Ok(())
}
