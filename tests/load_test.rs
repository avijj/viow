use viow::{
    data::{SimTime, SimTimeUnit},
    wave::Wave,
    load::vcd::VcdLoader
};
use rug::Integer;
use std::path::PathBuf;


#[test]
fn load_vcd_test() {
    const FILE_NAME: &'static str = "examples/core.vcd";
    const CYCLE_TIME: SimTime = SimTime::new(100, SimTimeUnit::Ps);

    let loader = Box::new(VcdLoader::new(PathBuf::from(FILE_NAME), Some(CYCLE_TIME)).unwrap());
    let wave = Wave::load(loader).unwrap();

    assert_eq!(100, wave.num_cycles());
    assert_eq!(50, wave.num_signals());
    assert_eq!(Some("tb_core.mem.act_rd_q"), wave.name(11));
    assert_eq!(Some(&Integer::from(1)), wave.value(11, 13));
    assert_eq!(Some(&Integer::from(2)), wave.value(36, 37));

    {
        let one = Integer::from(1);
        let zero = Integer::from(0);
        let clk_vals = wave.slice_of_signal(0, 0, wave.num_cycles());

        for (cycle,val) in clk_vals.enumerate() {
            if cycle % 2 == 0 {
                assert_eq!(&zero, val);
            } else {
                assert_eq!(&one, val);
            }
        }
    }
}


#[test]
fn load_vcd_cycletime_test() {
    const FILE_NAME: &'static str = "examples/core.vcd";
    const CYCLE_TIME: SimTime = SimTime::new(50, SimTimeUnit::Ps);

    let loader = Box::new(VcdLoader::new(PathBuf::from(FILE_NAME), Some(CYCLE_TIME)).unwrap());
    let wave = Wave::load(loader).unwrap();

    assert_eq!(200, wave.num_cycles());
    assert_eq!(50, wave.num_signals());
    assert_eq!(Some("tb_core.mem.act_rd_q"), wave.name(11));
    assert_eq!(Some(&Integer::from(1)), wave.value(11, 13*2));
    assert_eq!(Some(&Integer::from(2)), wave.value(36, 37*2));

    {
        let one = Integer::from(1);
        let zero = Integer::from(0);
        let clk_vals = wave.slice_of_signal(0, 0, wave.num_cycles());

        for (cycle,val) in clk_vals.enumerate() {
            if cycle % 4 < 2 {
                assert_eq!(&zero, val);
            } else {
                assert_eq!(&one, val);
            }
        }
    }
}
