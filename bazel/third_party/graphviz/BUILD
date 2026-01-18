# load("@rules_foreign_cc//foreign_cc:defs.bzl", "configure_make")
#
# package(default_visibility = ["//visibility:public"])
#
# filegroup(
#     name = "all_srcs",
#     srcs = glob(["**"]),
# )
#
# configure_make(
#     name = "graphviz",
#     lib_source = ":all_srcs",
#     deps = [
#         # "@gd//:gd",
#         "@cairo//:cairo",
#         # "@pango//:pango",
#         "@glib//glib",
#         "@expat//:libexpat",
#         "@zlib",
#         # "@gts", # Optional, skipped for now
#     ],
#     out_lib_dir = "lib",
#     autogen = True,
#     autogen_command = "autoreconf -vif",
#     configure_in_place = True,
#     out_headers_only = False,
#     # Graphviz produces many libraries. We need to list the main ones we care about.
#     # common ones: libgvc, libcgraph, libcdt, libpathplan, libxdot
#     out_shared_libs = select({
#         "@platforms//os:macos": [
#             "libgvc.dylib",
#             "libcgraph.dylib",
#             "libcdt.dylib",
#             "libxdot.dylib",
#             "libpathplan.dylib",
#             # "liblab_gamut.dylib",
#         ],
#         "//conditions:default": [
#             "libgvc.so",
#             "libcgraph.so",
#             "libcdt.so",
#             "libxdot.so",
#             "libpathplan.so",
#         ],
#     }),
#     configure_options = [
#         "--with-quartz=yes", # macOS
#         "--with-x=no",
#         "--disable-swig",
#         "--disable-sharp",
#         "--disable-go",
#         "--disable-guile",
#         "--disable-io",
#         "--disable-java",
#         "--disable-lua",
#         "--disable-ocaml",
#         "--disable-perl",
#         "--disable-php",
#         "--disable-python", # We bind via pygraphviz separately
#         "--disable-r",
#         "--disable-ruby",
#         "--disable-tcl",
#     ],
#     # ensure it finds includes from deps
#     env = {
#         "CFLAGS": "-Wno-deprecated-declarations",
#         "PATH": "/opt/homebrew/opt/bison/bin:/opt/homebrew/bin:$$PATH",
#     },
# )
