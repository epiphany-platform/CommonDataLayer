use anyhow::Context;
use pbr::ProgressBar;
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use std::io::Stdout;
use std::sync::Arc;
use std::time::Duration;
use structopt::StructOpt;
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::Mutex;
use tokio::time::sleep;
use uuid::Uuid;

mod utils;

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    brokers: String,
    #[structopt(short, long)]
    topic: String,
    #[structopt(short, long)]
    schema_id: Uuid,
    #[structopt(short, long)]
    count: usize,
    #[structopt(short, long, default_value = "50")]
    window: usize,
}

struct HandleMessageContext {
    pub topic: String,
    pub producer: FutureProducer,
    pub pb: Mutex<ProgressBar<Stdout>>,
    pub status_sender: Mutex<Sender<anyhow::Result<()>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::from_args();
    let pb = utils::create_progress_bar(args.count as u64);
    let (status_sender, mut status_receiver) = channel::<anyhow::Result<()>>(args.window);

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &args.brokers)
        .set("message.timeout.ms", "5000")
        .set("acks", "all")
        .set("compression.type", "none")
        .set("max.in.flight.requests.per.connection", "1")
        .create()
        .context("Producer creation error")?;

    let context = Arc::new(HandleMessageContext {
        topic: args.topic,
        producer,
        pb,
        status_sender: Mutex::new(status_sender),
    });

    let samples = utils::load_samples()?;
    let messages = utils::generate_messages(samples, args.schema_id);

    for (object_id, data) in messages.take(args.count) {
        let context = context.clone();

        tokio::spawn(async move {
            let key = object_id.to_string();
            let record = FutureRecord::to(&context.topic).payload(&data).key(&key);

            let timeout = Duration::from_secs(1);
            let delivery_status = context.producer.send(record, timeout).await;
            let result = delivery_status
                .map_err(|(err, _m)| err)
                .context("failed to send message")
                .map(|_| ());

            context.status_sender.lock().await.send(result).await.ok();
            context.pb.lock().await.inc();
        });
    }

    for _ in 0..args.count {
        if let Some(Err(error)) = status_receiver.recv().await {
            return Err(error);
        }
    }

    // ensure delivery of all messages
    sleep(Duration::from_secs(1)).await;

    context.pb.lock().await.finish_print("done");

    Ok(())
}
