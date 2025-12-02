load("@rules_python//python:defs.bzl", "py_library")

py_library(
    name = "__init__",
    srcs = ["__init__.py"],
    imports = ["../.."],
    visibility = [
        "//python/src:__subpackages__",
        "//python/tests:__subpackages__",
    ],
    deps = [":base"],
)

py_library(
    name = "base",
    srcs = ["base.py"],
    imports = ["../.."],
    visibility = [
        "//python/src:__subpackages__",
        "//python/tests:__subpackages__",
    ],
    deps = [
        "//python/src/dataplatform/compute/schema:__init__",
        "//python/src/dataplatform/storage:protocol",
    ],
)
