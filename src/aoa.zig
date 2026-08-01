const std = @import("std");

const c = @import("c");
const UsbContext = @import("libusb.zig").Context;

const GOOGLE_VID = 0x18D1;
const AOA_PID_MIN = 0x2D00;
const AOA_PID_MAX = 0x2D05;

const Self = @This();
const log = std.log.scoped(.AoaDriver);

//=============================================================================
// 1. FIELDS AND LIFECYCLE
//=============================================================================

io: std.Io,
alloc: std.mem.Allocator,
mutex: std.Io.Mutex = std.Io.Mutex.init,
should_exit: std.atomic.Value(bool) = std.atomic.Value(bool).init(false),

queue_cond: std.Io.Condition = .init,
conn_cond: std.Io.Condition = .init,

event_thread: ?std.Thread = null,
worker_thread: ?std.Thread = null,

ctx: *UsbContext,
hotplug_handle: c.libusb_hotplug_callback_handle = 0,
bulk_in_ep: u8 = undefined,

is_connected: bool = false,
active_device: ?*c.libusb_device_handle = null,
pending_devices: std.ArrayListUnmanaged(*c.libusb_device) = undefined,

/// Initializes the driver and worker threads.
pub fn init(ctx: *UsbContext, alloc: std.mem.Allocator, io: std.Io) !*Self {
    // I can't imagine anyone having more than 128 USB devices connected.
    const devicesList = try std.ArrayListUnmanaged(*c.libusb_device).initCapacity(alloc, 128);
    const self = try alloc.create(Self);
    self.* = .{
        .io = io,
        .alloc = alloc,
        .ctx = ctx,
        .pending_devices = devicesList,
    };

    const rc = c.libusb_hotplug_register_callback(
        self.ctx.libusb_ctx,
        c.LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED | c.LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT,
        c.LIBUSB_HOTPLUG_ENUMERATE,
        c.LIBUSB_HOTPLUG_MATCH_ANY,
        c.LIBUSB_HOTPLUG_MATCH_ANY,
        c.LIBUSB_HOTPLUG_MATCH_ANY,
        hotplugCallback,
        @ptrCast(self),
        &self.hotplug_handle,
    );

    if (rc != c.LIBUSB_SUCCESS) {
        return error.FailedToRegisterCallback;
    }

    self.event_thread = try std.Thread.spawn(.{}, libusbEventLoop, .{self});
    self.worker_thread = try std.Thread.spawn(.{}, runDriverLoopThread, .{self});

    return self;
}

pub fn deinit(self: *Self) void {
    self.should_exit.store(true, .release);

    {
        self.mutex.lock(self.io) catch {
            log.debug(
                "Failed to lock mutex during device deinitialization. Proceeding anyway.",
                .{},
            );
        };
        defer self.mutex.unlock(self.io);
        self.queue_cond.signal(self.io);
        self.conn_cond.signal(self.io);
    }

    if (self.event_thread) |thread| {
        thread.join();
    }

    if (self.hotplug_handle != 0) {
        c.libusb_hotplug_deregister_callback(self.ctx.libusb_ctx, self.hotplug_handle);
    }

    self.mutex.lock(self.io) catch {
        log.debug(
            "Failed to lock mutex during device deinitialization. Proceeding anyway.",
            .{},
        );
    };
    defer self.mutex.unlock(self.io);

    for (self.pending_devices.items) |dev| {
        c.libusb_unref_device(dev);
    }
    self.pending_devices.deinit(self.alloc);

    self.stopActiveDevice();
    self.alloc.destroy(self);
}

//=============================================================================
// 2. PUBLIC API
//=============================================================================

pub fn waitForDevice(self: *Self) !void {
    try self.mutex.lock(self.io);
    defer self.mutex.unlock(self.io);

    while (!self.is_connected and !self.should_exit.load(.acquire)) {
        try self.conn_cond.wait(self.io, &self.mutex);
    }

    if (self.should_exit.load(.acquire)) return error.Interrupted;
}

pub fn read(self: *Self, buffer: []u8, timeoutMs: u32) !usize {
    const handle, const epIn = block: {
        try self.mutex.lock(self.io);
        defer self.mutex.unlock(self.io);

        if (!self.is_connected or self.active_device == null) {
            return error.Disconnected;
        }
        break :block .{ self.active_device.?, self.bulk_in_ep };
    };

    var transferred: c_int = 0;
    const rc = c.libusb_bulk_transfer(
        handle,
        epIn,
        buffer.ptr,
        @intCast(buffer.len),
        &transferred,
        timeoutMs,
    );

    if (rc == c.LIBUSB_SUCCESS) {
        return @intCast(transferred);
    }

    return switch (rc) {
        c.LIBUSB_ERROR_TIMEOUT => error.Timeout,
        c.LIBUSB_ERROR_NO_DEVICE, c.LIBUSB_ERROR_IO, c.LIBUSB_ERROR_PIPE => {
            self.stopActiveDevice();
            return error.Disconnected;
        },
        else => error.LibusbReadFailed,
    };
}

//=============================================================================
// 3. INTERNAL ENGINE THREADS
//=============================================================================

fn runDriverLoopThread(self: *Self) void {
    self.runDriverLoop() catch |err| {
        log.debug("Error in driver loop: {any}", .{err});
    };
}

fn runDriverLoop(self: *Self) !void {
    while (!self.should_exit.load(.acquire)) {
        var devToProcess: ?*c.libusb_device = null;

        {
            try self.mutex.lock(self.io);
            defer self.mutex.unlock(self.io);

            while (self.pending_devices.items.len == 0 and !self.should_exit.load(.acquire)) {
                try self.queue_cond.wait(self.io, &self.mutex);
            }

            if (self.pending_devices.items.len > 0) {
                devToProcess = self.pending_devices.orderedRemove(0);
            }
        }

        if (devToProcess) |dev| {
            defer c.libusb_unref_device(dev);
            try self.handleDeviceEvent(dev);
        }
    }
}

fn libusbEventLoop(self: *Self) void {
    var tv: c.timeval = .{
        .tv_sec = 0,
        .tv_usec = 250_000,
    };
    while (!self.should_exit.load(.acquire)) {
        _ = c.libusb_handle_events_timeout_completed(self.ctx.libusb_ctx, &tv, null);
    }
    log.debug("Exiting because we should exit", .{});
}

//============================================================================
// 4. USB PROTOCOL AND STATE MACHINE
//============================================================================

fn stopActiveDevice(self: *Self) void {
    if (self.active_device) |handle| {
        log.debug("Closing the device", .{});
        _ = c.libusb_release_interface(handle, 0);
        c.libusb_close(handle);
        self.active_device = null;
    }
    self.is_connected = false;

    self.conn_cond.signal(self.io);
}

fn handleDeviceEvent(self: *Self, device: *c.struct_libusb_device) !void {
    var desc: c.libusb_device_descriptor = undefined;
    if (c.libusb_get_device_descriptor(device, &desc) != c.LIBUSB_SUCCESS) {
        return;
    }

    if (desc.idVendor == GOOGLE_VID and desc.idProduct >= AOA_PID_MIN and desc.idProduct <= AOA_PID_MAX) {
        if (!self.is_connected) {
            log.debug("[driver] Connecting to AOA device...", .{});
            self.attachAoaDevice(device);
        }
    } else if (!self.is_connected) {
        self.tryInitializeAoa(device);
    }
}

fn tryInitializeAoa(self: *Self, dev: *c.libusb_device) void {
    if (self.is_connected) return;
    var handle: ?*c.libusb_device_handle = null;

    const openRc = c.libusb_open(dev, &handle);
    if (openRc != c.LIBUSB_SUCCESS) {
        log.debug("[AOA] Failed to open device handle", .{});
        return;
    }
    defer c.libusb_close(handle);

    const reqTypeIn = c.LIBUSB_ENDPOINT_IN | c.LIBUSB_REQUEST_TYPE_VENDOR | c.LIBUSB_RECIPIENT_DEVICE;
    const reqTypeOut = c.LIBUSB_ENDPOINT_OUT | c.LIBUSB_REQUEST_TYPE_VENDOR | c.LIBUSB_RECIPIENT_DEVICE;

    var protocolVersion: u16 = 0;
    const protoRc = c.libusb_control_transfer(
        handle,
        reqTypeIn,
        51,
        0,
        0,
        @ptrCast(&protocolVersion),
        @sizeOf(u16),
        1000,
    );

    if (protoRc < 0 or protocolVersion < 1) {
        log.debug("[AOA] Device does not support protocol version v{d}", .{protocolVersion});
        return;
    }

    const strings = [_][:0]const u8{
        "bpavuk",
        "touche",
        "making your phone a touchepad and graphics tablet",
        "v0", // TODO: switch to v1 when driver is done
        "what://",
        "528491",
    };

    for (strings, 0..) |str, idx| {
        const strRc = c.libusb_control_transfer(
            handle,
            reqTypeOut,
            52,
            0,
            @intCast(idx),
            @ptrCast(@constCast(str.ptr)),
            @intCast(str.len + 1),
            1000,
        );

        if (strRc < 0) {
            log.debug("[AOA] Failed to send identity string {d}: {s}", .{
                idx,
                c.libusb_strerror(strRc),
            });
            return;
        }
    }

    const startRc = c.libusb_control_transfer(
        handle,
        reqTypeOut,
        53,
        0,
        0,
        null,
        0,
        1000,
    );

    if (startRc < 0) {
        log.debug("[AOA] Failed to trigger accessory start: {s}", .{
            c.libusb_strerror(startRc),
        });
    }

    log.debug("Device should re-enumerate in AOA mode now.", .{});
}

fn attachAoaDevice(self: *Self, dev: *c.libusb_device) void {
    var handle: ?*c.libusb_device_handle = null;
    if (c.libusb_open(dev, &handle) != c.LIBUSB_SUCCESS) {
        return;
    }

    if (c.libusb_claim_interface(handle, 0) != c.LIBUSB_SUCCESS) {
        c.libusb_close(handle);
        return;
    }

    self.bulk_in_ep = discoverBulkInEndpoint(dev) orelse 0x81;
    self.active_device = handle;
    self.is_connected = true;
    log.debug("[Driver] Device claimed successfully", .{});

    self.conn_cond.broadcast(self.io);
}

//=============================================================================
// C INTEROP AND HELPERS
//=============================================================================

fn hotplugCallback(
    ctx: ?*c.libusb_context,
    device: ?*c.libusb_device,
    event: c_uint,
    user_data: ?*anyopaque,
) callconv(.c) c_int {
    const callbackLog = std.log.scoped(.hotplugCallback);
    callbackLog.debug("Hotplug callback received event", .{});
    _ = ctx;
    const self: *Self = @ptrCast(@alignCast(user_data.?));
    switch (event) {
        c.LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED => {
            if (device) |dev| {
                log.debug("Received pending device", .{});
                self.mutex.lock(self.io) catch {
                    log.debug("Something cancelled mutex lock\n", .{});
                    return 0;
                };
                defer self.mutex.unlock(self.io);

                _ = c.libusb_ref_device(dev);
                self.pending_devices.append(self.alloc, dev) catch |e| {
                    log.warn("Failed to put a device into a queue because .{any}", .{ e });
                    c.libusb_unref_device(dev);
                    return 0;
                };

                self.queue_cond.signal(self.io);
            }
        },
        c.LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT => {
            if (device) |dev| {
                log.debug("A USB device left", .{});
                self.mutex.lock(self.io) catch {
                    log.debug("Something cancelled mutex lock", .{});
                    return 0;
                };
                defer self.mutex.unlock(self.io);

                if (self.active_device) |handle| {
                    if (c.libusb_get_device(handle) == dev) {
                        log.debug("This was AOA device that driver held. Not doing anything about it.", .{});
                    }
                }

                self.queue_cond.signal(self.io);
            }
        },
        else => {},
    }
    return 0;
}

fn discoverBulkInEndpoint(device: *c.struct_libusb_device) ?u8 {
    var config: ?*c.libusb_config_descriptor = null;
    const confRc = c.libusb_get_active_config_descriptor(device, &config);
    if (confRc != c.LIBUSB_SUCCESS) {
        return null;
    }
    defer c.libusb_free_config_descriptor(config);

    if (config.?.bNumInterfaces == 0) return null;
    const interface = config.?.interface[0];
    if (interface.num_altsetting == 0) return null;
    const alt = interface.altsetting[0];

    for (alt.endpoint[0..alt.bNumEndpoints]) |ep| {
        const isBulk = (ep.bmAttributes & c.LIBUSB_TRANSFER_TYPE_MASK) == c.LIBUSB_TRANSFER_TYPE_BULK;
        const isIn = (ep.bEndpointAddress & c.LIBUSB_ENDPOINT_IN) != 0;
        if (isBulk and isIn) {
            return ep.bEndpointAddress;
        }
    }

    return null;
}
