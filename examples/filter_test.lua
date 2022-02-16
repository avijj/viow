wave = load_vcd("core.vcd", 100, "ps")

signals = {
	"tb.uut.ifu.i1_next_dword",
}

wave = replace_prefix(wave, 'tb_core.', 'tb.')
wave = grep(wave, [[.*uut.*]])
wave = filter_signals(wave, signals)
wave = remove_comments(wave)
