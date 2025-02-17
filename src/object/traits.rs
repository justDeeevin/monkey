pub trait Object:
    std::fmt::Display + std::fmt::Debug + downcast_rs::Downcast + dyn_clone::DynClone
{
    fn truthy(&self) -> bool;
}
downcast_rs::impl_downcast!(Object);
dyn_clone::clone_trait_object!(Object);
