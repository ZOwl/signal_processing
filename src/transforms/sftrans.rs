use core::ops::{Add, Div, DivAssign, MulAssign, Sub};
use std::ops::Mul;

use num::{complex::ComplexFloat, Complex, Float, One, Zero};
use thiserror::Error;

use crate::{MaybeList, ProductSequence, System, ToZpk, Zpk};

#[derive(Debug, Clone, Copy, PartialEq, Error)]
pub enum SfTransError
{
    #[error("Non causal system, i.e. it contains one or more poles at infinity.")]
    NonCausal,
    #[error("The system must contain at least one pole.")]
    ZeroPoles
}

pub trait SfTrans: System
{
    type Output: System<Domain = Self::Domain>;

    fn sftrans<const W: usize>(self, w: [<Self::Domain as ComplexFloat>::Real; W], stop: bool) -> Result<Self::Output, SfTransError>
    where
        [(); W - 1]:,
        [(); 2 - W]:;
}

impl<T, Z, P, K> SfTrans for Zpk<T, Z, P, K>
where
    T: ComplexFloat + Mul<T::Real, Output = T> + Div<T::Real, Output = T> + Sub<T::Real, Output = T> + Into<Complex<T::Real>>,
    K: ComplexFloat<Real = T::Real> + DivAssign<T::Real> + MulAssign<T::Real>,
    T::Real: Into<T>,
    Complex<T::Real>: Add<T, Output = Complex<T::Real>>,
    Z: MaybeList<T>,
    P: MaybeList<T>,
    Self: ToZpk<T, Vec<T>, Vec<T>, K, (), ()> + System<Domain = K>
{
    type Output = Zpk<Complex<T::Real>, Vec<Complex<T::Real>>, Vec<Complex<T::Real>>, K>;

    fn sftrans<const W: usize>(self, w: [T::Real; W], stop: bool) -> Result<Self::Output, SfTransError>
    where
        [(); W - 1]:,
        [(); 2 - W]:
    {
        let Zpk::<T, Vec<T>, Vec<T>, K> {z: sz, p: sp, k: mut sg} = self.to_zpk((), ());
    
        let two = T::Real::one() + T::Real::one();
        let c = T::Real::one();
        let p = sp.len();
        let z = sz.len();
        if z > p
        {
            return Err(SfTransError::NonCausal)
        }
        if p == 0
        {
            return Err(SfTransError::ZeroPoles)
        }
    
        if W == 2
        {
            let fl = w[0];
            let fh = w[1];
            if stop
            {
                if let Some(prod) = sp.iter()
                    .map(|&sp| -sp)
                    .reduce(Mul::mul)
                {
                    sg /= prod.re()
                }
                if let Some(prod) = sz.iter()
                    .map(|&sz| -sz)
                    .reduce(Mul::mul)
                {
                    sg /= prod.re()
                }
                let b_mul = c*(fh - fl)/two;
                let sp = {
                    let b = sp.into_inner()
                        .into_iter()
                        .map(|sp| Into::<T>::into(b_mul)/sp)
                        .collect::<Vec<_>>();
                    let bs = b.iter()
                        .map(|&b| Into::<Complex<T::Real>>::into(b*b - fh*fl).sqrt())
                        .collect::<Vec<_>>();
                    [
                        b.iter()
                            .zip(bs.iter())
                            .map(|(&b, &bs)| bs + b)
                            .collect::<Vec<_>>(),
                        b.into_iter()
                            .zip(bs)
                            .map(|(b, bs)| -bs + b)
                            .collect()
                    ].concat()
                };
                let mut sz = {
                    let b = sz.into_inner()
                        .into_iter()
                        .map(|sz| Into::<T>::into(b_mul)/sz)
                        .collect::<Vec<_>>();
                    let bs = b.iter()
                        .map(|&b| Into::<Complex<T::Real>>::into(b*b - fh*fl).sqrt())
                        .collect::<Vec<_>>();
                    [
                        b.iter()
                            .zip(bs.iter())
                            .map(|(&b, &bs)| bs + b)
                            .collect::<Vec<_>>(),
                        b.into_iter()
                            .zip(bs)
                            .map(|(b, bs)| -bs + b)
                            .collect()
                    ].concat()
                };
                let extend0 = Into::<Complex<T::Real>>::into(-fh*fl).sqrt();
                let extend = [extend0, -extend0];
                sz.append(&mut (1..=2*(p - z)).map(|i| extend[i % 2]).collect());
                Ok(Zpk {
                    z: ProductSequence::new(sz),
                    p: ProductSequence::new(sp),
                    k: sg
                })
            }
            else
            {
                sg *= Float::powi((fh - fl)/c, (p - z) as i32);
                let b_mul = (fh - fl)/(two*c);
                let sp = {
                    let b = sp.into_inner()
                        .into_iter()
                        .map(|sp| sp*b_mul)
                        .collect::<Vec<_>>();
                    let bs = b.iter()
                        .map(|&b| Into::<Complex<T::Real>>::into(b*b - fh*fl).sqrt())
                        .collect::<Vec<_>>();
                    [
                        b.iter()
                            .zip(bs.iter())
                            .map(|(&b, &bs)| bs + b)
                            .collect::<Vec<_>>(),
                        b.into_iter()
                            .zip(bs)
                            .map(|(b, bs)| -bs + b)
                            .collect()
                    ].concat()
                };
                let mut sz = {
                    let b = sz.into_inner()
                        .into_iter()
                        .map(|sz| sz*b_mul)
                        .collect::<Vec<_>>();
                    let bs = b.iter()
                        .map(|&b| Into::<Complex<T::Real>>::into(b*b - fh*fl).sqrt())
                        .collect::<Vec<_>>();
                    [
                        b.iter()
                            .zip(bs.iter())
                            .map(|(&b, &bs)| bs + b)
                            .collect::<Vec<_>>(),
                        b.into_iter()
                            .zip(bs)
                            .map(|(b, bs)| -bs + b)
                            .collect()
                    ].concat()
                };
                sz.append(&mut vec![Zero::zero(); p - z]);
                Ok(Zpk {
                    z: ProductSequence::new(sz),
                    p: ProductSequence::new(sp),
                    k: sg
                })
            }
        }
        else
        {
            let fc = w[0];
            if stop
            {
                if let Some(prod) = sp.iter()
                    .map(|&sp| -sp)
                    .reduce(Mul::mul)
                {
                    sg /= prod.re()
                }
                if let Some(prod) = sz.iter()
                    .map(|&sz| -sz)
                    .reduce(Mul::mul)
                {
                    sg /= prod.re()
                }
                let b_mul = c*fc;
                let sp = sp.into_inner()
                    .into_iter()
                    .map(|sp| Into::<Complex<T::Real>>::into(Into::<T>::into(b_mul)/sp))
                    .collect::<Vec<_>>();
                let mut sz = sz.into_inner()
                    .into_iter()
                    .map(|sz| Into::<Complex<T::Real>>::into(Into::<T>::into(b_mul)/sz))
                    .collect::<Vec<_>>();
                sz.append(&mut vec![Zero::zero(); p - z]);
                Ok(Zpk {
                    z: ProductSequence::new(sz),
                    p: ProductSequence::new(sp),
                    k: sg
                })
            }
            else
            {
                sg *= Float::powi(fc/c, (p - z) as i32);
                let b_mul = fc/c;
                let sp = sp.into_inner()
                    .into_iter()
                    .map(|sp| Into::<Complex<T::Real>>::into(Into::<T>::into(b_mul)*sp))
                    .collect::<Vec<_>>();
                let sz = sz.into_inner()
                    .into_iter()
                    .map(|sz| Into::<Complex<T::Real>>::into(Into::<T>::into(b_mul)*sz))
                    .collect::<Vec<_>>();
                Ok(Zpk {
                    z: ProductSequence::new(sz),
                    p: ProductSequence::new(sp),
                    k: sg
                })
            }
        }
    }
}