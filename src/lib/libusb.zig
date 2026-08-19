const c = @import("c");
const builtin = @import("builtin");

const CLibusbContext = c.struct_libusb_context;

pub const Context = struct {
    const Self = @This();
    libusb_ctx: *CLibusbContext,

    pub fn init() !Context {
        var self: Self = .{ .libusb_ctx = undefined };
        const options = [_]c.struct_libusb_init_option{
            .{
                .option = c.LIBUSB_OPTION_LOG_LEVEL,
                .value = .{ .ival = 
                    // if (builtin.mode == .Debug) 
                    //     c.LIBUSB_LOG_LEVEL_DEBUG 
                    // else 
                        c.LIBUSB_LOG_LEVEL_NONE
                },
            },
        };
        const result = c.libusb_init_context(@ptrCast(&self.libusb_ctx), &options, options.len);
        if (result != 0) {
            return error.LibusbInitializationFailed;
        }

        return self;
    }

    pub fn deinit(self: *const Self) void {
        c.libusb_exit(self.libusb_ctx);
    }
};
