#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Coins {
    #[prost(message, repeated, tag = "1")]
    pub list: ::prost::alloc::vec::Vec<Coin>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Coin {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub creator: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub coin_type: ::prost::alloc::string::String,
    #[prost(uint32, tag = "4")]
    pub decimals: u32,
    #[prost(string, tag = "5")]
    pub symbol: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "7")]
    pub description: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "8")]
    pub icon_url: ::core::option::Option<::prost::alloc::string::String>,
}
