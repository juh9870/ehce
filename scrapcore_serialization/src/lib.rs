mod registry;

mod reservation;
mod serialization;

#[cfg(test)]
mod test;

pub type ItemId = String;
pub type ItemIdRef<'a> = &'a str;
pub type AssetName = String;
pub type AssetNameRef<'a> = &'a str;
