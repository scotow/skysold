use crate::auction::{self, Auction};

pub trait AuctionExt: Iterator {
    fn filled(self) -> Filled<Self>
        where Self: Sized
    {
        Filled { iter: self }
    }

    fn auction_type(self, auction_type: auction::AuctionType) -> AuctionType<Self>
        where Self: Sized
    {
        AuctionType {
            iter: self,
            auction_type,
        }
    }

    fn min_price(self, price: u32) -> MinPrice<Self>
        where Self: Sized
    {
        MinPrice {
            iter: self,
            price,
        }
    }
}

impl<'a, T> AuctionExt for T where T: Iterator<Item=&'a Auction> {}

pub struct Filled<I> {
    iter: I
}

impl<'a, I> Iterator for Filled<I> where I: Iterator<Item=&'a Auction> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.find(|a| a.sold)
    }
}

pub struct AuctionType<I> {
    iter: I,
    auction_type: auction::AuctionType
}

impl<'a, I> Iterator for AuctionType<I> where I: Iterator<Item=&'a Auction> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let auction_type = self.auction_type;
        self.iter.find(|a| a.auction_type == auction_type)
    }
}

pub struct MinPrice<I> {
    iter: I,
    price: u32
}

impl<'a, I> Iterator for MinPrice<I> where I: Iterator<Item=&'a Auction> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let price = self.price;
        self.iter.find(|a| a.price >= price)
    }
}
