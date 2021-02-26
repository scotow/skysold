use std::collections::HashSet;
use crate::auction::Auction;
use crate::auction::AuctionType::Bin;

pub mod auction;
pub mod auction_ext;
pub mod error;

// fn it() {
//     let mut set = HashSet::new();
//     set.insert(Auction {
//         id: Default::default(),
//         name: "".to_string(),
//         item_id: "".to_string(),
//         price: 0,
//         auction_type: Bin,
//         sold: false
//     });
//
//     use auction_ext::AuctionExt;
//     set.iter().filled().collect::<Vec<_>>();
// }