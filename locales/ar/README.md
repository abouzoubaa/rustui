# Arabic locale

Framework messages are in `rustui.ftl`; `locale.json` declares the active runtime catalog
format and right-to-left logical layout direction. RustUI reverses HStack flow and maps
VStack start/end alignment for RTL. GPUI remains responsible for glyph shaping.
