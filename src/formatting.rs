use rug::Integer;
use viow_plugin_api::SignalType;
use std::convert::From;

#[derive(Clone,Copy,PartialEq)]
pub enum WaveFormat {
    Bit,
    Vector(u32),
    BitVector(u32),
    Comment,
}

impl From<SignalType> for WaveFormat {
    fn from(t: SignalType) -> Self {
        use SignalType::*;

        match t {
            Bit => WaveFormat::Bit,
            Vector(a, b) => WaveFormat::Vector((a - b).abs() as u32),
        }
    }
}

fn build_waveform_vec<'a, T>(line_data: T, zoom: usize) -> String 
    where
        T: Iterator<Item = &'a Integer>
{
    line_data
        .map(|x| core::iter::repeat(x).take(zoom))
        .flatten()
        .fold(FormatAcc::new(), format_vec)
        .msg
}

fn build_waveform_bitvec<'a, T>(line_data: T, zoom: usize) -> String 
    where
        T: Iterator<Item = &'a Integer>
{
    line_data
        .map(|x| core::iter::repeat(x).take(zoom))
        .flatten()
        .fold(FormatAcc::new(), format_bitvec)
        .msg
}

fn build_waveform_bit<'a, T>(line_data: T, zoom: usize) -> String 
    where
        T: Iterator<Item = &'a Integer>
{
    line_data
        .map(|x| core::iter::repeat(x).take(zoom))
        .flatten()
        .map(format_bit)
        .collect()
}

fn build_waveform_comment<'a, T>(line_data: T, zoom: usize) -> String
    where
        T: Iterator<Item = &'a Integer>
{
    core::iter::repeat('.')
        .take(zoom * line_data.count())
        .collect()
}

pub fn build_waveform<'a, T>(line_data: T, format: WaveFormat, zoom: usize) -> String 
    where
        T: Iterator<Item = &'a Integer>
{
    match format {
        WaveFormat::Bit => build_waveform_bit(line_data, zoom),
        WaveFormat::Vector(_) => build_waveform_vec(line_data, zoom),
        WaveFormat::BitVector(_) => build_waveform_bitvec(line_data, zoom),
        WaveFormat::Comment => build_waveform_comment(line_data, zoom),
    }
}


pub fn format_value(value: &Integer, format: WaveFormat) -> String {
    match format {
        WaveFormat::Bit => format!("{:b}", value),
        WaveFormat::Vector(size) => {
            let hex_digits = if size % 4 == 0 {
                size / 4
            } else {
                size / 4 + 1
            };
            format!("{:#0width$x}", value, width = (hex_digits as usize) + 2)
        }
        WaveFormat::BitVector(size) => format!("{:#0width$b}", value, width = size as usize + 2),
        WaveFormat::Comment => "".to_string(),
    }
}


fn format_bit(value: &Integer) -> char {
    if *value == 0 {
        '▁'
    } else {
        '▇'
    }
}

struct FormatAcc {
    last: Option<Integer>,
    cnt: usize,
    msg: String,
}

impl FormatAcc {
    fn new() -> Self {
        Self {
            last: None,
            cnt: 0,
            msg: String::from("")
        }
    }
}

fn format_vec(acc: FormatAcc, value: &Integer) -> FormatAcc {
    format_folder(acc, value, WaveFormat::Vector(0))
}

fn format_bitvec(acc: FormatAcc, value: &Integer) -> FormatAcc {
    format_folder(acc, value, WaveFormat::BitVector(0))
}

fn format_folder(mut acc: FormatAcc, value: &Integer, format: WaveFormat) -> FormatAcc {
    let emit;

    let val = match format {
        WaveFormat::BitVector(_) => format!("{:b}", *value),
        _ => format!("{:x}", *value)
    };
    let val_len = val.chars().count();

    if let Some(last) = acc.last {
        if last == *value {
            if acc.cnt >= val_len {
                emit = ' ';
            } else {
                emit = val.chars().nth(acc.cnt).unwrap();
            }

            acc.cnt += 1;
        } else {
            if (acc.cnt < val_len) && (acc.cnt > 0) {
                acc.msg.pop();
                acc.msg.push('…');
            }
            acc.cnt = 0;
            emit = '╳';
        }
    } else {
        emit = '╳';
    }

    acc.last = Some(value.clone());
    acc.msg.push(emit);

    acc
}
