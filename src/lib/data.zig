pub const ToucheInputSerialized = union(enum) {
    @"ToucheInput.Stylus": struct {
        x: i32,
        y: i32,
        pressed: bool,
        pressure: f32,
    },

    @"ToucheInput.Finger": struct {
        x: i32,
        y: i32,
        pressed: bool,
        touchId: i64,
    },

    @"ToucheInput.Screen": struct {
        x: i32,
        y: i32,
    },

    @"ToucheInput.Action": enum {
        @"ToucheInput.Action.Init",
    },

    pub fn deserialize(self: *const ToucheInputSerialized) ToucheInput {
        return switch (self.*) {
            .@"ToucheInput.Stylus" => |value| .{ .Stylus = .{
                .x = value.x,
                .y = value.y,
                .pressed = value.pressed,
                .pressure = value.pressure,
            } },
            .@"ToucheInput.Finger" => |value| .{ .Finger = .{
                .x = value.x,
                .y = value.y,
                .pressed = value.pressed,
                .touchId = value.touchId,
            } },
            .@"ToucheInput.Screen" => |value| .{ .Screen = .{
                .x = value.x,
                .y = value.y,
            } },
            .@"ToucheInput.Action" => |value| switch (value) {
                .@"ToucheInput.Action.Init" => .{ .Action = .Init },
            },
        };
    }
};

pub const ToucheInput = union(enum) {
    Stylus: struct {
        x: i32,
        y: i32,
        pressed: bool,
        pressure: f32,
    },

    Finger: struct {
        x: i32,
        y: i32,
        pressed: bool,
        touchId: i64,
    },

    Screen: struct {
        x: i32,
        y: i32,
    },

    Action: enum {
        Init,
    },
};
