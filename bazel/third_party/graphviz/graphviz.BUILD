load("@rules_cc//cc:defs.bzl", "cc_library")

package(default_visibility = ["//visibility:public"])

# Config header
# Expanded to support more features based on dependencies
genrule(
    name = "generate_config_h",
    outs = ["config.h"],
    cmd = "cat > $@ <<EOF\n" +
          "#define HAVE_STDLIB_H 1\n" +
          "#define HAVE_STRING_H 1\n" +
          "#define HAVE_UNISTD_H 1\n" +
          "#define HAVE_STDINT_H 1\n" +
          "#define HAVE_INTTYPES_H 1\n" +
          "#define HAVE_SYS_TYPES_H 1\n" +
          "#define HAVE_SYS_STAT_H 1\n" +
          "#define HAVE_FCNTL_H 1\n" +
          "#define HAVE_PTHREAD 1\n" +
          "#define HAVE_STRCASECMP 1\n" +
          "#define HAVE_STRNCASECMP 1\n" +
          "#define GVPLUGIN_VERSION 1\n" +
          "#define PACKAGE \"graphviz\"\n" +
          "#define PACKAGE_VERSION \"12.2.1\"\n" +
          "/* Dependencies */\n" +
          "#define HAVE_LIBGD 1\n" +
          "#define HAVE_CAIRO_H 1\n" +
          "#define DEFAULT_DPI 96\n" +
          "#define HAVE_EXPAT_H 1\n" +
          "#define HAVE_LIBZ 1\n" +
          "#define NONCONFIGURE 0\n" +
          "EOF",
)

genrule(
    name = "generate_builddate_h",
    outs = ["builddate.h"],
    cmd = "echo '#define BUILDDATE \"2026-08-28\"' > $@",
)

cc_library(
    name = "builddate_h",
    hdrs = [":generate_builddate_h"],
    includes = ["."],
)

cc_library(
    name = "config_h",
    hdrs = [":generate_config_h"],
    defines = [
        "HAVE_STDLIB_H=1",
        "HAVE_STRING_H=1",
        "HAVE_UNISTD_H=1",
        "HAVE_STDINT_H=1",
        "HAVE_INTTYPES_H=1",
        "HAVE_SYS_TYPES_H=1",
        "HAVE_SYS_STAT_H=1",
        "HAVE_FCNTL_H=1",
        "HAVE_PTHREAD=1",
        "HAVE_STRCASECMP=1",
        "HAVE_STRNCASECMP=1",
        "GVPLUGIN_VERSION=1",
        'PACKAGE=\\"graphviz\\"',
        'PACKAGE_VERSION=\\"12.2.1\\"',
        "HAVE_LIBGD=1",
        "HAVE_CAIRO_H=1",
        "DEFAULT_DPI=96",
        "HAVE_EXPAT_H=1",
        "HAVE_LIBZ=1",
        "NONCONFIGURE=0",
    ],
    includes = ["."],
)

# Util Library
cc_library(
    name = "util",
    srcs = glob(["lib/util/*.c"]),
    hdrs = glob(["lib/util/*.h"]),
    includes = [
        "lib",
        "lib/util",
    ],
    deps = [":config_h"],
)

# CDT Library
cc_library(
    name = "cdt",
    srcs = glob(["lib/cdt/*.c"]),
    hdrs = glob(["lib/cdt/*.h"]),
    includes = [
        "lib",
        "lib/cdt",
    ],
    deps = [
        ":config_h",
        ":util",
    ],
)

# Cgraph Library
cc_library(
    name = "cgraph",
    srcs = glob(
        ["lib/cgraph/*.c"],
        exclude = ["lib/cgraph/y.tab.c"],
    ),
    hdrs = glob(["lib/cgraph/*.h"]),
    includes = [
        "lib",
        "lib/cgraph",
    ],
    deps = [
        ":cdt",
        ":config_h",
        ":util",
    ],
)

# Pathplan Library
cc_library(
    name = "pathplan",
    srcs = glob(["lib/pathplan/*.c"]),
    hdrs = glob(["lib/pathplan/*.h"]),
    includes = [
        "lib",
        "lib/pathplan",
    ],
    deps = [
        ":cgraph",
        ":config_h",
        ":util",
    ],
)

# Xdot Library
cc_library(
    name = "xdot",
    srcs = glob(["lib/xdot/*.c"]),
    hdrs = glob(["lib/xdot/*.h"]),
    includes = [
        "lib",
        "lib/xdot",
    ],
    deps = [
        ":cgraph",
        ":config_h",
    ],
)

# RBTree Library
cc_library(
    name = "rbtree",
    srcs = glob(
        ["lib/rbtree/*.c"],
        exclude = ["lib/rbtree/test_red_black_tree.c"],
    ),
    hdrs = glob(["lib/rbtree/*.h"]),
    includes = [
        "lib",
        "lib/rbtree",
    ],
    deps = [
        ":config_h",
        ":util",
    ],
)

# Sparse Library
cc_library(
    name = "sparse",
    srcs = glob(["lib/sparse/*.c"]),
    hdrs = glob(["lib/sparse/*.h"]),
    includes = [
        "lib",
        "lib/sparse",
    ],
    deps = [
        ":cgraph",
        ":common_hdrs",
        ":config_h",
        ":util",
    ],
)

# Pack Library
cc_library(
    name = "pack",
    srcs = glob(["lib/pack/*.c"]),
    hdrs = glob(["lib/pack/*.h"]),
    includes = [
        "lib",
        "lib/pack",
    ],
    deps = [
        ":cgraph",
        ":common_hdrs",
        ":config_h",
        ":util",
    ],
)

# Common Headers
cc_library(
    name = "common_hdrs",
    hdrs = glob(["lib/**/*.h"]),
    includes = [
        "lib",
        "lib/common",
    ],
    deps = [
        ":cgraph",
        ":config_h",
        ":gvc_hdrs",
        ":pathplan",
        ":util",
        ":xdot",
        "@cairo",
        "@expat//:libexpat",
        "@glib//glib",
        "@zlib",
    ],
)

# Label Library
cc_library(
    name = "label",
    srcs = glob(["lib/label/*.c"]),
    hdrs = glob(["lib/label/*.h"]),
    includes = [
        "lib",
        "lib/label",
    ],
    deps = [
        ":cgraph",
        ":common_hdrs",
        ":config_h",
        ":pack",
        ":rbtree",
        ":sparse",
        ":util",
    ],
)

# GVC Headers
cc_library(
    name = "gvc_hdrs",
    hdrs = glob(["lib/gvc/*.h"]),
    includes = ["lib/gvc"],
    deps = [":builddate_h"],
)

# Common Library
cc_library(
    name = "common",
    srcs = glob(["lib/common/*.c"]),
    hdrs = glob(["lib/common/*.h"]),
    includes = [
        "lib",
        "lib/common",
    ],
    deps = [
        ":cgraph",
        ":common_hdrs",
        ":config_h",
        ":gvc_hdrs",
        ":label",
        ":pack",
        ":pathplan",
        ":util",
        ":xdot",
        "@cairo",
        "@expat//:libexpat",
        "@glib//glib",
        "@zlib",
    ],
)

# GVC Library
cc_library(
    name = "gvc",
    srcs = glob(["lib/gvc/*.c"]),
    hdrs = glob(["lib/gvc/*.h"]),
    includes = [
        "lib",
        "lib/gvc",
    ],
    deps = [
        ":builddate_h",
        ":cdt",
        ":cgraph",
        ":common",
        ":config_h",
        ":gvc_hdrs",
        ":pathplan",
        ":util",
        ":xdot",
    ],
)

# Dotgen Library
cc_library(
    name = "dotgen",
    srcs = glob(["lib/dotgen/*.c"]),
    hdrs = glob(["lib/dotgen/*.h"]),
    includes = [
        "lib",
        "lib/dotgen",
    ],
    deps = [
        ":cgraph",
        ":common",
        ":gvc",
    ],
)

# Neatogen Library
cc_library(
    name = "neatogen",
    srcs = glob(["lib/neatogen/*.c"]),
    hdrs = glob(["lib/neatogen/*.h"]),
    includes = [
        "lib",
        "lib/neatogen",
    ],
    deps = [
        ":cgraph",
        ":common",
        ":gvc",
    ],
)

# Core Plugin
cc_library(
    name = "plugin_core",
    srcs = glob(["plugin/core/*.c"]),
    hdrs = glob(
        ["plugin/core/*.h"],
        allow_empty = True,
    ),
    copts = [
        "-include",
        "unistd.h",
        "-Wno-implicit-function-declaration",
    ],
    includes = [
        "lib",
        "plugin/core",
    ],
    deps = [
        ":common",
        ":config_h",
        ":gvc",
    ],
)

# Dot Layout Plugin
cc_library(
    name = "plugin_dot_layout",
    srcs = glob(["plugin/dot_layout/*.c"]),
    hdrs = glob(
        ["plugin/dot_layout/*.h"],
        allow_empty = True,
    ),
    includes = [
        "lib",
        "plugin/dot_layout",
    ],
    deps = [
        ":common",
        ":config_h",
        ":dotgen",
        ":gvc",
    ],
)

cc_library(
    name = "cgraph_public_headers",
    hdrs = glob(["lib/cgraph/*.h"]),
    include_prefix = "graphviz",
    strip_include_prefix = "lib/cgraph",
)

cc_library(
    name = "gvc_public_headers",
    hdrs = glob(["lib/gvc/*.h"]),
    include_prefix = "graphviz",
    strip_include_prefix = "lib/gvc",
)

cc_library(
    name = "cdt_public_headers",
    hdrs = glob(["lib/cdt/*.h"]),
    include_prefix = "graphviz",
    strip_include_prefix = "lib/cdt",
)

cc_library(
    name = "pathplan_public_headers",
    hdrs = glob(["lib/pathplan/*.h"]),
    include_prefix = "graphviz",
    strip_include_prefix = "lib/pathplan",
)

cc_library(
    name = "xdot_public_headers",
    hdrs = glob(["lib/xdot/*.h"]),
    include_prefix = "graphviz",
    strip_include_prefix = "lib/xdot",
)

cc_library(
    name = "graphviz_public_headers",
    deps = [
        ":cdt_public_headers",
        ":cgraph_public_headers",
        ":gvc_public_headers",
        ":pathplan_public_headers",
        ":xdot_public_headers",
    ],
)

# GVC with Builtins
cc_library(
    name = "graphviz",
    deps = [
        ":graphviz_public_headers",
        ":gvc",
        ":plugin_core",
        ":plugin_dot_layout",
    ],
)
