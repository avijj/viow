function load_it(wave)
	wave = load('2Mx2k.hello', 1, 'ps')

	signals = {
		'signal_4747',
		'signal_288',
		'signal_0'
	}

	wave = filter_signals(wave, signals)
	return wave
end

wave = load_it(wave)
