const std = @import("std");

const AoaDevice = @import("aoa.zig");
const data = @import("data.zig");
const cbor = @import("cbor");
const devices = @import("devices.zig");
const libusb = @import("libusb.zig");
const log = std.log.scoped(.Main);

pub fn main(init: std.process.Init) !void {
    var libusbContext = try libusb.Context.init();
    defer libusbContext.deinit();
    try startDriver(&libusbContext, init.gpa, init.io);
}

fn uinputFuckery(alloc: std.mem.Allocator, io: std.Io) !void {
    const device = try devices.TabletDevice.init(alloc, io, 1920, 1080);
    defer device.deinit(alloc, io);

    device.emit(.{ .Action = .Init }, io);
}

const DriverState = struct {
    const Self = @This();

    tablet: ?devices.TabletDevice,
    touchpad: ?devices.TouchpadDevice,

    fn deinit(self: *Self) void {
        if (self.tablet) |tab| {
            tab.deinit();
        }
    }
};

fn startDriver(ctx: *libusb.Context, alloc: std.mem.Allocator, io: std.Io) !void {
    const aoa = try AoaDevice.init(ctx, alloc, io);
    defer aoa.deinit();

    var state: DriverState = .{
        .tablet = null,
        .touchpad = null,
    };

    while (true) {
        log.debug("waiting for AOA device to be connected...", .{});
        try aoa.waitForDevice();
        log.debug("AOA device connected. starting the driver loop...", .{});

        var rxBuffer: [512]u8 = undefined;
        var fbaBuffer: [4096]u8 = undefined;
        var fba = std.heap.FixedBufferAllocator.init(&fbaBuffer);
        while (true) {
            defer fba.reset();
            const bytesRead = aoa.read(&rxBuffer, 1000) catch |err| switch (err) {
                error.Timeout => continue,
                error.Disconnected => {
                    log.debug("device disconnnected", .{});
                    break;
                },
                else => return err,
            };
            log.debug("received {d} bytes from Android", .{bytesRead});

            if (!(try cbor.match(rxBuffer[0..bytesRead], cbor.array))) {
                log.err("Invalid data received. Expected CBOR array.", .{});
                continue;
            }
            // from now on we know this is an array

            // 1. find out array's size
            var iter: []const u8 = rxBuffer[0..bytesRead];
            const header = try cbor.decodeArrayHeader(&iter);
            if (header >= 20) {
                log.err("The driver supports maximum 20 input events at a time, e.g. 20 fingers. Skipping.", .{});
                continue;
            }

            // 2. iterate over an array
            var list = try std.ArrayList(data.ToucheInput).initCapacity(fba.allocator(), header);
            errdefer list.deinit(fba.allocator());

            var i: usize = 0;
            while (i < header) : (i += 1) {
                // 3. deserialize ToucheEventSerialized
                var raw: data.ToucheInputSerialized = undefined;
                const matched = try cbor.match(iter, cbor.extract(&raw));
                try cbor.skipValue(&iter);
                if (!matched) {
                    log.err("One of events is not valid touché input.", .{});
                    continue;
                }
                // 4. write them into an ArrayList
                list.appendAssumeCapacity(raw.deserialize());
            }

            const events = list.toOwnedSliceAssert();
            try emitDriverEvents(&state, alloc, io, events);
        }
    }
}

fn emitDriverEvents(
    state: *DriverState,
    alloc: std.mem.Allocator,
    io: std.Io,
    events: []data.ToucheInput,
) !void {
    if (state.touchpad) |touchpad| {
        touchpad.emit(events);
    }
    if (state.tablet) |tablet| {
        tablet.emit(events);
    }
    for (events) |event| {
        switch (event) {
            .Action => |value| switch (value) {
                .Init => {
                    if (state.touchpad) |touchpad| {
                        touchpad.deinit(alloc);
                    }
                    if (state.tablet) |tablet| {
                        tablet.deinit(alloc);
                    }
                    state.touchpad = null;
                    state.tablet = null;
                    break;
                },
            },
            .Screen => |value| {
                if (state.touchpad) |touchpad| {
                    touchpad.deinit(alloc);
                }
                if (state.tablet) |tablet| {
                    tablet.deinit(alloc);
                }

                state.tablet = try .init(alloc, io, value.x, value.y);
                errdefer state.tablet.?.deinit(alloc);
                state.touchpad = try .init(alloc, io, value.x, value.y);
                break;
            },
            else => {},
        }
    }
}
