pub trait Object: std::fmt::Display + std::fmt::Debug + downcast_rs::Downcast {
    fn truthy(&self) -> bool;
}
downcast_rs::impl_downcast!(Object);
