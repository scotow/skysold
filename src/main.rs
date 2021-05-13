mod options;

use std::error::Error;
use std::time::Duration;

use lib::auction::{Auction, Auctions};
use nustify::Builder;

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

        let current_filled = match Auctions::current(&opt.hypixel_key, &opt.player).await {
            Ok(auctions) => auctions,
            Err(err) => {
                eprintln!("cannot fetch current auctions: {:?}", err);
                continue;
            }
        }
            .filled();
        let current = current_filled.clone().min_price(opt.min_price);

        if let Some((body, icon)) = body_icon(&previous, &current, &current_filled) {
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

fn body_icon(previous: &Auctions, current: &Auctions, current_filled: &Auctions) -> Option<(String, String)> {
    let mut new =
        current
            .difference(previous)
            .collect::<Vec<_>>();
    if new.is_empty() {
        return None;
    }
    new.sort_by_key(|a| a.price);

    let (names, total) = if new.len() == 1 {
        (format_auction(new[0]), new[0].price.to_formatted_string(&Locale::en))
    } else {
        let (last, others) = new.split_last().unwrap();
        (
            format!("{} and {}", others.iter().map(|a| format_auction(a)).collect::<Vec<_>>().join(", "), last.name),
            format!("a total of {}", total_sold(new.iter().copied()))
        )
    };
    let mut body = format!("Your {} just sold for {} coins.", names, total);

    if current_filled.len() > new.len() {
        body.push_str(
            &format!(
                " You have a total of {} coins to claim from {} filled auctions.",
                total_sold(current_filled.iter()), current_filled.len()
            )
        );
    }

    Some((body, new.last().unwrap().icon_url()))
}

fn format_auction(auction: &Auction) -> String {
    if auction.quantity == 1 {
        auction.name.clone()
    } else {
        format!("{} x{}", auction.name, auction.quantity)
    }
}

#[cfg(test)]
mod tests {
    use lib::auction::{Auction, AuctionType, Auctions};
    use uuid::Uuid;

    fn fake_auction(name: &str, id: &str, price: u32) -> Auction {
        Auction {
            id: Uuid::new_v3(&Uuid::NAMESPACE_URL, id.as_bytes()),
            name: name.to_owned(),
            item_id: id.to_owned(),
            quantity: 1,
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
        // One new, no previous.
        let current = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000)
        ].into_iter().collect();
        let body_icon = super::body_icon(&Auctions::empty(), &current, &Auctions::empty());
        assert!(body_icon.is_some());
        let (body, icon) = body_icon.unwrap();
        assert_eq!(body, "Your Ultimate Carrot Candy just sold for 14,000,000 coins.");
        assert_eq!(icon, "https://sky.shiiyu.moe/item/ULTIMATE_CARROT_CANDY");

        // One new, one previous.
        let all = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
            fake_auction("Healing Ring", "HEALING_RING", 120_000),
        ].into_iter().collect();
        let previous = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000)
        ].into_iter().collect();
        let current = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
            fake_auction("Healing Ring", "HEALING_RING", 120_000),
        ].into_iter().collect();
        let body_icon = super::body_icon(&previous, &current, &all);
        assert!(body_icon.is_some());
        let (body, icon) = body_icon.unwrap();
        assert_eq!(body, "Your Healing Ring just sold for 120,000 coins. You have a total of 14,120,000 coins to claim from 2 filled auctions.");
        assert_eq!(icon, "https://sky.shiiyu.moe/item/HEALING_RING");

        // Two new, no previous.
        let all = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
            fake_auction("Healing Ring", "HEALING_RING", 120_000),
        ].into_iter().collect();
        let current = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
            fake_auction("Healing Ring", "HEALING_RING", 120_000),
        ].into_iter().collect();
        let body_icon = super::body_icon(&Auctions::empty(), &current, &all);
        assert!(body_icon.is_some());
        let (body, icon) = body_icon.unwrap();
        assert_eq!(body, "Your Healing Ring and Ultimate Carrot Candy just sold for a total of 14,120,000 coins.");
        assert_eq!(icon, "https://sky.shiiyu.moe/item/ULTIMATE_CARROT_CANDY");

        // Three new (one cheap), no previous.
        let all = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
            fake_auction("Healing Ring", "HEALING_RING", 120_000),
            fake_auction("Raider Axe", "RAIDER_AXE", 90_000),
        ].into_iter().collect();
        let current = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
            fake_auction("Healing Ring", "HEALING_RING", 120_000),
        ].into_iter().collect();
        let body_icon = super::body_icon(&Auctions::empty(), &current, &all);
        assert!(body_icon.is_some());
        let (body, icon) = body_icon.unwrap();
        assert_eq!(body, "Your Healing Ring and Ultimate Carrot Candy just sold for a total of 14,120,000 coins. You have a total of 14,210,000 coins to claim from 3 filled auctions.");
        assert_eq!(icon, "https://sky.shiiyu.moe/item/ULTIMATE_CARROT_CANDY");

        // One new, one previous (cheap).
        let all = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
            fake_auction("Raider Axe", "RAIDER_AXE", 90_000),
        ].into_iter().collect();
        let current = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
        ].into_iter().collect();
        let body_icon = super::body_icon(&Auctions::empty(), &current, &all);
        assert!(body_icon.is_some());
        let (body, icon) = body_icon.unwrap();
        assert_eq!(body, "Your Ultimate Carrot Candy just sold for 14,000,000 coins. You have a total of 14,090,000 coins to claim from 2 filled auctions.");
        assert_eq!(icon, "https://sky.shiiyu.moe/item/ULTIMATE_CARROT_CANDY");

        // One new (cheap), one previous.
        let all = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
            fake_auction("Raider Axe", "RAIDER_AXE", 90_000),
        ].into_iter().collect();
        let previous = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
        ].into_iter().collect();
        let current = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
        ].into_iter().collect();
        let body_icon = super::body_icon(&previous, &current, &all);
        assert!(body_icon.is_none());

        // No new, two previous.
        let all = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
            fake_auction("Healing Ring", "HEALING_RING", 120_000),
        ].into_iter().collect();
        let previous = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
            fake_auction("Healing Ring", "HEALING_RING", 120_000),
        ].into_iter().collect();
        let current = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
            fake_auction("Healing Ring", "HEALING_RING", 120_000),
        ].into_iter().collect();
        let body_icon = super::body_icon(&previous, &current, &all);
        assert!(body_icon.is_none());

        // Two new (cheaper first), two previous.
        let all = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
            fake_auction("Healing Ring", "HEALING_RING", 120_000),
            fake_auction("Midas's Sword", "MIDAS_SWORD", 50_040_000),
            fake_auction("Shredder", "SHREDDER", 999_000),
        ].into_iter().collect();
        let previous = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
            fake_auction("Healing Ring", "HEALING_RING", 120_000),
        ].into_iter().collect();
        let current = vec![
            fake_auction("Ultimate Carrot Candy", "ULTIMATE_CARROT_CANDY", 14_000_000),
            fake_auction("Healing Ring", "HEALING_RING", 120_000),
            fake_auction("Midas's Sword", "MIDAS_SWORD", 50_040_000),
            fake_auction("Shredder", "SHREDDER", 999_000),
        ].into_iter().collect();
        let body_icon = super::body_icon(&previous, &current, &all);
        assert!(body_icon.is_some());
        let (body, icon) = body_icon.unwrap();
        assert_eq!(body, "Your Shredder and Midas\'s Sword just sold for a total of 51,039,000 coins. You have a total of 65,159,000 coins to claim from 4 filled auctions.");
        assert_eq!(icon, "https://sky.shiiyu.moe/item/MIDAS_SWORD");
    }
}
