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
