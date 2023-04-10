mod prompt;
mod cli;

use prompt::AccountPrompt;
use clap::Parser;
use cli::{ClArgs, PostCommand};
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::prompt::SelectAccountPrompt;

#[macro_export]
macro_rules! println_log_info {
    ($($arg:tt)+) => {
        println!($($arg)+);
        tracing::info!($($arg)+);
    };
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().ok();

    let cl_args = ClArgs::parse();

    set_up_logging()?;

    match cl_args.process_command().await? {
        PostCommand::Start(account_manager) => {
            println_log_info!("Starting wallet.");

            let select_account_prompt = SelectAccountPrompt { account_manager: &account_manager };
            while let Some(account_id) = select_account_prompt.run().await? {
                let prompt = AccountPrompt {
                    account_manager: &account_manager,
                    account_id,
                };
                prompt.run().await?;
            }
        }
        _ => {}
    }

    println_log_info!("Exiting wallet.");
    Ok(())
}

fn set_up_logging() -> eyre::Result<()> {
    std::panic::set_hook(Box::new(|p| {
        tracing::error!("{}", p);
    }));

    let registry = tracing_subscriber::registry();

    let registry = {
        registry
            .with(EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE))
    };

    registry.init();
    Ok(())
}
