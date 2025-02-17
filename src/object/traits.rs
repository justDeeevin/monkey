pub trait Object: std::fmt::Display + downcast_rs::Downcast {}
downcast_rs::impl_downcast!(Object);
