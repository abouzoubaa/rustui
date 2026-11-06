# Stepper disabled label theme

Stepper now colors its visible label from the active platform profile: enabled labels use `primary_text`, and disabled labels use `disabled_text`, selected for the active light or dark appearance. The numeric value, button styling, glyph styling, and separator styling retain their existing behavior.

The focused source test checks both text tokens in both appearances. This establishes color selection from the resolved profile only; it does not establish native rendering or GUI acceptance.
