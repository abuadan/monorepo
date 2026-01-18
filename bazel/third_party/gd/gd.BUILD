# package(default_visibility = ["//visibility:public"])
#
# # Generate config.h for LibGD
# genrule(
#     name = "generate_config_h",
#     outs = ["src/config.h"],
#     cmd = "cat > $@ <<EOF\n" +
#           "#define HAVE_DIRENT_H 1\n" +
#           "#define HAVE_DLFCN_H 1\n" +
#           "#define HAVE_ERRNO_H 1\n" +
#           "#define HAVE_FCNTL_H 1\n" +
#           "#define HAVE_FT2BUILD_H 1\n" +
#           "#define HAVE_ICONV 1\n" +
#           "#define HAVE_ICONV_H 1\n" +
#           "#define HAVE_ICONV_T_DEF 1\n" +
#           "#define HAVE_INTTYPES_H 1\n" +
#           "#define HAVE_LIBFONTCONFIG 1\n" +
#           "#define HAVE_LIBFREETYPE 1\n" +
#           "#define HAVE_LIBJPEG 1\n" +
#           "#define HAVE_LIBPNG 1\n" +
#           "#define HAVE_LIBZ 1\n" +
#           "#define HAVE_LIMITS_H 1\n" +
#           "#define HAVE_MEMORY_H 1\n" +
#           "#define HAVE_STDDEF_H 1\n" +
#           "#define HAVE_STDINT_H 1\n" +
#           "#define HAVE_STDLIB_H 1\n" +
#           "#define HAVE_STRING_H 1\n" +
#           "#define HAVE_STRINGS_H 1\n" +
#           "#define HAVE_SYS_STAT_H 1\n" +
#           "#define HAVE_SYS_TYPES_H 1\n" +
#           "#define HAVE_UNISTD_H 1\n" +
#           "#define PACKAGE \"gd\"\n" +
#           "#define PACKAGE_BUGREPORT \"https://github.com/libgd/libgd/issues\"\n" +
#           "#define PACKAGE_NAME \"GD\"\n" +
#           "#define PACKAGE_STRING \"GD 2.3.3\"\n" +
#           "#define PACKAGE_TARNAME \"gd\"\n" +
#           "#define PACKAGE_URL \"https://libgd.org/\"\n" +
#           "#define PACKAGE_VERSION \"2.3.3\"\n" +
#           "#define STDC_HEADERS 1\n" +
#           "#define VERSION \"2.3.3\"\n" +
#           "EOF",
# )
#
# cc_library(
#     name = "gd",
#     srcs = glob(
#         ["src/*.c"],
#         exclude = [
#             "src/gd_tiff.c",   # Disabled tiff
#             "src/gd_webp.c",   # Disabled webp
#             "src/gd_xpm.c",    # Disabled xpm
#             "src/gd_color_match.c",
#             "src/gdtest.c",
#             "src/*_test.c",
#         ],
#     ),
#     hdrs = glob(["src/*.h"]) + [":generate_config_h"],
#     copts = [
#         "-DHAVE_CONFIG_H",
#         "-include src/config.h",
#         "-U_MSC_VER",
#         "-U__BORLANDC__",
#         "-U__DMC__",
#     ],
#     includes = ["src"],
#     deps = [
#         "@libpng//:libpng",
#         "@zlib//:zlib",
#         "@libjpeg_turbo//:jpeg",
#         "@freetype//:freetype",
#         "@fontconfig//:fontconfig",
#     ],
# )
