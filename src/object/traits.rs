pub trait Object:
    std::fmt::Display + std::fmt::Debug + downcast_rs::Downcast + dyn_clone::DynClone
{
    fn truthy(&self) -> bool;
    fn type_name(&self) -> &'static str;
}
downcast_rs::impl_downcast!(Object);
dyn_clone::clone_trait_object!(Object);

impl std::hash::Hash for dyn Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}

impl std::cmp::PartialEq for dyn Object {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}
impl Eq for dyn Object {}
