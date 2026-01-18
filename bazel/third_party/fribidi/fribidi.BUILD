load("@rules_cc//cc:defs.bzl", "cc_library")

# Minimal config.h generation
genrule(
    name = "config_h",
    outs = ["config.h"],
    cmd = """cat > $@ <<EOF
#define HAVE_STDLIB_H 1
#define HAVE_STRING_H 1
#define HAVE_STRINGS_H 1
#define HAVE_MEMORY_H 1
#define HAVE_STRINGIZE 1
#define FRIBIDI_NO_DEPRECATED 0
#define SIZEOF_INT 4
EOF""",
)

# Generate fribidi-config.h
genrule(
    name = "fribidi_config_h",
    outs = ["lib/fribidi-config.h"],
    cmd = """cat > $@ <<EOF
#ifndef FRIBIDI_CONFIG_H
#define FRIBIDI_CONFIG_H
#define FRIBIDI "1.0.12"
#define FRIBIDI_BUGREPORT "https://github.com/fribidi/fribidi/issues"
#define FRIBIDI_INTERFACE_VERSION 0
#define FRIBIDI_INTERFACE_VERSION_STRING "0"
#define FRIBIDI_MAJOR_VERSION 1
#define FRIBIDI_MINOR_VERSION 0
#define FRIBIDI_MICRO_VERSION 12
#define FRIBIDI_VERSION "1.0.12"
#endif /* FRIBIDI_CONFIG_H */
EOF""",
)

cc_library(
    name = "fribidi",
    srcs = glob(
        ["lib/*.c"],
        exclude = ["lib/*main.c"],
    ),
    hdrs = glob([
        "lib/*.h",
        "lib/*.tab.i",
    ]) + [
        ":config_h",
        ":fribidi_config_h",
    ],
    copts = ["-DHAVE_CONFIG_H"],
    includes = [
        ".",
        "lib",
    ],
    visibility = ["//visibility:public"],
)
