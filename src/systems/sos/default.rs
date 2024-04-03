use num::complex::ComplexFloat;
use option_trait::Maybe;

use crate::{List, MaybeList, ProductSequence, Sos, Tf};

impl<T, B, A, S> Default for Sos<T, B, A, S>
where
    T: ComplexFloat,
    B: Maybe<[T; 3]> + MaybeList<T>,
    A: Maybe<[T; 3]> + MaybeList<T>,
    S: List<Tf<T, B, A>>,
    ProductSequence<Tf<T, B, A>, S>: Default
{
    fn default() -> Self
    {
        Self {
            sos: Default::default()
        }
    }
}