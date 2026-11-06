# Stage 3 / Window Controls reference gap

The bundled `Default with Titlebar and Sidebar` references show two titlebar
control states: inactive 14 px translucent circles, and active close/minimize/
zoom controls with red `#FF5C60`, yellow `#FAC800`, and green `#35C759` fills.
The Playground preview currently renders the words Close, Minimize, and Zoom.

RustUI has no public Window Controls or titlebar renderer that can reproduce
these controls. `ThemeProfile.window_controls` stores token-name strings; it
does not provide a rendered control. The UI rules also reserve native chrome
for a separately approved phase. Keep this catalog entry partial until that
renderer is authorized and implemented.

The focused catalog regression preserves this distinction. No native or visual
acceptance was run.
