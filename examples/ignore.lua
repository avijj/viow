wave = open("verilator.vcd", 1, 'ps')

allow_list = {
	[[.*_reg]]
}

deny_list = {
	[[.*hello.*]],
	[[.*reset.*]]
}

wave = allow_deny(wave, allow_list, deny_list)
