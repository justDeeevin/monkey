pub trait Node: std::fmt::Debug + std::fmt::Display + downcast_rs::Downcast {
    fn token_literal(&self) -> &str;
}
downcast_rs::impl_downcast!(Node);

pub trait Statement: Node + downcast_rs::Downcast {}
downcast_rs::impl_downcast!(Statement);

pub trait Expression: Node + downcast_rs::Downcast {}
downcast_rs::impl_downcast!(Expression);
