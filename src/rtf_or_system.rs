use num::complex::ComplexFloat;
use option_trait::Maybe;

use crate::{
    quantities::{ListOrSingle, MaybeList, MaybeLists, MaybeOwnedList},
    systems::{Ar, Rpk, Sos, Ss, SsAMatrix, SsBMatrix, SsCMatrix, SsDMatrix, Tf, Zpk, Rtf},
    MaybeRtfOrSystem,
    System
};

pub trait RtfOrSystem: MaybeRtfOrSystem<Self::Set>
{
    type Set: ComplexFloat;
}

impl !RtfOrSystem for () {}

impl<W, S> RtfOrSystem for Rtf<W, S>
where
    W: ComplexFloat<Real = <S::Set as ComplexFloat>::Real>,
    S::Set: Into<W>,
    S: System
{
    type Set = W;
}

impl<T, B, A> RtfOrSystem for Tf<T, B, A>
where
    T: ComplexFloat,
    B: MaybeLists<T>,
    A: MaybeList<T>
{
    type Set = T;
}

impl<T, Z, P, K, R> RtfOrSystem for Zpk<T, Z, P, K>
where
    T: ComplexFloat<Real = R>,
    K: ComplexFloat<Real = R>,
    Z: MaybeList<T>,
    P: MaybeList<T>
{
    type Set = K;
}

impl<T, A, B, C, D> RtfOrSystem for Ss<T, A, B, C, D>
where
    T: ComplexFloat,
    A: SsAMatrix<T, B, C, D>,
    B: SsBMatrix<T, A, C, D>,
    C: SsCMatrix<T, A, B, D>,
    D: SsDMatrix<T, A, B, C>
{
    type Set = T;
}

impl<T, B, A, S> RtfOrSystem for Sos<T, B, A, S>
where
    T: ComplexFloat,
    B: Maybe<[T; 3]> + MaybeOwnedList<T>,
    A: Maybe<[T; 3]> + MaybeOwnedList<T>,
    S: MaybeList<Tf<T, B, A>>
{
    type Set = T;
}

impl<T, R, P, RP, K> RtfOrSystem for Rpk<T, R, P, RP, K>
where
    T: ComplexFloat,
    R: ComplexFloat<Real = T::Real>,
    P: ComplexFloat<Real = T::Real>,
    RP: MaybeList<(R, P)>,
    K: MaybeList<T>
{
    type Set = T;
}

impl<T, A, AV> RtfOrSystem for Ar<T, A, AV>
where
    T: ComplexFloat,
    A: MaybeList<T>,
    AV: ListOrSingle<(A, T::Real)>
{
    type Set = T;
}