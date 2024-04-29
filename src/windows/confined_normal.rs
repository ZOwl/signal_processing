

use array_math::ArrayOps;
use num::{traits::FloatConst, Float};

use crate::gen::window::{WindowGen, WindowRange};

pub struct ConfinedNormal<T>
where
    T: Float
{
    pub sigma: T,
    pub p: T
}

impl<T, const N: usize> WindowGen<T, [T; N], ()> for ConfinedNormal<T>
where
    T: Float + FloatConst
{
    type Output = [T; N];

    fn window_gen(&self, (): (), r: WindowRange) -> Self::Output
    {
        if N <= 1
        {
            return [T::one(); N]
        }

        let m = match r
        {
            WindowRange::Symmetric => N - 1,
            WindowRange::Periodic => N,
        };

        let one = T::one();
        let two = one + one;
        let half = two.recip();
        let l = T::from(m).unwrap();
        let g = |x| {
            let z: T = (x - half*T::from(N - 1).unwrap())/(self.sigma*two*l);
            (-z.abs().powf(self.p)).exp()
        };
        let gmul = g(-half)/(g(-half + l) + g(-half - l));
        ArrayOps::fill(|i| {
            let i = T::from(i).unwrap();
            g(i) - (g(i + l) + g(i - l))*gmul
        })
    }
}
impl<T> WindowGen<T, Vec<T>, usize> for ConfinedNormal<T>
where
    T: Float + FloatConst
{
    type Output = Vec<T>;

    fn window_gen(&self, n: usize, r: WindowRange) -> Self::Output
    {
        if n <= 1
        {
            return vec![T::one(); n]
        }

        let m = match r
        {
            WindowRange::Symmetric => n - 1,
            WindowRange::Periodic => n,
        };

        let one = T::one();
        let two = one + one;
        let half = two.recip();
        let l = T::from(m).unwrap();
        let g = |x| {
            let z: T = (x - half*T::from(n - 1).unwrap())/(self.sigma*two*l);
            (-z.abs().powf(self.p)).exp()
        };
        let gmul = g(-half)/(g(-half + l) + g(-half - l));
        (0..n).map(|i| {
            let i = T::from(i).unwrap();
            g(i) - (g(i + l) + g(i - l))*gmul
        }).collect()
    }
}

#[cfg(test)]
mod test
{
    use core::f64::consts::{PI, TAU};

    use array_math::ArrayOps;
    use linspace::LinspaceArray;

    use crate::{plot, gen::window::{WindowGen, WindowRange}, analysis::FreqZ, systems::Tf};

    use super::ConfinedNormal;

    #[test]
    fn test()
    {
        const N: usize = 1024;
        let w: [_; N/2] = ConfinedNormal {sigma: 0.1, p: 1.5}.window_gen((), WindowRange::Symmetric);
        let n = (0.0..1.0).linspace_array();

        plot::plot_curves("g(n/N)", "plots/windows/g_n_confined_normal.png", [&n.zip(w)]).unwrap();

        let (mut w_f, mut omega): ([_; N], _) = Tf::new(w, ()).freqz((), false);
        omega.map_assign(|omega| (omega + PI) % TAU - PI);
        w_f.rotate_right(N/2);
        omega.rotate_right(N/2);
        
        plot::plot_curves("G(e^jw)", "plots/windows/g_f_confined_normal.png", [&omega.zip(w_f.map(|w| 20.0*w.norm().log10()))]).unwrap();
    }
}