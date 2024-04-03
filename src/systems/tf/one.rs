use std::ops::Mul;

use num::{complex::ComplexFloat, One};

use crate::{List, MaybeList, Tf};

impl<T, B, A> One for Tf<T, B, A>
where
    T: ComplexFloat,
    Self: Mul<Output = Self> + Default,
    B: List<T>,
    A: MaybeList<T>
{
    fn one() -> Self
    {
        Self::one()
    }
}