use crate::error::Error::{self, *};
use crate::auction::AuctionType::Bin;

use std::hash::{Hash, Hasher};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;
use serde::Deserialize;
use reqwest::StatusCode;
use std::ops::Deref;
use std::iter::FromIterator;

#[derive(Clone, Debug, Default)]
pub struct Auctions(HashSet<Auction>);

impl Auctions {
    pub fn new(auctions: HashSet<Auction>) -> Self {
        Self(auctions)
    }

    pub fn empty() -> Self {
        Self(HashSet::new())
    }

    pub async fn current(api_key: &Uuid, player: &Uuid) -> Result<Self, Error> {
        let url = format!(
            "https://api.hypixel.net/skyblock/auction?key={:x}&player={:x}",
            api_key.to_hyphenated_ref(),
            player.to_simple_ref()
        );
        let response =
            reqwest::get(&url).await
                .map_err(|e| InvalidApiRequest { source: e.into() })?;
        if response.status() != StatusCode::OK {
            return Err(InvalidApiStatusCode { code: response.status().as_u16() })
        }

        let data =
            response.json::<HypixelResponse>().await
                .map_err(|e| InvalidApiResponse { source: e.into() })?;
        if !data.success {
            return Err(InvalidApiStatus);
        }

        Ok(
            Auctions(
                data.auctions.into_iter()
                    .map(|a| Auction::from_raw(a))
                    .collect::<Result<HashSet<_>, Error>>()?
            )
        )
    }

    pub fn filled(mut self) -> Self {
        self.0.retain(|a| a.sold);
        self
    }

    pub fn auction_type(mut self, auction_type: AuctionType) -> Self {
        self.0.retain(|a| a.auction_type == auction_type);
        self
    }

    pub fn min_price(mut self, price: u32) -> Self {
        self.0.retain(|a| a.price >= price);
        self
    }
}

impl Deref for Auctions {
    type Target = HashSet<Auction>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<Auction> for Auctions {
    fn from_iter<T: IntoIterator<Item=Auction>>(iter: T) -> Self {
        Self(HashSet::from_iter(iter))
    }
}

#[derive(Clone, Debug)]
pub struct Auction {
    pub id: Uuid,
    pub name: String,
    pub item_id: String,
    pub quantity: u8,
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

        #[derive(Deserialize)]
        struct Tooltip {
            #[serde(rename = "i")]
            info: Vec<Info>,
        }

        #[derive(Deserialize)]
        struct Info {
            tag: Tag,
            #[serde(rename = "Count")]
            count: u8,
        }

        #[derive(Deserialize)]
        struct Tag {
            #[serde(rename = "ExtraAttributes")]
            extra: ExtraAttributes,
        }

        #[derive(Deserialize)]
        struct ExtraAttributes {
            #[serde(rename = "id")]
            item_id: String,
        }

        let tooltip: Tooltip = nbt::from_gzip_reader(&mut b64_reader).map_err(|e| InvalidTooltip {
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
                name: raw.name,
                item_id: item_id.tag.extra.item_id.clone(),
                quantity: item_id.count,
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

#[derive(Deserialize)]
struct HypixelResponse {
    success: bool,
    auctions: Vec<AuctionRaw>,
}

#[derive(Deserialize)]
struct AuctionRaw {
    #[serde(rename = "uuid")]
    id: Uuid,
    #[serde(rename = "item_name")]
    name: String,
    #[serde(rename = "bin", default)]
    is_bin: bool,
    #[serde(rename = "starting_bid")]
    price: u32,
    #[serde(rename = "highest_bid_amount")]
    sold_price: u32,
    #[serde(rename = "item_bytes")]
    tooltip: ItemBytes,
    end: u64,
}

#[derive(Deserialize)]
struct ItemBytes {
    data: String,
}
