pub struct Data {}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub mod cowsay;
// pub mod explode;
// pub mod ping;
pub mod set_channel;
