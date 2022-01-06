use rug::Integer;

#[derive(Clone,Copy,PartialEq)]
pub enum WaveFormat {
    Bit,
    Vector,
    Comment,
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
        WaveFormat::Vector => build_waveform_vec(line_data, zoom),
        WaveFormat::Comment => build_waveform_comment(line_data, zoom),
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

fn format_vec(mut acc: FormatAcc, value: &Integer) -> FormatAcc {
    let emit;

    let val = format!("{:x}", *value);
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
