use uuid::Uuid;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "skysold")]
pub(crate) struct Opt {
    #[structopt(short = "H", long)]
    pub hypixel_key: Uuid,
    #[structopt(short, long)]
    pub player: Uuid,
    #[structopt(short, long)]
    pub ifttt_key: String,
    #[structopt(short = "e", long, default_value = "nustify")]
    pub ifttt_event: String,
    #[structopt(short = "I", long, default_value = "20")]
    pub fetch_interval: u64,
    #[structopt(short = "m", long, default_value = "0")]
    pub min_price: u32,
}