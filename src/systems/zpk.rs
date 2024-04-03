use core::ops::{Div, Mul};

use num::{complex::ComplexFloat, Complex, Float, NumCast, One, Zero};
use option_trait::Maybe;

use crate::{ComplexRealError, MaybeList, ProductSequence};

moddef::moddef!(
    mod {
        default,
        div,
        from,
        inv,
        mul,
        neg,
        one,
        pow,
        product,
        zero
    }
);

#[derive(Debug, Clone, Copy)]
pub struct Zpk<T: ComplexFloat, Z: MaybeList<T> = (), P: MaybeList<T> = (), K: ComplexFloat<Real = T::Real> = T>
{
    pub z: ProductSequence<T, Z>,
    pub p: ProductSequence<T, P>,
    pub k: K
}

impl<T, Z, P, K> Zpk<T, Z, P, K>
where
    T: ComplexFloat,
    Z: MaybeList<T>,
    P: MaybeList<T>,
    K: ComplexFloat<Real = T::Real>
{
    /*pub type View<'a> = Zpk<Z::View<'a>, P::View<'a>>
    where
        Z: 'a,
        P: 'a;
    pub type Owned = Zpk<Z::Owned, P::Owned>;*/

    pub fn as_view<'a>(&'a self) -> Zpk<T, Z::View<'a>, P::View<'a>, K>
    where
        Z::View<'a>: MaybeList<T>,
        P::View<'a>: MaybeList<T>
    {
        Zpk {
            z: self.z.as_view(),
            p: self.p.as_view(),
            k: self.k
        }
    }
    pub fn to_owned(&self) -> Zpk<T, Z::Owned, P::Owned, K>
    where
        Z::Owned: MaybeList<T>,
        P::Owned: MaybeList<T>
    {
        Zpk {
            z: self.z.to_owned(),
            p: self.p.to_owned(),
            k: self.k
        }
    }
    pub fn one() -> Self
    where
        Self: Default,
    {
        Zpk::default()
    }
    pub fn zero() -> Self
    where
        Self: Default
    {
        Zpk {k: K::zero(), ..Default::default()}
    }

    pub fn complex_real<'a, Tol>(&'a self, tolerance: Tol) -> Result<(Vec<[Complex<T::Real>; 2]>, Vec<[Complex<T::Real>; 2]>, Vec<T::Real>, Vec<T::Real>, K), ComplexRealError>
    where
        Tol: Maybe<T::Real>,
        T: Into<Complex<T::Real>>,
        &'a Self: Into<Zpk<T, Vec<T>, Vec<T>, K>>
    {
        let tol = if let Some(tol) = tolerance.into_option()
        {
            if tol < Zero::zero() || tol > One::one()
            {
                return Err(ComplexRealError::TolaranceOutOfRange)
            }
            tol
        }
        else
        {
            <T::Real as NumCast>::from(100.0).unwrap()*<T::Real as Float>::epsilon()
        };

        let Zpk {z, p, k}: Zpk<T, Vec<T>, Vec<T>, K> = self.into();
        let mut zc = vec![];
        let mut pc = vec![];
        let mut zr = vec![];
        let mut pr = vec![];

        for (mut m, c, r) in [(z, &mut zc, &mut zr), (p, &mut pc, &mut pr)]
        {
            while let Some(z) = m.pop()
            {
                if z.is_zero() || Float::abs(z.im()) <= tol*z.abs()
                {
                    r.push(z.re());
                }
                else
                {
                    let z_conj = z.conj();
                    if let Some(i) = m.iter()
                        .enumerate()
                        .filter(|(_, z)| !(z.is_zero() || Float::abs(z.im()) <= tol*z.abs()))
                        .reduce(|a, b| if (*a.1 - z_conj).abs() < (*b.1 - z_conj).abs()
                        {
                            a
                        }
                        else
                        {
                            b
                        }).map(|(i, _)| i)
                    {
                        let z = [z.into(), m.remove(i).into()];
                        c.push(z)
                    }
                    else
                    {
                        return Err(ComplexRealError::OddNumberComplex)
                    }
                }
            }
        }

        Ok((zc, pc, zr, pr, k))
    }
}

macro_rules! impl_op_extra {
    ($t:ident :: $f:tt) => {
        impl<'a, T1, T2, Z1, Z2, P1, P2, K1, K2, O> $t<Zpk<T2, Z2, P2, K2>> for &'a Zpk<T1, Z1, P1, K1>
        where
            T1: ComplexFloat,
            T2: ComplexFloat,
            Z1: MaybeList<T1>,
            Z2: MaybeList<T2>,
            P1: MaybeList<T1>,
            P2: MaybeList<T2>,
            K1: ComplexFloat<Real = T1::Real>,
            K2: ComplexFloat<Real = T2::Real>,
            Z1::View<'a>: MaybeList<T1>,
            P1::View<'a>: MaybeList<T1>,
            Zpk<T1, Z1::View<'a>, P1::View<'a>, K1>: $t<Zpk<T2, Z2, P2, K2>, Output = O>
        {
            type Output = O;

            fn $f(self, rhs: Zpk<T2, Z2, P2, K2>) -> Self::Output
            {
                self.as_view().$f(rhs)
            }
        }
        impl<'b, T1, T2, Z1, Z2, P1, P2, K1, K2, O> $t<&'b Zpk<T2, Z2, P2, K2>> for Zpk<T1, Z1, P1, K1>
        where
            T1: ComplexFloat,
            T2: ComplexFloat,
            Z1: MaybeList<T1>,
            Z2: MaybeList<T2>,
            P1: MaybeList<T1>,
            P2: MaybeList<T2>,
            K1: ComplexFloat<Real = T1::Real>,
            K2: ComplexFloat<Real = T2::Real>,
            Z2::View<'b>: MaybeList<T2>,
            P2::View<'b>: MaybeList<T2>,
            Zpk<T1, Z1, P1, K1>: $t<Zpk<T2, Z2::View<'b>, P2::View<'b>, K2>, Output = O>
        {
            type Output = O;

            fn $f(self, rhs: &'b Zpk<T2, Z2, P2, K2>) -> Self::Output
            {
                self.$f(rhs.as_view())
            }
        }
        impl<'a, 'b, T1, T2, Z1, Z2, P1, P2, K1, K2, O> $t<&'b Zpk<T2, Z2, P2, K2>> for &'a Zpk<T1, Z1, P1, K1>
        where
            T1: ComplexFloat,
            T2: ComplexFloat,
            Z1: MaybeList<T1>,
            Z2: MaybeList<T2>,
            P1: MaybeList<T1>,
            P2: MaybeList<T2>,
            K1: ComplexFloat<Real = T1::Real>,
            K2: ComplexFloat<Real = T2::Real>,
            Z1::View<'a>: MaybeList<T1>,
            P1::View<'a>: MaybeList<T1>,
            Z2::View<'b>: MaybeList<T2>,
            P2::View<'b>: MaybeList<T2>,
            Zpk<T1, Z1::View<'a>, P1::View<'a>, K1>: $t<Zpk<T2, Z2::View<'b>, P2::View<'b>, K2>, Output = O>
        {
            type Output = O;

            fn $f(self, rhs: &'b Zpk<T2, Z2, P2, K2>) -> Self::Output
            {
                self.as_view().$f(rhs.as_view())
            }
        }

        impl<T, Z, P, K, O> $t<K> for Zpk<T, Z, P, K>
        where
            T: ComplexFloat,
            Z: MaybeList<T>,
            P: MaybeList<T>,
            K: ComplexFloat<Real = T::Real>,
            Self: $t<Zpk<T, (), (), K>, Output = O>
        {
            type Output = O;

            fn $f(self, rhs: K) -> Self::Output
            {
                self.$f(Zpk {
                    z: ProductSequence::new(()),
                    p: ProductSequence::new(()),
                    k: rhs
                })
            }
        }
        impl<'b, T, Z, P, K, O> $t<K> for &'b Zpk<T, Z, P, K>
        where
            T: ComplexFloat,
            Z: MaybeList<T>,
            P: MaybeList<T>,
            K: ComplexFloat<Real = T::Real>,
            Self: $t<Zpk<T, (), (), K>, Output = O>
        {
            type Output = O;

            fn $f(self, rhs: K) -> Self::Output
            {
                self.$f(Zpk {
                    z: ProductSequence::new(()),
                    p: ProductSequence::new(()),
                    k: rhs
                })
            }
        }
    };
}
impl_op_extra!(Mul::mul);
impl_op_extra!(Div::div);