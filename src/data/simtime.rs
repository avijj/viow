use crate::error::*;

use rug::{Integer, Rational};

#[derive(Debug, Clone, Copy)]
pub enum SimTimeUnit {
    Fs, Ps, Us, Ns, Ms, S,
}

impl SimTimeUnit {
    pub fn from_string(s: impl AsRef<str>) -> Result<Self> {
        let s = s.as_ref();

        match s {
            "s"  => Ok(Self::S ),
            "ms" => Ok(Self::Ms),
            "us" => Ok(Self::Us),
            "ns" => Ok(Self::Ns),
            "ps" => Ok(Self::Ps),
            "fs" => Ok(Self::Fs),
            _    => Err(Error::InvalidTime(s.to_string()))
        }
    }


    fn to_multiplier(&self) -> u64 {
        use SimTimeUnit::*;
        match self {
            S  => 1_000_000_000_000_000,
            Ms =>     1_000_000_000_000,
            Us =>         1_000_000_000,
            Ns =>             1_000_000,
            Ps =>                 1_000,
            Fs =>                     1
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct SimTime {
    value: u64,
    unit: SimTimeUnit,
}

impl SimTime {
    pub const fn new(v: u64, u: SimTimeUnit) -> Self {
        Self {
            value: v,
            unit: u,
        }
    }

    pub const fn zero() -> Self {
        Self::new(0, SimTimeUnit::S)
    }

    pub const fn from_s(v: u64) -> Self {
        Self::new(v, SimTimeUnit::S)
    }
    
    pub const fn from_ms(v: u64) -> Self {
        Self::new(v, SimTimeUnit::Ms)
    }
    
    pub const fn from_us(v: u64) -> Self {
        Self::new(v, SimTimeUnit::Us)
    }

    pub const fn from_ns(v: u64) -> Self {
        Self::new(v, SimTimeUnit::Ns)
    }

    pub const fn from_ps(v: u64) -> Self {
        Self::new(v, SimTimeUnit::Ps)
    }

    pub const fn from_fs(v: u64) -> Self {
        Self::new(v, SimTimeUnit::Fs)
    }

    pub fn get_value(&self) -> u64 {
        self.value
    }

    pub fn get_unit(&self) -> SimTimeUnit {
        self.unit
    }

    fn to_bigint(&self) -> Integer {
        Integer::from(self.value) * self.unit.to_multiplier()
    }
}

impl std::ops::Mul<u64> for SimTime {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self::Output {
        Self {
            value: self.value * rhs,
            unit: self.unit
        }
    }
}

impl std::ops::Div<SimTime> for SimTime {
    type Output = u64;

    fn div(self, rhs: SimTime) -> Self::Output {
        let r = Rational::from((self.to_bigint(), rhs.to_bigint()));
        let (_, floor) = r.fract_floor(Integer::new());
        floor.to_u64()
            .expect("Integer overflow in division")
    }
}

//impl std::ops::Sub<SimTime> for SimTime {
    //type Output = Self;

    //fn sub(self, rhs: Self) -> Self::Output {
        //let a = self.to_bigint();
        //let b = rhs.to_bigint();
        //let res = a - b;

        //Self {
            //value
        //}
    //}
//}

#[derive(Debug)]
pub struct SimTimeRange(pub SimTime, pub SimTime);


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simtime_div() {
        let s_232 = SimTime::from_s(232);
        let ms_13 = SimTime::from_ms(13);
        let us_42 = SimTime::from_us(42);
        let ns_100 = SimTime::from_ns(100);
        let ps_10 = SimTime::from_ps(10);
        let fs_5 = SimTime::from_fs(5);

        assert_eq!(10000, ns_100 / ps_10);
        assert_eq!(0, ps_10 / ns_100);

        assert_eq!((100.0e-9 / 5.0e-15) as u64, ns_100 / fs_5);
        assert_eq!((232.0 / 42.0e-6) as u64, s_232 / us_42);
        assert_eq!((13.0e-3 / 10.0e-12) as u64, ms_13 / ps_10);
    }

    #[test]
    fn test_simtime_mul() {
        let a = SimTime::from_ms(15323);

        assert_eq!(15323 * 5, (a * 5).get_value());
        assert_eq!(15323    , (a * 1).get_value());
        assert_eq!(15323 * 0, (a * 0).get_value());
    }
}
