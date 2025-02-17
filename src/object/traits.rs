pub trait Object: std::fmt::Display + std::fmt::Debug + downcast_rs::Downcast {}
downcast_rs::impl_downcast!(Object);
