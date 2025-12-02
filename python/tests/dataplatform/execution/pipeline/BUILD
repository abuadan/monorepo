load("@rules_python//python:defs.bzl", "py_test")

py_test(
    name = "test_graph",
    srcs = ["test_graph.py"],
    deps = [
        "//python/src/dataplatform/compute:__init__",
        "//python/src/dataplatform/compute/schema:__init__",
        "//python/src/dataplatform/execution/pipeline:__init__",
        "//python/src/dataplatform/execution/pipeline:graph",
        "//python/src/dataplatform/storage:protocol",
        "@pypi//pytest",
    ],
)
