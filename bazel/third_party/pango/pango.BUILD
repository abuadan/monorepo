load("@rules_cc//cc:defs.bzl", "cc_library")

genrule(
    name = "config_h",
    outs = ["config.h"],
    cmd = """cat > $@ <<EOF
#define HAVE_DIRENT_H 1
#define HAVE_FCNTL_H 1
#define HAVE_STDLIB_H 1
#define HAVE_STRING_H 1
#define HAVE_UNISTD_H 1
#define HAVE_SYS_STAT_H 1
#define HAVE_SYS_TYPES_H 1
#define HAVE_CAIRO 1
#define HAVE_CAIRO_FREETYPE 1
#define HAVE_FREETYPE 1
#define HAVE_HARFBUZZ 1
/* #define HAVE_XFT 0 */
#define PACKAGE "pango"
#define PACKAGE_BUGREPORT "http://bugzilla.gnome.org/enter_bug.cgi?product=pango"
#define PACKAGE_NAME "pango"
#define PACKAGE_STRING "pango 1.42.4"
#define PACKAGE_TARNAME "pango"
#define PACKAGE_URL "http://www.pango.org"
#define PACKAGE_VERSION "1.42.4"
#define VERSION "1.42.4"
EOF""",
)

cc_library(
    name = "pango",
    srcs = glob(
        ["pango/*.c"],
        exclude = [
            "pango/pango-view.c",  # Tool source?
            "pango/test*.c",  # Tests
            "pango/pangoxft*.c",  # X11 backend
            "pango/pango-ot*.c",  # OpenType
            "pango/break-*.c",  # Included by break.c
            "pango/pangowin32*.c",  # Windows
        ],
    ),
    hdrs = glob([
        "pango/*.h",
        "pango/break-*.c",
    ]) + [":config_h"],
    copts = [
        "-DHAVE_CONFIG_H",
        "-DCAIRO_HAS_FT_FONT=1",
        "-I$(GENDIR)/external/pango",
        "-Iexternal/pango",
        "-DPANGO_ENABLE_BACKEND",
        "-DPANGO_ENABLE_ENGINE",
    ],
    includes = ["."],
    visibility = ["//visibility:public"],
    deps = [
        "@cairo",
        "@fontconfig",
        "@freetype",
        "@fribidi",
        "@glib//glib",
        "@glib//gobject",
        "@harfbuzz",
    ],
)
