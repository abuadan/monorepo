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
#     name = "fontconfig",
#     lib_source = ":all_srcs",
#     deps = [
#         "@expat//:libexpat",
#         "@freetype",
#     ],
#     out_lib_dir = "lib",
#     out_shared_libs = select({
#         "@platforms//os:macos": ["libfontconfig.dylib"],
#         "//conditions:default": ["libfontconfig.so"],
#     }),
#     configure_options = [
#         "--enable-libxml2=no",
#         "--enable-iconv=yes",
#         # "--disable-docs", # Not a valid flag?
#     ],
#     env = {
#         "FREETYPE_CFLAGS": " ", # Rely on compiler wrapper
#         "FREETYPE_LIBS": "-lfreetype",
#     },
# )
