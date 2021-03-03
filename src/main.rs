mod options;

use std::error::Error;
use std::collections::HashSet;
use std::time::Duration;

use lib::auction::{Auction, Auctions};
use nustify::notification::Builder;

use options::Opt;
use structopt::StructOpt;

use tokio::time::sleep;
use num_format::{ToFormattedString, Locale};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();

    let mut previous =
        Auctions::current(&opt.hypixel_key, &opt.player).await?
            .filled()
            .min_price(opt.min_price);

    loop {
        sleep(Duration::from_secs(opt.fetch_interval)).await;

        let current = match Auctions::current(&opt.hypixel_key, &opt.player).await {
            Ok(auctions) => auctions,
            Err(err) => {
                eprintln!("cannot fetch current auctions: {:?}", err);
                continue;
            }
        }
            .filled()
            .min_price(opt.min_price);

        if let Some((body, icon)) = body_icon(previous.as_ref(), current.as_ref()) {
            let notification = Builder::new(body)
                .title("Hypixel | Skyblock | Auction House".to_owned())
                .image_url(icon)
                .build();
            match nustify::send(&notification, &opt.ifttt_event, &opt.ifttt_key).await {
                Err(err) => {
                    eprintln!("cannot send IFTTT notification: {:?}", err);
                },
                _ => ()
            }
        }

        previous = current;
    }
}

fn total_sold<'a>(auctions: impl Iterator<Item=&'a Auction>) -> String {
    auctions
        .map(|a| a.price as u64)
        .sum::<u64>()
        .to_formatted_string(&Locale::en)
}

fn body_icon(previous: &HashSet<Auction>, current: &HashSet<Auction>) -> Option<(String, String)> {
    let mut new = current.difference(&previous).collect::<Vec<_>>();
    if new.is_empty() {
        return None;
    }
    new.sort_by_key(|a| a.price);

    let (names, total) = if new.len() == 1 {
        (new[0].name.clone(), new[0].price.to_formatted_string(&Locale::en))
    } else {
        let (last, others) = new.split_last().unwrap();
        (
            format!("{} and {}", others.iter().map(|a| a.name.as_ref()).collect::<Vec<_>>().join(", "), last.name),
            format!("a total of {}", total_sold(new.iter().copied()))
        )
    };
    let mut body = format!("Your {} just sold for {} coins.", names, total);
    if current.len() > new.len() {
        body.push_str(
            &format!("\n\nYou have a total of {} coins to claim from {} filled auctions.", total_sold(current.iter()), current.len())
        );
    }

    Some((body, new.last().unwrap().icon_url()))
}

#[cfg(test)]
mod tests {
    use lib::auction::{Auction, AuctionType};
    use std::collections::HashSet;
    use std::iter::FromIterator;
    use uuid::Uuid;

    fn fake_auction(name: &str, id: &str, price: u32) -> Auction {
        Auction {
            id: Uuid::new_v3(&Uuid::NAMESPACE_URL, id.as_bytes()),
            name: name.to_owned(),
            item_id: id.to_owned(),
            auction_type: AuctionType::Bin,
            price,
            sold: true
        }
    }

    #[test]
    fn total_sold() {
        let auctions = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
            fake_auction("Healing Ring", "HEALING_RING", 120_000),
        ];
        assert_eq!(super::total_sold(auctions.iter()), "14,120,000");
    }

    #[test]
    fn body_icon() {
        let previous = HashSet::new();
        let current = HashSet::from_iter(
            vec![
                fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
            ]
        );
        let body_icon = super::body_icon(&previous, &current);
        assert!(body_icon.is_some());
        let (body, icon) = body_icon.unwrap();
        assert_eq!(body, "Your Ultimate Carrot Candy just sold for 14,000,000 coins.");
        assert_eq!(icon, "https://sky.shiiyu.moe/item/ULTIMATE_CARROT_CANDY");

        let previous = current;
        let current = HashSet::from_iter(
            vec![
                fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
                fake_auction("Healing Ring", "HEALING_RING", 120_000),
            ]
        );
        let body_icon = super::body_icon(&previous, &current);
        assert!(body_icon.is_some());
        let (body, icon) = body_icon.unwrap();
        assert_eq!(body, "Your Healing Ring just sold for 120,000 coins.\n\nYou have a total of 14,120,000 coins to claim from 2 filled auctions.");
        assert_eq!(icon, "https://sky.shiiyu.moe/item/HEALING_RING");

        let previous = HashSet::new();
        let current = HashSet::from_iter(
            vec![
                fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
                fake_auction("Healing Ring", "HEALING_RING", 120_000),
            ]
        );
        let body_icon = super::body_icon(&previous, &current);
        assert!(body_icon.is_some());
        let (body, icon) = body_icon.unwrap();
        assert_eq!(body, "Your Healing Ring and Ultimate Carrot Candy just sold for a total of 14,120,000 coins.");
        assert_eq!(icon, "https://sky.shiiyu.moe/item/ULTIMATE_CARROT_CANDY");

        let previous = current;
        let current = HashSet::from_iter(
            vec![
                fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
                fake_auction("Healing Ring", "HEALING_RING", 120_000),
            ]
        );
        let body_icon = super::body_icon(&previous, &current);
        assert!(body_icon.is_none());

        let current = HashSet::from_iter(
            vec![
                fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
                fake_auction("Healing Ring", "HEALING_RING", 120_000),
                fake_auction("Midas's Sword", "MIDAS_SWORD", 50_040_000),
                fake_auction("Shredder", "SHREDDER", 999_000),
            ]
        );
        let body_icon = super::body_icon(&previous, &current);
        assert!(body_icon.is_some());
        let (body, icon) = body_icon.unwrap();
        assert_eq!(body, "Your Shredder and Midas\'s Sword just sold for a total of 51,039,000 coins.\n\nYou have a total of 65,159,000 coins to claim from 4 filled auctions.");
        assert_eq!(icon, "https://sky.shiiyu.moe/item/MIDAS_SWORD");
    }
}
