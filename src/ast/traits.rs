pub trait Node: std::fmt::Debug + std::fmt::Display {
    fn token_literal(&self) -> &str;
}

#[cfg(not(test))]
pub trait Statement: Node {}
#[cfg(test)]
pub trait Statement: Node + downcast_rs::Downcast {}
#[cfg(test)]
downcast_rs::impl_downcast!(Statement);

#[cfg(not(test))]
pub trait Expression: Node {}
#[cfg(test)]
pub trait Expression: Node + downcast_rs::Downcast {}
#[cfg(test)]
downcast_rs::impl_downcast!(Expression);
