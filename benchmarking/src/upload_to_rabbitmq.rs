use anyhow::Context;
use lapin::{
    options::BasicPublishOptions, BasicProperties, Channel, Connection, ConnectionProperties,
};
use pbr::ProgressBar;
use std::io::Stdout;
use std::sync::Arc;
use std::time::Duration;
use structopt::StructOpt;
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::Mutex;
use tokio::time::delay_for;
use uuid::Uuid;

mod utils;

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    address: String,
    #[structopt(short, long)]
    exchange: String,
    #[structopt(short, long)]
    queue: String,
    #[structopt(short, long)]
    schema_id: Uuid,
    #[structopt(short, long)]
    count: usize,
    #[structopt(short, long, default_value = "50")]
    window: usize,
}

struct HandleMessageContext {
    pub queue: String,
    pub exchange: String,
    pub channel: Channel,
    pub pb: Mutex<ProgressBar<Stdout>>,
    pub status_sender: Mutex<Sender<anyhow::Result<()>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::from_args();
    let pb = utils::create_progress_bar(args.count as u64);
    let (status_sender, mut status_receiver) = channel::<anyhow::Result<()>>(args.window);

    let conn = Connection::connect(&args.address, ConnectionProperties::default())
        .await
        .context("connection error")?;
    let channel = conn.create_channel().await?;

    let context = Arc::new(HandleMessageContext {
        queue: args.queue,
        exchange: args.exchange,
        channel,
        pb,
        status_sender: Mutex::new(status_sender),
    });

    let samples = utils::load_samples()?;
    let messages = utils::generate_messages(samples, args.schema_id);

    for (_object_id, data) in messages.take(args.count) {
        let context = context.clone();

        tokio::spawn(async move {
            let status = context
                .channel
                .basic_publish(
                    &context.exchange,
                    &context.queue,
                    BasicPublishOptions::default(),
                    data,
                    BasicProperties::default(),
                )
                .await;

            let result = match status.context("failed to deliver message") {
                Err(error) => Err(error),
                Ok(success) => success
                    .await
                    .context("failed to confirm message")
                    .map(|_| ()),
            };

            let mut sender = context.status_sender.lock().await;
            sender.send(result).await.ok();

            context.pb.lock().await.inc();
        });
    }

    for _ in 0..args.count {
        if let Some(Err(error)) = status_receiver.recv().await {
            return Err(error);
        }
    }

    // ensure delivery of all messages
    delay_for(Duration::from_secs(1)).await;

    context.pb.lock().await.finish_print("done");

    Ok(())
}
