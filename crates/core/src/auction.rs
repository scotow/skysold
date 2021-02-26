use isahc::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use uuid::Uuid;
use std::hash::{Hash, Hasher};
use nbt::from_gzip_reader;
use crate::error::Error::{self, *};
use crate::auction::AuctionType::Bin;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct Auction {
    pub id: Uuid,
    pub name: String,
    pub item_id: String,
    pub auction_type: AuctionType,
    pub price: u32,
    pub sold: bool,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum AuctionType {
    Auction,
    Bin,
}

impl Auction {
    fn from_raw(raw: AuctionRaw) -> Result<Self, Error> {
        let mut bytes = raw.tooltip.data.as_bytes();
        let mut b64_reader = base64::read::DecoderReader::new(
            &mut bytes,
            base64::STANDARD
        );

        #[derive(Debug, Deserialize)]
        struct Tooltip {
            #[serde(rename = "i")]
            info: Vec<Info>,
        }

        #[derive(Debug, Deserialize)]
        struct Info {
            tag: Tag,
        }

        #[derive(Debug, Deserialize)]
        struct Tag {
            #[serde(rename = "ExtraAttributes")]
            extra: ExtraAttributes,
        }

        #[derive(Debug, Deserialize)]
        struct ExtraAttributes {
            #[serde(rename = "id")]
            item_id: String,
        }

        let tooltip: Tooltip = from_gzip_reader(&mut b64_reader).map_err(|e| InvalidTooltip {
            source: Some(e.into()),
            id: raw.id,
            name: raw.name.clone(),
        })?;
        let item_id = tooltip.info.first().ok_or(InvalidTooltip {
            source: None,
            id: raw.id,
            name: raw.name.clone(),
        })?;

        let (auction_type, sold) = {
            if raw.is_bin {
                (Bin, raw.sold_price == raw.price)
            } else {
                let now =
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_err(|e| InvalidEndDate {
                            source: e.into(),
                            end: raw.end,
                        })?
                        .as_millis() as u64;
                (AuctionType::Auction, now > raw.end)
            }
        };

        Ok(
            Auction {
                id: raw.id,
                name: raw.name.clone(),
                item_id: item_id.tag.extra.item_id.clone(),
                auction_type,
                price: raw.price,
                sold,
            }
        )
    }

    pub fn icon_url(&self) -> String {
        format!("https://sky.shiiyu.moe/item/{}", self.item_id)
    }
}

impl PartialEq for Auction {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Auction {}

impl Hash for Auction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Deserialize)]
struct HypixelResponse {
    success: bool,
    auctions: Vec<AuctionRaw>
}

#[derive(Debug, Deserialize)]
struct AuctionRaw {
    #[serde(rename = "uuid")]
    id: Uuid,
    #[serde(rename = "item_name")]
    name: String,
    #[serde(rename = "bin")]
    is_bin: bool,
    #[serde(rename = "starting_bid")]
    price: u32,
    #[serde(rename = "highest_bid_amount")]
    sold_price: u32,
    #[serde(rename = "item_bytes")]
    tooltip: ItemBytes,
    end: u64,
}

#[derive(Debug, Deserialize)]
struct ItemBytes {
    data: String,
}

pub async fn currents(api_key: &Uuid, player: &Uuid) -> Result<HashSet<Auction>, Error> {
    let url = format!(
        "https://api.hypixel.net/skyblock/auction?key={:x}&player={:x}",
        api_key.to_hyphenated_ref(),
        player.to_simple_ref()
    );
    let mut response =
        isahc::get_async(url).await
            .map_err(|e| InvalidRequest { source: e.into() })?;
    let data = response.json::<HypixelResponse>().await
        .map_err(|e| Json { source: e.into() })?;

    if !data.success {
        return Err(InvalidApiStatus)
    }

    data.auctions.into_iter()
        .map(|a| Auction::from_raw(a))
        .collect()
}