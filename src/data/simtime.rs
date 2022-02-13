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
            Fs =>                 1_000,
            Ps =>                     1
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

    pub fn zero() -> Self {
        Self::new(0, SimTimeUnit::S)
    }

    pub fn from_ps(v: u64) -> Self {
        Self::new(v, SimTimeUnit::Ps)
    }

    pub fn _from_ns(v: u64) -> Self {
        Self::new(v, SimTimeUnit::Ns)
    }

    pub fn _get_value(&self) -> u64 {
        self.value
    }

    pub fn _get_unit(&self) -> SimTimeUnit {
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

#[derive(Debug)]
pub struct SimTimeRange(pub SimTime, pub SimTime);

