# package(default_visibility = ["//visibility:public"])
#
# # Config header
# # Expanded to support more features based on dependencies
# genrule(
#     name = "generate_config_h",
#     outs = ["config.h"],
#     cmd = "cat > $@ <<EOF\n" +
#           "#define HAVE_STDLIB_H 1\n" +
#           "#define HAVE_STRING_H 1\n" +
#           "#define HAVE_UNISTD_H 1\n" +
#           "#define HAVE_STDINT_H 1\n" +
#           "#define HAVE_INTTYPES_H 1\n" +
#           "#define HAVE_SYS_TYPES_H 1\n" +
#           "#define HAVE_SYS_STAT_H 1\n" +
#           "#define HAVE_FCNTL_H 1\n" +
#           "#define HAVE_PTHREAD 1\n" +
#           "#define HAVE_STRCASECMP 1\n" +
#           "#define HAVE_STRNCASECMP 1\n" +
#           "#define GVPLUGIN_VERSION 1\n" +
#           "#define PACKAGE \"graphviz\"\n" +
#           "#define PACKAGE_VERSION \"12.2.1\"\n" +
#           "/* Dependencies */\n" +
#           "#define HAVE_LIBGD 1\n" +
#           "#define HAVE_CAIRO_H 1\n" +
#           "#define DEFAULT_DPI 96\n" +
#           # "#define HAVE_PANGOCAIRO_H 1\n" +
#           "#define HAVE_EXPAT_H 1\n" +
#           "#define HAVE_LIBZ 1\n" +
#           '# define NONCONFIGURE 0\n' +
#           # Static plugin registration macros (for gv_builtins) can be handled in code config if needed
#           "EOF",
# )
#
# cc_library(
#     name = "config_h",
#     hdrs = [":generate_config_h"],
#     includes = ["."],
# )
#
# # Util Library
# cc_library(
#     name = "util",
#     srcs = glob(["lib/util/*.c"]),
#     hdrs = glob(["lib/util/*.h"]),
#     includes = ["lib/util", "lib"],
#     deps = [":config_h"],
# )
#
# # CDT Library
# cc_library(
#     name = "cdt",
#     srcs = glob(["lib/cdt/*.c"]),
#     hdrs = glob(["lib/cdt/*.h"]),
#     includes = ["lib/cdt", "lib"],
#     deps = [":config_h", ":util"],
# )
#
# # Cgraph Library
# cc_library(
#     name = "cgraph",
#     srcs = glob(["lib/cgraph/*.c"], exclude = ["lib/cgraph/y.tab.c"]), # generated parsers might be needed
#     hdrs = glob(["lib/cgraph/*.h"]),
#     includes = ["lib/cgraph", "lib"],
#     deps = [":cdt", ":config_h", ":util"],
# )
#
# # Pathplan Library
# cc_library(
#     name = "pathplan",
#     srcs = glob(["lib/pathplan/*.c"]),
#     hdrs = glob(["lib/pathplan/*.h"]),
#     includes = ["lib/pathplan", "lib"],
#     deps = [":cgraph", ":config_h", ":util"],
# )
#
# # Xdot Library
# cc_library(
#     name = "xdot",
#     srcs = glob(["lib/xdot/*.c"]),
#     hdrs = glob(["lib/xdot/*.h"]),
#     includes = ["lib/xdot", "lib"],
#     deps = [":cgraph", ":config_h"],
# )
#
# # RBTree Library
# cc_library(
#     name = "rbtree",
#     srcs = glob(["lib/rbtree/*.c"]),
#     hdrs = glob(["lib/rbtree/*.h"]),
#     includes = ["lib/rbtree", "lib"],
#     deps = [":config_h", ":util"],
# )
#
# # Sparse Library
# cc_library(
#     name = "sparse",
#     srcs = glob(["lib/sparse/*.c"]),
#     hdrs = glob(["lib/sparse/*.h"]),
#     includes = ["lib/sparse", "lib"],
#     deps = [":cgraph", ":config_h", ":util", ":common_hdrs"],
# )
#
# # Pack Library
# cc_library(
#     name = "pack",
#     srcs = glob(["lib/pack/*.c"]),
#     hdrs = glob(["lib/pack/*.h"]),
#     includes = ["lib/pack", "lib"],
#     deps = [":cgraph", ":util", ":config_h", ":common_hdrs"],
# )
#
# # Common Headers (to resolve label <-> common cycle)
# cc_library(
#     name = "common_hdrs",
#     hdrs = glob(["lib/common/*.h"]),
#     includes = ["lib/common", "lib"],
#     deps = [
#         ":cgraph", ":pathplan", ":util", ":config_h", ":xdot",
#         ":gvc_hdrs",
#         # "@gd//:gd",
#         "@cairo//:cairo",
#         # "@pango//:pango",
#         "@glib//glib:glib",
#         "@expat//:libexpat",
#         "@zlib//:zlib",
#     ],
# )
#
# # Label Library
# cc_library(
#     name = "label",
#     srcs = glob(["lib/label/*.c"]),
#     hdrs = glob(["lib/label/*.h"]),
#     includes = ["lib/label", "lib"],
#     deps = [":cgraph", ":util", ":config_h", ":sparse", ":rbtree", ":pack", ":common_hdrs"],
# )
#
# # GVC Headers (to break common <-> gvc cycle)
# cc_library(
#     name = "gvc_hdrs",
#     hdrs = glob(["lib/gvc/*.h"]),
#     includes = ["lib/gvc"],
# )
#
# # Common Library (Graphviz common rendering)
# cc_library(
#     name = "common",
#     srcs = glob(["lib/common/*.c"]),
#     hdrs = glob(["lib/common/*.h"]), # Allow duplicate hdrs or remove if common_hdrs used? Keep for safety.
#     includes = ["lib/common", "lib"],
#     deps = [
#         ":cgraph", ":pathplan", ":util", ":config_h", ":xdot",
#         ":gvc_hdrs",
#         ":common_hdrs",
#         ":label",
#         ":pack",
#         # "@gd//:gd",
#         "@cairo//:cairo",
#         # "@pango//:pango",
#         "@glib//glib:glib",
#         "@expat//:libexpat",
#         "@zlib//:zlib",
#     ],
# )
#
# # GVC Library
# cc_library(
#     name = "gvc",
#     srcs = glob(["lib/gvc/*.c"]),
#     hdrs = glob(["lib/gvc/*.h"]),
#     includes = ["lib/gvc", "lib"],
#     deps = [
#         ":common",
#         ":cgraph",
#         ":cdt",
#         ":pathplan",
#         ":xdot",
#         ":util",
#         ":config_h",
#         ":gvc_hdrs",
#     ],
# )
#
# # Dotgen Library (Dot Layout)
# cc_library(
#     name = "dotgen",
#     srcs = glob(["lib/dotgen/*.c"]),
#     hdrs = glob(["lib/dotgen/*.h"]),
#     includes = ["lib/dotgen", "lib"],
#     deps = [":common", ":gvc", ":cgraph"],
# )
#
# # Neatogen Library (Neato Layout)
# cc_library(
#     name = "neatogen",
#     srcs = glob(["lib/neatogen/*.c"]),
#     hdrs = glob(["lib/neatogen/*.h"]),
#     includes = ["lib/neatogen", "lib"],
#     deps = [":common", ":gvc", ":cgraph"],
# )
#
# # Core Plugin
# cc_library(
#     name = "plugin_core",
#     srcs = glob(["plugin/core/*.c"]),
#     hdrs = glob(["plugin/core/*.h"], allow_empty = True),
#     includes = ["plugin/core", "lib"],
#     deps = [":gvc", ":common"],
# )
#
# # Dot Layout Plugin
# cc_library(
#     name = "plugin_dot_layout",
#     srcs = glob(["plugin/dot_layout/*.c"]),
#     hdrs = glob(["plugin/dot_layout/*.h"], allow_empty = True),
#     includes = ["plugin/dot_layout", "lib"],
#     deps = [":gvc", ":dotgen", ":common"],
# )
#
# # GVC with Builtins (Static Bundle)
# # This is a convenience target to link everything
# cc_library(
#     name = "graphviz",
#     deps = [
#         ":gvc",
#         ":plugin_core",
#         ":plugin_dot_layout",
#     ],
# )
