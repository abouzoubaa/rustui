# Stepper localized action names

Stepper's derived increment and decrement accessible names now resolve through
the active message catalog, with English fallback, and include the localized
stepper name. The RustUI English, French, and Arabic catalogs provide these
messages. The focused unit test covers message lookup and interpolation; native
screen-reader announcement acceptance remains open.
