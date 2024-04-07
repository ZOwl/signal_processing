use core::ops::{AddAssign, DivAssign, Mul, MulAssign};

use num::{complex::ComplexFloat, traits::FloatConst, Complex, NumCast, One};
use array_math::{ArrayMath, SliceMath};

use crate::{Container, List, ListOrSingle, Lists, OwnedLists, DFT, TruncateIm};

pub trait DCT<'a, T>: Lists<T>
where
    T: ComplexFloat
{
    fn dct_i(&'a self) -> Self::Owned;
    fn dct_ii(&'a self) -> Self::Owned;
    fn dct_iii(&'a self) -> Self::Owned;
    fn dct_iv(&'a self) -> Self::Owned;
}

impl<'a, T, L> DCT<'a, T> for L
where
    T: ComplexFloat + Into<Complex<T::Real>> + MulAssign<T::Real> + DivAssign<T::Real> + 'static,
    L: Lists<T, IndexView<'a>: List<T>> + 'a,
    L::Owned: OwnedLists<T>,
    L::RowsMapped<Vec<Complex<T::Real>>>: OwnedLists<Complex<T::Real>>,
    L::Mapped<Complex<T::Real>>: OwnedLists<Complex<T::Real>>,
    Complex<T::Real>: AddAssign + MulAssign + DivAssign<T::Real> + Mul<T, Output = Complex<T::Real>> + Mul<T::Real, Output = Complex<T::Real>>,
    T::Real: Into<T> + Into<Complex<T::Real>>,
    Self: DFT<T>,
{
    fn dct_i(&'a self) -> Self::Owned
    {
        let mut y = self.to_owned();
        for x in y.as_mut_slice2()
            .into_iter()
        {
            SliceMath::dct_i(x)
        }
        y
    }
    fn dct_ii(&'a self) -> Self::Owned
    {
        let mut y = self.to_owned();
        for x in y.as_mut_slice2()
            .into_iter()
        {
            SliceMath::dct_ii(x)
        }
        y
    }
    fn dct_iii(&'a self) -> Self::Owned
    {
        let mut y = self.to_owned();
        for x in y.as_mut_slice2()
            .into_iter()
        {
            SliceMath::dct_iii(x)
        }
        y
    }
    fn dct_iv(&'a self) -> Self::Owned
    {
        let mut y = self.to_owned();
        for x in y.as_mut_slice2()
            .into_iter()
        {
            SliceMath::dct_iv(x)
        }
        y
    }
}

#[cfg(test)]
mod test
{
    use core::f64::consts::TAU;

    use array_math::{ArrayOps};
    use linspace::LinspaceArray;

    use crate::{plot, DCT};

    #[test]
    fn test()
    {
        const N: usize = 1024;
        const T: f64 = 1.0;
        const F: f64 = 220.0;
        
        let x: [_; N] = ArrayOps::fill(|i| (TAU*F*i as f64/N as f64*T).sin());

        let xf = [
            x.dct_i(),
            x.dct_ii(),
            x.dct_iii(),
            x.dct_iv()
        ];

        let w = (0.0..TAU).linspace_array();

        plot::plot_curves("X(e^jw)", "plots/x_z_dct.png", [
                &w.zip(xf[0]),
                &w.zip(xf[1]),
                &w.zip(xf[2]),
                &w.zip(xf[3])
            ]).unwrap()
    }
}