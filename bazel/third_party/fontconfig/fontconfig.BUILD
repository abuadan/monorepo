# load("@rules_cc//cc:defs.bzl", "cc_library")
#
# genrule(
#     name = "generate_aliases",
#     srcs = ["fontconfig/fontconfig.h", "src/makealias"] + glob(["src/*.c"]),
#     outs = ["src/fcalias.h", "src/fcaliastail.h"],
#     # usage: makealias SRCDIR HEAD TAIL [INPUT_FILES...]
#     # We pass srcdir as the directory where .c files are located.
#     # The script uses grep on SRCDIR/*.c to find ifdefs.
#     # We need to ensure SRCDIR is correct relative to execution.
#     # The script uses $SRCDIR/*.c.
#     cmd = "sh $(location src/makealias) $$(dirname $(location src/makealias)) $(location src/fcalias.h) $(location src/fcaliastail.h) $(location fontconfig/fontconfig.h)",
# )
#
# genrule(
#     name = "generate_ft_aliases",
#     srcs = ["fontconfig/fcfreetype.h", "src/makealias"] + glob(["src/*.c"]),
#     outs = ["src/fcftalias.h", "src/fcftaliastail.h"],
#     cmd = "sh $(location src/makealias) $$(dirname $(location src/makealias)) $(location src/fcftalias.h) $(location src/fcftaliastail.h) $(location fontconfig/fcfreetype.h)",
# )
#
# genrule(
#     name = "generate_fccase",
#     srcs = ["fc-case/fc-case.py", "fc-case/CaseFolding.txt", "fc-case/fccase.tmpl.h"],
#     outs = ["fc-case/fccase.h"],
#     cmd = "python3 $(location fc-case/fc-case.py) $(location fc-case/CaseFolding.txt) --output $(location fc-case/fccase.h) --template $(location fc-case/fccase.tmpl.h)",
# )
#
# genrule(
#     name = "generate_fclang",
#     srcs = ["fc-lang/fc-lang.py", "fc-lang/fclang.tmpl.h"] + glob(["fc-lang/*.orth"]),
#     outs = ["fc-lang/fclang.h"],
#     cmd = """
#         dir=$$(dirname $(location fc-lang/aa.orth))
#         files=$$(cd $$dir && ls *.orth)
#         python3 $(location fc-lang/fc-lang.py) --directory $$dir --output $(location fc-lang/fclang.h) --template $(location fc-lang/fclang.tmpl.h) $$files
#     """,
# )
#
# genrule(
#     name = "generate_fcobjshash",
#     srcs = ["src/fcobjshash.gperf.h", "src/fcobjs.h"],
#     outs = ["src/fcobjshash.h"],
#     # Manually construct the gperf input file because gperf doesn't support C preprocessor includes directly in the keywords section.
#     # 1. Extract the prelude from .gperf.h (up to %%)
#     # 2. Extract FC_OBJECT macros from fcobjs.h and transform them to "FC_NAME, FC_NAME_OBJECT" format
#     # 3. Run gperf on the result
#     cmd = """
#         cat $(location src/fcobjshash.gperf.h) | sed -n '1,/%%/p' | grep -v 'CUT_OUT' > fcobjshash.gperf
#         grep '^FC_OBJECT' $(location src/fcobjs.h) | sed 's/FC_OBJECT *(\\([^,]*\\).*/FC_\\1, FC_\\1_OBJECT/' >> fcobjshash.gperf
#         gperf -m 100 fcobjshash.gperf > $(location src/fcobjshash.h)
#     """,
# )
#
# genrule(
#     name = "config_h",
#     outs = ["config.h"],
#     cmd = """cat > $@ <<EOF
# #define HAVE_DIRENT_H 1
# #define HAVE_FCNTL_H 1
# #define HAVE_STDLIB_H 1
# #define HAVE_STRING_H 1
# #define HAVE_UNISTD_H 1
# #define HAVE_SYS_STAT_H 1
# #define HAVE_SYS_TYPES_H 1
# #define HAVE_FT2BUILD_H 1
# #define HAVE_VPRINTF 1
# #define HAVE_RANDOM 1
# #define HAVE_LSTAT 1
# #define HAVE_EXPAT 1
# #define HAVE_STDATOMIC_PRIMITIVES 1
# #define HAVE_MKSTEMP 1
# #define HAVE_GETOPT 1
# #define HAVE_GETOPT_LONG 1
# #define FC_DEFAULT_FONTS "/usr/share/fonts"
# #define FC_CACHEDIR "/var/cache/fontconfig"
# #define FONTCONFIG_PATH "/etc/fonts"
# #define CONFIGDIR "/etc/fonts/conf.d"
# #define FC_TEMPLATEDIR "/usr/share/fontconfig/conf.avail"
# #define ALIGNOF_DOUBLE 8
# #define SIZEOF_VOID_P 8
# #define SIZEOF_INT 4
# #define ALIGNOF_VOID_P 8
# #define FC_GPERF_SIZE_T unsigned int
# #define FLEXIBLE_ARRAY_MEMBER
# EOF""",
# )
#
# # Some files in src/ might be generated or special tools.
# # Excluding fc-*.c that are tools.
# cc_library(
#     name = "fontconfig",
#     srcs = glob(
#         ["src/*.c"],
#         exclude = [
#             "src/fc-case.c", # Example generated or tool
#             "src/fc-lang.c",
#             "src/fc-glyphname.c",
#             # Exclude main programs
#             "src/fcarch.c",
#         ],
#     ),
#     hdrs = glob(["fontconfig/*.h", "src/*.h"]) + [
#         ":config_h",
#         ":generate_aliases",
#         ":generate_ft_aliases",
#         ":generate_fccase",
#         ":generate_fclang",
#         ":generate_fcobjshash",
#     ],
#     copts = [
#         "-DHAVE_CONFIG_H",
#         "-I$(GENDIR)/external/fontconfig",
#         "-I$(GENDIR)/external/fontconfig/src",
#         "-Iexternal/fontconfig",
#         "-Iexternal/fontconfig/src",
#     ],
#     includes = [".", "src"],
#     deps = [
#         "@expat//:libexpat",
#         "@freetype//:freetype",
#     ],
#     visibility = ["//visibility:public"],
# )
