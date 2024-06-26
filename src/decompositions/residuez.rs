use core::ops::Mul;

use num::{complex::ComplexFloat, traits::FloatConst, Float};
use option_trait::Maybe;
use array_math::SliceMath;

use crate::{quantities::{MaybeList, MaybeOwnedList}, operations::Simplify, decompositions::Residue, systems::{Rpk, Tf}, System};

pub trait ResidueZ: System
{
    type Output: System<Set: ComplexFloat<Real = <Self::Set as ComplexFloat>::Real>>;
    
    fn residuez<TOL>(self, tol: TOL) -> Self::Output
    where
        TOL: Maybe<<Self::Set as ComplexFloat>::Real>;
}

impl<T, B, B2, A, A2, TR, R, P, RP, K> ResidueZ for Tf<T, B, A>
where
    T: ComplexFloat<Real = TR>,
    B: MaybeList<T>,
    A: MaybeList<T>,
    B2: MaybeOwnedList<T>,
    A2: MaybeOwnedList<T>,
    TR: Float + FloatConst,
    Self: Simplify<Output = Tf<T, B2, A2>> + System<Set = T>,
    Tf<T, B2, A2>: Residue<Output = Rpk<T, R, P, RP, K>> + System<Set = T>,
    RP: MaybeOwnedList<(R, P)>,
    K: MaybeOwnedList<T>,
    R: ComplexFloat<Real = TR> + Mul<P, Output = R>,
    P: ComplexFloat<Real = TR>
{
    type Output = Rpk<T, R, P, RP, K>;

    fn residuez<TOL>(self, tol: TOL) -> Self::Output
    where
        TOL: Maybe<TR>
    {
        let mut tf = self.simplify();

        if let Some(b) = tf.b.as_mut_slice_option()
        {
            b.reverse();
            b.conj_assign_all();
        }
        if let Some(a) = tf.a.as_mut_slice_option()
        {
            a.reverse();
            a.conj_assign_all();
        }

        let mut rpk = tf.residue(tol);

        if let Some(rp) = rpk.rp.as_mut_slice_option()
        {
            let mut p_prev = None;
            let mut m = 1;
            for (r, p) in rp.iter_mut()
            {
                if p_prev == Some(*p)
                {
                    m += 1
                }
                else
                {
                    m = 1;
                    p_prev = Some(*p)
                }

                *p = p.recip();
                *r = *r*((-*p).powi(m));
            }
        }
        if let Some(k) = rpk.k.as_mut_slice_option()
        {
            k.reverse();
            k.conj_assign_all();
        }

        rpk
    }
}

impl<T, R, P, RP, K, B, A> ResidueZ for Rpk<T, R, P, RP, K>
where
    T: ComplexFloat,
    R: ComplexFloat<Real = T::Real> + Mul<P, Output = R>,
    P: ComplexFloat<Real = T::Real>,
    RP: MaybeList<(R, P)>,
    K: MaybeList<T>,
    RP::Owned: MaybeOwnedList<(R, P)>,
    K::Owned: MaybeOwnedList<T>,
    Rpk<T, R, P, RP::Owned, K::Owned>: Residue<Output = Tf<T, B, A>> + System<Set = T>,
    B: MaybeOwnedList<T>,
    A: MaybeOwnedList<T>,
    Tf<T, B, A>: Simplify + System<Set = T>
{
    type Output = <Tf<T, B, A> as Simplify>::Output;

    fn residuez<TOL>(self, tol: TOL) -> Self::Output
    where
        TOL: Maybe<<Self::Set as ComplexFloat>::Real>
    {
        let mut rpk = self.into_owned();

        if let Some(k) = rpk.k.as_mut_slice_option()
        {
            k.reverse();
            k.conj_assign_all();
        }
        if let Some(rp) = rpk.rp.as_mut_slice_option()
        {
            let mut p_prev = None;
            let mut m = 1;
            for (r, p) in rp.iter_mut()
            {
                if p_prev == Some(*p)
                {
                    m += 1
                }
                else
                {
                    m = 1;
                    p_prev = Some(*p)
                }

                *p = p.recip();
                *r = *r*((-*p).powi(m));
            }
        }

        let mut tf = rpk.residue(tol);
        
        if let Some(b) = tf.b.as_mut_slice_option()
        {
            b.reverse();
            b.conj_assign_all();
        }
        if let Some(a) = tf.a.as_mut_slice_option()
        {
            a.reverse();
            a.conj_assign_all();
        }

        tf.simplify()
    }
}

#[cfg(test)]
mod test
{
    use crate::{decompositions::ResidueZ, systems::Tf};

    #[test]
    fn test()
    {
        let h = Tf::new(
            [4.0, 5.0, 6.0],
            [1.0, 2.0, 3.0]
        );
        let rpk = h.residuez(());
        println!("{:?}", rpk);

        let rpk = &rpk + &rpk;
        
        println!("\n{:?}", rpk);

        let h = rpk.residuez(());
        println!("\n{:?}", h);
    }
}