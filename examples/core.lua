require('pipeline')

wave = open("core.vcd", 100, "ps")

signals = {
	"tb_core.clk",
	"tb_core.reset",
	"tb_core.uut.ifu.i1_next_dword",
}

--wave = remove_comments(wave)
--wave = replace_prefix(wave, 'tb_core.', 'tb.')
--wave = grep(wave, [[.*uut.*]])
wave = pipeline(wave)
wave = filter_signals(wave, signals)
