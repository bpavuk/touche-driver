this is a list of things that needs to happen before I promote this to v1

- [ ] wireless pairing
    - this also ties to the TUI - we need to display the QR code somehow!!
- [ ] proper TUI
    - perhaps could be done in Zig? I enjoyed ZigZag
- [ ] try out evdev-rs and see if it's any better
- [ ] investigate cursor flickering when tablet (re: phone with a stylus) stops
      emitting events
    - maybe the app could keep emitting last remembered event infinitely, and
        instead of pushing events as they arrive, just mutate the remembered
        event? feels weird, though...

