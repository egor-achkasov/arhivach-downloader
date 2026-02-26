use crate::{config::Config, events::Reporter, post::Post};
use anyhow::Result;

pub mod html;

pub trait Export {
    fn export(&self, posts: &[Post], config: &Config, reporter: &dyn Reporter) -> Result<()>;
}
