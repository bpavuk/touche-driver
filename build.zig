const std = @import("std");
const Translator = @import("translate_c").Translator;

// Although this function looks imperative, it does not perform the build
// directly and instead it mutates the build graph (`b`) that will be then
// executed by an external runner. The functions in `std.Build` implement a DSL
// for defining build steps and express dependencies between them, allowing the
// build runner to parallelize the build automatically (and the cache system to
// know when a step doesn't need to be re-run).
pub fn build(b: *std.Build) void {
    // Standard target options allow the person running `zig build` to choose
    // what target to build for. Here we do not override the defaults, which
    // means any target is allowed, and the default is native. Other options
    // for restricting supported target set are available.
    const target = b.standardTargetOptions(.{});
    // Standard optimization options allow the person running `zig build` to select
    // between Debug, ReleaseSafe, ReleaseFast, and ReleaseSmall. Here we do not
    // set a preferred release mode, allowing the user to decide how to optimize.
    const optimize = b.standardOptimizeOption(.{});
    // It's also possible to define more custom flags to toggle optional features
    // of this build script using `b.option()`. All defined flags (including
    // target and optimize options) will be listed when running `zig build --help`
    // in this directory.

    const cbor = b.dependency("cbor", .{});

    const translate_c = b.dependency("translate_c", .{});
    const translator: Translator = .init(translate_c, .{
        .c_source_file = b.path("src/lib/c.h"),
        .target = target,
        .optimize = optimize,
        .link_libc = true,
    });

    translator.linkSystemLibrary("libusb-1.0", .{ .needed = true, .preferred_link_mode = .dynamic });
    translator.linkSystemLibrary("libevdev", .{ .needed = true, .preferred_link_mode = .dynamic });

    const driverLib = b.addModule("touche-lib", .{
        .root_source_file = b.path("src/lib/lib.zig"),
        .target = target,
        .optimize = optimize,
        .imports = &.{
            .{ .name = "cbor", .module = cbor.module("cbor") },
            .{ .name = "c", .module = translator.mod },
        },
    });

    driverLib.linkSystemLibrary("libusb-1.0", .{ .needed = true, .preferred_link_mode = .dynamic });
    driverLib.linkSystemLibrary("libevdev", .{ .needed = true, .preferred_link_mode = .dynamic });

    const exeDriverCli = b.addExecutable(.{
        .name = "touche-cli",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/cli/main.zig"),
            .target = target,
            .optimize = optimize,
            .imports = &.{
                .{ .name = "driverLib", .module = driverLib },
            },
        }),
    });

    // This declares intent for the executable to be installed into the
    // install prefix when running `zig build` (i.e. when executing the default
    // step). By default the install prefix is `zig-out/` but can be overridden
    // by passing `--prefix` or `-p`.
    b.installArtifact(exeDriverCli);

    const exeDriverDesktop = b.addExecutable(.{
        .name = "touche-desktop",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/desktop/main.zig"),
            .target = target,
            .optimize = optimize,
            .imports = &.{
                .{ .name = "driverLib", .module = driverLib },
            }
        }),
    });

    b.installArtifact(exeDriverDesktop);

    // This creates a top level step. Top level steps have a name and can be
    // invoked by name when running `zig build` (e.g. `zig build run`).
    // This will evaluate the `run` step rather than the default step.
    // For a top level step to actually do something, it must depend on other
    // steps (e.g. a Run step, as we will see in a moment).
    const run_step = b.step("run", "Run the driver CLI (intended for debugging the driver)");

    const runDesktopStep = b.step("run-desktop", "Run the desktop app");

    // This creates a RunArtifact step in the build graph. A RunArtifact step
    // invokes an executable compiled by Zig. Steps will only be executed by the
    // runner if invoked directly by the user (in the case of top level steps)
    // or if another step depends on it, so it's up to you to define when and
    // how this Run step will be executed. In our case we want to run it when
    // the user runs `zig build run`, so we create a dependency link.
    const runCmd = b.addRunArtifact(exeDriverCli);
    run_step.dependOn(&runCmd.step);

    const runDesktopCmd = b.addRunArtifact(exeDriverDesktop);
    runDesktopStep.dependOn(&runDesktopCmd.step);

    // By making the run step depend on the default step, it will be run from the
    // installation directory rather than directly from within the cache directory.
    runCmd.step.dependOn(b.getInstallStep());

    runDesktopCmd.step.dependOn(b.getInstallStep());

    // This allows the user to pass arguments to the application in the build
    // command itself, like this: `zig build run -- arg1 arg2 etc`
    if (b.args) |args| {
        runCmd.addArgs(args);
    }

    // Creates an executable that will run `test` blocks from the executable's
    // root module. Note that test executables only test one module at a time,
    // hence why we have to create two separate ones.
    const exeTests = b.addTest(.{
        .root_module = exeDriverCli.root_module,
    });

    // A run step that will run the second test executable.
    const runExeTests = b.addRunArtifact(exeTests);

    // A top level step for running all tests. dependOn can be called multiple
    // times and since the two run steps do not depend on one another, this will
    // make the two of them run in parallel.
    const test_step = b.step("test", "Run tests");
    test_step.dependOn(&runExeTests.step);

    // Just like flags, top level steps are also listed in the `--help` menu.
    //
    // The Zig build system is entirely implemented in userland, which means
    // that it cannot hook into private compiler APIs. All compilation work
    // orchestrated by the build system will result in other Zig compiler
    // subcommands being invoked with the right flags defined. You can observe
    // these invocations when one fails (or you pass a flag to increase
    // verbosity) to validate assumptions and diagnose problems.
    //
    // Lastly, the Zig build system is relatively simple and self-contained,
    // and reading its source code will allow you to master it.
}
