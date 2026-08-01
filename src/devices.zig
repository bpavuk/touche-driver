const std = @import("std");
const c = @import("c");
const dataModels = @import("data.zig");
const InputEvent = dataModels.ToucheInput;
const linux = std.os.linux;
const libevdev = c.libevdev;

const TabletDeviceData = struct {
    x: i32,
    y: i32,
    uidev: *c.libevdev_uinput,
};

pub const TabletDevice = struct {
    const Self = @This();
    const Data = TabletDeviceData;
    const log = std.log.scoped(.Tablet);
    _data: *anyopaque,

    pub fn init(alloc: std.mem.Allocator, io: std.Io, width: i32, height: i32) !TabletDevice {
        const data = try alloc.create(TabletDeviceData);
        errdefer alloc.destroy(data);

        const evdev = c.libevdev_new() orelse return error.FailedToCreateEvdevDevice;
        defer c.libevdev_free(evdev);
        libevdev.set_name(evdev, "touchetab");
        libevdev.set_id_version(evdev, 5);
        libevdev.set_id_product(evdev, 0x1234);
        libevdev.set_id_vendor(evdev, 0x5678);
        libevdev.set_id_bustype(evdev, c.BUS_USB);

        // SCARY: so many discards!

        _ = libevdev.enable_property(evdev, c.INPUT_PROP_DIRECT);

        _ = libevdev.enable_event_type(evdev, c.EV_KEY);
        _ = libevdev.enable_event_code(evdev, c.EV_KEY, c.BTN_TOOL_PEN, null);
        _ = libevdev.enable_event_code(evdev, c.EV_KEY, c.BTN_TOUCH, null);
        _ = libevdev.enable_event_code(evdev, c.EV_KEY, c.BTN_STYLUS, null);

        const absX: c.input_absinfo = .{
            .minimum = 0,
            .maximum = width,
            .resolution = 100,
            .value = 0,
            .fuzz = 0,
            .flat = 0,
        };

        const absY: c.input_absinfo = .{
            .minimum = 0,
            .maximum = height,
            .resolution = 100,
            .value = 0,
            .fuzz = 0,
            .flat = 0,
        };

        const absPressure: c.input_absinfo = .{
            .minimum = 0,
            .maximum = 4096,
            .resolution = 100,
            .value = 0,
            .fuzz = 0,
            .flat = 0,
        };

        const absDistance: c.input_absinfo = .{
            .minimum = 0,
            .maximum = 1024,
            .resolution = 100,
            .value = 0,
            .fuzz = 0,
            .flat = 0,
        };

        _ = libevdev.enable_event_type(evdev, c.EV_ABS);
        _ = libevdev.enable_event_code(evdev, c.EV_ABS, c.ABS_X, &absX);
        _ = libevdev.enable_event_code(evdev, c.EV_ABS, c.ABS_Y, &absY);
        _ = libevdev.enable_event_code(evdev, c.EV_ABS, c.ABS_PRESSURE, &absPressure);
        _ = libevdev.enable_event_code(evdev, c.EV_ABS, c.ABS_DISTANCE, &absDistance);

        var uidevNullable: ?*c.libevdev_uinput = undefined;

        const uidevRc = libevdev.uinput_create_from_device(evdev, c.LIBEVDEV_UINPUT_OPEN_MANAGED, &uidevNullable);
        if (uidevRc < 0) {
            log.debug("Failed to create uinput device: {s}", .{c.strerror(uidevRc)});
            return error.FailedToCreateUinputDevice;
        }
        var uidev: *c.libevdev_uinput = undefined;
        if (uidevNullable) |uidevExact| {
            uidev = uidevExact;
        } else {
            log.debug("uidevNullable is null for some reason!!", .{});
            return error.FailedToCreateUinputDevice;
        }

        try io.sleep(.fromMilliseconds(100), .real);

        // TODO: finish setting up a virtual tablet device

        data.x = width;
        data.y = height;
        data.uidev = uidev;

        const device: Self = .{
            ._data = data,
        };
        return device;
    }

    pub fn deinit(self: *const Self, alloc: std.mem.Allocator) void {
        const self_data: *Data = @ptrCast(@alignCast(self._data));

        c.libevdev_uinput_destroy(self_data.uidev);
        alloc.destroy(self_data);
    }

    pub fn emit(self: *const Self, events: []InputEvent) void {
        const data: *Data = @ptrCast(@alignCast(self._data));
        const writeEvent = c.libevdev_uinput_write_event;

        for (events) |event| {
            switch (event) {
                .Stylus => |value| {
                    // SCARY: discarding of input event emission error
                    _ = writeEvent(data.uidev, c.EV_KEY, c.BTN_TOOL_PEN, 1);
                    _ = writeEvent(data.uidev, c.EV_ABS, c.ABS_X, value.x);
                    _ = writeEvent(data.uidev, c.EV_ABS, c.ABS_Y, value.y);
                    _ = writeEvent(data.uidev, c.EV_ABS, c.ABS_PRESSURE, @trunc(value.pressure * 4096));
                    _ = writeEvent(data.uidev, c.EV_KEY, c.BTN_TOUCH, @intFromBool(value.pressed));

                    _ = writeEvent(data.uidev, c.EV_SYN, c.SYN_REPORT, 0);
                },
                else => {},
            }
        }
    }
};

const TouchpadDeviceData = struct {
    x: i32,
    y: i32,
    uidev: *c.libevdev_uinput,
};

pub const TouchpadDevice = struct {
    const Self = @This();
    const Data = TouchpadDeviceData;
    const log = std.log.scoped(.Touchpad);
    _data: *anyopaque,

    pub fn init(alloc: std.mem.Allocator, io: std.Io, width: i32, height: i32) !TouchpadDevice {
        log.debug("creating touchpad x: {d} y: {d}", .{ width, height });
        const data = try alloc.create(TouchpadDeviceData);
        errdefer alloc.destroy(data);

        const evdev = c.libevdev_new() orelse return error.FailedToCreateEvdevDevice;
        defer c.libevdev_free(evdev);
        libevdev.set_name(evdev, "touchepad");
        libevdev.set_id_version(evdev, 1);
        libevdev.set_id_product(evdev, 0x0002);
        libevdev.set_id_vendor(evdev, 0x5120);
        libevdev.set_id_bustype(evdev, c.BUS_USB);

        // SCARY: so many discards!

        _ = libevdev.enable_property(evdev, c.INPUT_PROP_POINTER);

        _ = libevdev.enable_event_type(evdev, c.EV_KEY);
        _ = libevdev.enable_event_code(evdev, c.EV_KEY, c.BTN_TOUCH, null);
        _ = libevdev.enable_event_code(evdev, c.EV_KEY, c.BTN_TOOL_FINGER, null);
        _ = libevdev.enable_event_code(evdev, c.EV_KEY, c.BTN_TOOL_DOUBLETAP, null);
        _ = libevdev.enable_event_code(evdev, c.EV_KEY, c.BTN_TOOL_TRIPLETAP, null);
        _ = libevdev.enable_event_code(evdev, c.EV_KEY, c.BTN_TOOL_QUADTAP, null);
        _ = libevdev.enable_event_code(evdev, c.EV_KEY, c.BTN_TOOL_QUINTTAP, null);

        const absX: c.input_absinfo = .{
            .minimum = 0,
            .maximum = width,
            .resolution = 50,
            .value = 0,
            .fuzz = 8,
            .flat = 0,
        };

        const absY: c.input_absinfo = .{
            .minimum = 0,
            .maximum = height,
            .resolution = 50,
            .value = 0,
            .fuzz = 8,
            .flat = 0,
        };

        const absMtSlot: c.input_absinfo = .{
            .minimum = 0,
            .maximum = 19,
            .resolution = 50,
            .value = 0,
            .fuzz = 0,
            .flat = 0,
        };

        const absMtTrackingId: c.input_absinfo = .{
            .minimum = 0,
            .maximum = 65535,
            .resolution = 0,
            .value = 0,
            .fuzz = 0,
            .flat = 0,
        };

        const absMtPositionX: c.input_absinfo = .{
            .minimum = 0,
            .maximum = width,
            .resolution = 50,
            .value = 0,
            .fuzz = 8,
            .flat = 0,
        };

        const absMtPositionY: c.input_absinfo = .{
            .minimum = 0,
            .maximum = height,
            .resolution = 50,
            .value = 0,
            .fuzz = 8,
            .flat = 0,
        };

        _ = libevdev.enable_event_type(evdev, c.EV_ABS);
        _ = libevdev.enable_event_code(evdev, c.EV_ABS, c.ABS_MT_SLOT, &absMtSlot);
        _ = libevdev.enable_event_code(evdev, c.EV_ABS, c.ABS_MT_TRACKING_ID, &absMtTrackingId);
        _ = libevdev.enable_event_code(evdev, c.EV_ABS, c.ABS_MT_POSITION_X, &absMtPositionX);
        _ = libevdev.enable_event_code(evdev, c.EV_ABS, c.ABS_MT_POSITION_Y, &absMtPositionY);
        _ = libevdev.enable_event_code(evdev, c.EV_ABS, c.ABS_X, &absX);
        _ = libevdev.enable_event_code(evdev, c.EV_ABS, c.ABS_Y, &absY);

        var uidevNullable: ?*c.libevdev_uinput = undefined;

        const uidevRc = libevdev.uinput_create_from_device(evdev, c.LIBEVDEV_UINPUT_OPEN_MANAGED, &uidevNullable);
        if (uidevRc < 0) {
            log.debug("Failed to create uinput device: {s}", .{c.strerror(uidevRc)});
            return error.FailedToCreateUinputDevice;
        }
        var uidev: *c.libevdev_uinput = undefined;
        if (uidevNullable) |uidevExact| {
            uidev = uidevExact;
        } else {
            log.debug("uidevNullable is null for some reason!!", .{});
            return error.FailedToCreateUinputDevice;
        }

        try io.sleep(.fromMilliseconds(100), .real);

        data.x = width;
        data.y = height;
        data.uidev = uidev;

        const device: Self = .{
            ._data = data,
        };
        return device;
    }

    pub fn deinit(self: *const Self, alloc: std.mem.Allocator) void {
        const self_data: *Data = @ptrCast(@alignCast(self._data));

        c.libevdev_uinput_destroy(self_data.uidev);
        alloc.destroy(self_data);
    }

    pub fn emit(self: *const Self, events: []InputEvent) void {
        const data: *Data = @ptrCast(@alignCast(self._data));
        const writeEvent = c.libevdev_uinput_write_event;

        var fingers: u8 = 0;
        var fingerEventHappened: bool = false;
        for (events) |event| {
            switch (event) {
                .Finger => |value| {
                    // SCARY: discarding of input event emission error
                    const mtSlot: c_int = @intCast(@mod(value.touchId, 20));
                    log.debug("mt slot {d}, x {d}", .{mtSlot, value.x});
                    _ = writeEvent(data.uidev, c.EV_ABS, c.ABS_MT_SLOT, mtSlot);
                    if (value.pressed) {
                        fingers += 1;
                        _ = writeEvent(data.uidev, c.EV_ABS, c.ABS_MT_POSITION_X, value.x);
                        _ = writeEvent(data.uidev, c.EV_ABS, c.ABS_MT_POSITION_Y, value.y);
                    }
                    _ = writeEvent(
                        data.uidev,
                        c.EV_ABS,
                        c.ABS_MT_TRACKING_ID,
                        if (value.pressed) @intCast(value.touchId + 1) else -1,
                    );

                    fingerEventHappened = true;
                },
                else => {},
            }
        }
        if (fingerEventHappened) {
            _ = writeEvent(data.uidev, c.EV_KEY, c.BTN_TOUCH, @intFromBool(fingers > 0 and fingers <= 5));
            _ = writeEvent(data.uidev, c.EV_KEY, c.BTN_TOOL_FINGER, @intFromBool(fingers == 1));
            _ = writeEvent(data.uidev, c.EV_KEY, c.BTN_TOOL_DOUBLETAP, @intFromBool(fingers == 2));
            _ = writeEvent(data.uidev, c.EV_KEY, c.BTN_TOOL_TRIPLETAP, @intFromBool(fingers == 3));
            _ = writeEvent(data.uidev, c.EV_KEY, c.BTN_TOOL_QUADTAP, @intFromBool(fingers == 4));
            _ = writeEvent(data.uidev, c.EV_KEY, c.BTN_TOOL_QUINTTAP, @intFromBool(fingers == 5));
            _ = writeEvent(data.uidev, c.EV_SYN, c.SYN_REPORT, 0);

            log.debug("fingers: {d}", .{fingers});
        }
    }
};

const DeviceOptions = struct {
    vendor: u16,
    product: u16,
    version: u16,
    name: *[80]u8,
};
