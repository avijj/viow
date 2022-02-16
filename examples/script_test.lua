wave = load_vcd("core.vcd", 100, "ps")

-- this does nothing
-- On initial file load, data and terminal dimensions are not set until first render pass. That happens
-- only after this file is executed. So the cursor position is lost again.
-- TODO: Maybe trigger a render pass from within Lua. Alternatively, replace with a marker system
-- that is independent of dimensions. Or have callback after first render / resize
view.cursor_row = 1
view.cursor_col = 1
