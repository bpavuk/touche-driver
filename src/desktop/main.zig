const std = @import("std");
const dvui = @import("dvui");

pub const dvui_app: dvui.App = .{
    .config = .{
        .options = .{
            .size = .{ .h = 600.0, .w = 800.0 },
            .title = "touché",
            .window_init_options = .{
                .theme = dvui.Theme.builtin.gruvbox,
            },
        },
    },
    .frameFn = frameFn,
    .deinitFn = deinitFn,
    .initFn = initFn,
};

pub const main = dvui.App.main;
pub const panic = dvui.App.panic;
pub const std_options: std.Options = .{
    .logFn = dvui.App.logFn,
};

pub fn initFn(win: *dvui.Window) !void {
    _ = win;

    // TODO: initialize the app here
}

pub fn deinitFn(win: *dvui.Window) void {
    _ = win;

    // TODO: deinitialize the app's resources here
}

pub fn frameFn() !dvui.App.Result {

    return .ok;
}
