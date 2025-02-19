use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::{DynClone, clone_trait_object};
use std::fmt::{Debug, Display};

pub trait Node: Debug + Display + Downcast + DynClone {
    fn token_literal(&self) -> &str;
}
impl_downcast!(Node);
clone_trait_object!(Node);

pub trait Statement: Node + Downcast + DynClone {}
impl_downcast!(Statement);
clone_trait_object!(Statement);

pub trait Expression: Node + Downcast + DynClone {}
impl_downcast!(Expression);
clone_trait_object!(Expression);

impl std::hash::Hash for dyn Expression {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}

impl std::cmp::PartialEq for dyn Expression {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl std::cmp::Eq for dyn Expression {}
