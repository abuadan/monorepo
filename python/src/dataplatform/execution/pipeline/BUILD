load("@rules_python//python:defs.bzl", "py_library")

py_library(
    name = "__init__",
    srcs = ["__init__.py"],
    imports = ["../../.."],
    visibility = [
        "//python/src:__subpackages__",
        "//python/tests:__subpackages__",
    ],
    deps = [":graph"],
)

py_library(
    name = "graph",
    srcs = ["graph.py"],
    imports = ["../../.."],
    visibility = [
        "//python/src:__subpackages__",
        "//python/tests:__subpackages__",
    ],
    deps = [
        "//python/src/dataplatform/compute:base",
        "@pypi//networkx",
    ],
)
