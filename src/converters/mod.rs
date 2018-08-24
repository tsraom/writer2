pub mod basic;
pub mod naive;

use std::io;
use std::io::{ Read, Write };

pub trait Converter {
    type MoreData;

    fn new() -> Self where Self: Sized;

    fn convert<R: Read, W: Write>(
        &mut self,
        reader: &mut R,
        writer: &mut W,
        data: Self::MoreData
    ) -> io::Result<()>;
}
