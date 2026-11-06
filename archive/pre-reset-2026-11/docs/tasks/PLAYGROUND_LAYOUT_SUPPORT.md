# Playground layout support

RustUI Playground uses the framework's public view API for its window and shell. `RustUI::window_size(width, height)` centers the initial native window, `RustUI::title(...)` sets its native title with a transparent titlebar, and `RustUI::blurred_window()` requests the platform's blurred window background.

`Container(view)` lowers to a GPUI flex column. Its optional `width`, `height`, `padding`, `radius`, `border`, and `background` builders style that box. Fixed dimensions resist flex shrinking. `Surface::Background`, `Surface::Sidebar`, and `Surface::Panel` resolve the active GPUI Components theme at render time; the border uses its border color. Container content inherits the active theme's foreground color and font size. These are semantic surface choices, not separate runtime theme packs.

`Container`, `HStack`, and `VStack` offer `fill()` for both parent dimensions and `grow()` for available room along the parent's flex axis. Both set zero minimum dimensions so nested scroll views can receive finite bounds when their parent supplies them. Stack spacing remains on the existing `spacing` builder.

These builders support layout composition only. Component selection, catalog content, and playground navigation belong to the application.
