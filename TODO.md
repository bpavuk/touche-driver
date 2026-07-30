this is a list of things that needs to happen before I promote this to v1

- [ ] investigate cursor flickering when tablet (re: phone with a stylus) stops
      emitting events
    - maybe the app could keep emitting last remembered event infinitely, and
        instead of pushing events as they arrive, just mutate the remembered
        event? feels weird, though...
- [ ] rewrite the driver in Zig. it allows for much finer-grained resource
      control
