const std = @import("std");

const startDriver = @import("driverLib").startDriver;

const log = std.log.scoped(.Main);

pub fn main(init: std.process.Init) !void {
    try startDriver(init.gpa, init.io);
}
