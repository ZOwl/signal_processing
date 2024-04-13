use core::ops::{AddAssign, Div, DivAssign, Mul, MulAssign, SubAssign};

use ndarray_linalg::Lapack;
use num::{complex::ComplexFloat, traits::{float::TotalOrder, Euclid, FloatConst}, Complex, Float, One, Zero};

use array_math::SliceMath;
use option_trait::Maybe;

use crate::{MaybeList, Normalize, Polynomial, Rpk, SumSequence, System, Tf, ToTf};

pub trait Residue: System
{
    type Output: System<Domain: ComplexFloat<Real = <Self::Domain as ComplexFloat>::Real>>;

    fn residue<TOL>(self, tol: TOL) -> Self::Output
    where
        TOL: Maybe<<Self::Domain as ComplexFloat>::Real>;
}

impl<T, B, A, R> Residue for Tf<T, B, A>
where
    T: ComplexFloat<Real = R> + Lapack<Complex = Complex<R>> + 'static,
    R: Float + FloatConst + TotalOrder + Into<T>,
    B: MaybeList<T>,
    A: MaybeList<T>,
    Self: Normalize<Output: ToTf<T, Vec<T>, Vec<T>, (), ()>> + System<Domain = T>,
    Complex<R>: AddAssign + SubAssign + MulAssign + DivAssign + From<T> + DivAssign<R> + Div<T, Output = Complex<R>>,
    Polynomial<T, Vec<T>>: Euclid
{
    type Output = Rpk<T, Complex<R>, Complex<R>, Vec<(Complex<R>, Complex<R>)>, Vec<T>>;

    fn residue<TOL>(self, tol: TOL) -> Self::Output
    where
        TOL: Maybe<R>
    {
        let Tf {mut b, a} = self.normalize()
            .to_tf((), ());

        if a.is_zero()
        {
            let r = if b.is_zero()
            {
                Zero::zero()
            }
            else
            {
                One::one()
            };
            return Rpk {
                rp: SumSequence::new(vec![(r, Zero::zero())]),
                k: Polynomial::zero()
            }
        }

        let mut poles: Vec<_> = a.rpolynomial_roots();
        if b.is_zero()
        {
            poles.sort_by(|a, b| a.norm_sqr().total_cmp(&b.norm_sqr()));
            return Rpk {
                rp: SumSequence::new(poles.into_iter().map(|p| (Zero::zero(), p)).collect()),
                k: Polynomial::zero()
            }
        }

        let k = if b.len() < a.len()
        {
            Polynomial::zero()
        }
        else
        {
            let k;
            (k, b) = b.div_rem_euclid(&a);
            k
        };

        let tol = tol.into_option()
            .map(|tol| Float::abs(tol))
            .unwrap_or_else(|| R::from(1e-3).unwrap());
        let mut unique_poles_multiplicity: Vec<(Complex<_>, usize)> = vec![];
        'lp:
        for p in poles
        {
            for (pu, n) in unique_poles_multiplicity.iter_mut()
            {
                if (p - *pu).abs() < tol
                {
                    *n += 1;
                    continue 'lp;
                }
            }
            unique_poles_multiplicity.push((p, 1));
        }
        unique_poles_multiplicity.sort_by(|a, b| a.0.norm_sqr().total_cmp(&b.0.norm_sqr()));

        let residues = compute_residues(unique_poles_multiplicity.as_slice(), b);

        let norm = a[0];
        let rp = residues.into_iter()
            .map(|r| r/norm)
            .zip(unique_poles_multiplicity.into_iter()
                .map(|(pole, mult)| core::iter::repeat(pole).take(mult))
                .flatten()
            ).collect();

        Rpk {
            rp: SumSequence::new(rp),
            k
        }
    }
}

fn compute_residues<T>(unique_poles_multiplicity: &[(Complex<T::Real>, usize)], numer: Polynomial<T, Vec<T>>)
    -> Vec<Complex<T::Real>>
where
    T: ComplexFloat + Into<Complex<T::Real>> + 'static,
    Complex<T::Real>: AddAssign + MulAssign,
    Polynomial<Complex<T::Real>, Vec<Complex<T::Real>>>: Mul<Polynomial<Complex<T::Real>, Vec<Complex<T::Real>>>, Output = Polynomial<Complex<T::Real>, Vec<Complex<T::Real>>>> + Mul<Polynomial<Complex<T::Real>, [Complex<T::Real>; 2]>, Output = Polynomial<Complex<T::Real>, Vec<Complex<T::Real>>>>
{
    let (denom_factors, _) = compute_residue_factors::<Complex<T::Real>>(unique_poles_multiplicity, false);
    let numer: Polynomial<Complex<T::Real>, Vec<Complex<T::Real>>> = numer.map_into_owned(Into::into);

    let mut residues = vec![];
    for (&(pole, mult), factor) in unique_poles_multiplicity.iter()
        .zip(denom_factors)
    {
        if mult == 1
        {
            residues.push(numer.rpolynomial(pole)/factor.rpolynomial(pole))
        }
        else
        {
            let mut numer = numer.clone();
            let monomial = Polynomial::new(vec![One::one(), -pole]);
            let (factor, d) = factor.div_rem_euclid(&monomial);

            let mut block = vec![];
            for _ in 0..mult
            {
                let n;
                (numer, n) = numer.div_rem_euclid(&monomial);
                let r = n[0]/d[0];
                numer = numer - factor.clone()*Polynomial::new([r]);
                block.push(r);
            }
            block.reverse();
            residues.append(&mut block)
        }
    }
    residues
}

fn compute_residue_factors<T>(unique_poles_multiplicity: &[(Complex<T::Real>, usize)], include_powers: bool)
    -> (Vec<Polynomial<T, Vec<T>>>, Polynomial<T, Vec<T>>)
where
    T: ComplexFloat<Real: Into<T>> + 'static,
    Polynomial<Complex<T::Real>, Vec<Complex<T::Real>>>: Mul<Polynomial<Complex<T::Real>, Vec<Complex<T::Real>>>, Output = Polynomial<Complex<T::Real>, Vec<Complex<T::Real>>>> + Mul<Polynomial<Complex<T::Real>, [Complex<T::Real>; 2]>, Output = Polynomial<Complex<T::Real>, Vec<Complex<T::Real>>>>
{
    let mut current = Polynomial::new(vec![Complex::<T::Real>::one()]);
    let mut suffixes = vec![current.clone()];
    for &(pole, mult) in unique_poles_multiplicity.iter()
        .rev()
    {
        let monomial = Polynomial::new([One::one(), -pole]);
        for _ in 0..mult
        {
            current = current*monomial
        }
        suffixes.push(current.clone());
    }
    suffixes.reverse();

    let mut factors = vec![];
    current = Polynomial::new(vec![One::one()]);
    for (&(pole, mult), suffix) in unique_poles_multiplicity.iter()
        .zip(suffixes.into_iter().skip(1))
    {
        let monomial = Polynomial::new([One::one(), -pole]);
        let mut block = vec![];
        for i in 0..mult
        {
            if i == 0 || include_powers
            {
                block.push((current.clone()*suffix.clone()).truncate_im())
            }
            current = current*monomial
        }
        block.reverse();
        factors.append(&mut block)
    }

    (factors, current.truncate_im())
}

#[cfg(test)]
mod test
{
    use crate::{Residue, Tf};

    #[test]
    fn test()
    {
        let h = Tf::new(
            [1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0]
        );
        let rpk = h.residue(());
        println!("{:?}", rpk)
    }
}