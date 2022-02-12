use clap::Parser;
use fetcher2::settings::DownloadSettings;
use fetcher2::template::nodes::node::NodeEvent;
use fetcher2::template::Template;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the settings file
    #[clap(short, long)]
    settings_path: PathBuf,

    /// Path to the template file
    #[clap(short, long)]
    template_path: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let settings_bytes = tokio::fs::read(args.settings_path).await?;
    let settings: Arc<DownloadSettings> = Arc::new(ron::de::from_bytes(&settings_bytes)?);
    let (mut template, rx) = Template::load(&args.template_path).await?;
    let printer = tokio::spawn(event_printer(rx));
    if let Ok(template) = template.prepare(settings.clone()).await {
        template.run_root(settings.clone()).await;
    } else {
        println!("Could not prepare template")
    }
    printer.await?;
    Ok(())
}

async fn event_printer(rx: Receiver<NodeEvent>) {
    todo!()
}
