load("@pip//:requirements.bzl", "all_whl_requirements")
load("@rules_python_gazelle_plugin//manifest:defs.bzl", "gazelle_python_manifest")
load("@rules_python_gazelle_plugin//modules_mapping:def.bzl", "modules_mapping")
load("@rules_uv//uv:pip.bzl", "pip_compile")

exports_files(
    [
        "test.in",
        "requirements.in",
        "requirements.lock",
        "requirements.txt",
    ],
)
# This directive tells gazelle that our tests are named "test_foo.py" instead
# of "foo_test.py".
# gazelle:python_test_naming_convention test_$package_name$

# This directive tells gazelle to make a single bazel target per python file.
# The default is to make a single bazel target per python _package_).
# gazelle:python_generation_mode file

# filegroup(
#     name = "src_dir",
#     srcs = ["//python:src"],
# )

################################################################################
# Gazelle
#################################################################################
# Gazelle is an automatic BUILD(.bazel) file generator. Run via:
# bazel run //:gazelle
#################################################################################

# Comments that start with "# gazelle:XYZ" are called *directives*. Some directives
# can and should be set here, in the same bazel package (BUILD file) that defines
# gazelle, while other directives (such as "# gazelle:python_root") should be
# defined in a BUILD file specific to that part of the folder tree. See
# src/BUILD for such an example - it's how we define that the "src" dir should
# be the root of python files and thus get added to sys.path.

# This directive tells gazelle that our tests are named "test_foo.py" instead
# of "foo_test.py".
# gazelle:python_test_naming_convention test_$package_name$

# This directive tells gazelle to make a single bazel target per python file.
# The default is to make a single bazel target per python _package_).
# gazelle:python_generation_mode file

# This directive would be used if, for example, we wanted to make pytest_test
# rules instead of py_test rules. However, this project doesn't use `pytest`
# so the directive is inactive (double #).
## gazelle:map_kind py_test pytest_test //python:defs.bzl

# This directive tells gazelle to use import `foobar` using the py_library
# target, rather than the py_binary target (`foobar.py` is both a library
# and an executable, so gazelle gets confuzed, saying that multiple targets
# can satisfy the "mypackage.foobar" import).
# This directive can be set multiple times.
## gazelle:resolve py mypackage.foobar //src/mypackage:foobar

################################################################################
# End Gazelle Directives
################################################################################
# This rule will compile the project requirements into a lock file that
# contains versions and hashes. The lock file ends up getting used when
# installing dependencies via pip.
# bazel run //python:requirements.update
# compile_pip_requirements(
#     name = "requirements",
#     src = "requirements.in",
#     requirements_txt = "requirements.lock",
# )
# A single constraint-solution for the full monorepo.
# Allows dependencies between different applications without risk of version skew.
# NB: this can easily become overconstrained. It's okay to have applications opt-out.
# they can use virtual deps in rules_py in complex cases.
pip_compile(
    name = "requirements",
    requirements_in = "//:pyproject.toml",
    requirements_txt = "//python:requirements.txt",
)

# A single constraint-solution for the full monorepo.
# Allows dependencies between different applications without risk of version skew.
# NB: this can easily become overconstrained. It's okay to have applications opt-out.
# they can use virtual deps in rules_py in complex cases.
pip_compile(
    name = "requirements.all",
    data = [
        "requirements.txt",
        "test.in",
    ],
    requirements_in = "requirements.in",
    requirements_txt = "requirements.lock",
)

# This rule fetches the metadata for python packages we depend on. That data is
# required for the gazelle_python_manifest rule to update our manifest file.
modules_mapping(
    name = "modules_map",
    exclude_patterns = [
        "^_|(\\._)+",  # This is the default.
        "(\\.tests)+",  # Add a custom one to get rid of the psutil tests.
        "^colorama",  # Get rid of colorama on Windows.
        "^(\\.pip)+",
        "^lazy_object_proxy\\.cext$",  # Get rid of this on Linux because it isn't included on Windows.
    ],
    wheels = all_whl_requirements,
)

# Gazelle python extension needs a manifest file mapping from
# an import to the installed package that provides it. This target updates the
# "gazelle_python.yaml" file when run. The file must already exist.
# This target produces two targets:
# bazel run //:gazelle_python_manifest.update
# bazel run //:gazelle_python_manifest.test
gazelle_python_manifest(
    # Name this target. This will be how you run with `bazel run //:<name>.update`
    name = "gazelle_python_manifest",
    manifest = "//:gazelle_python.yaml",
    # Same as `modules_mapping.name` (with ":"), above.
    modules_mapping = ":modules_map",
    # This is what we called our `pip_parse` rule, where third-party
    # python libraries are loaded in BUILD files.
    pip_repository_name = "pip",
    # This should point to wherever we declare our python dependencies.
    # It's the same as what we passed to the pip.parse rule in MODULE.bazel and
    # is the same filename that we used in the `requirements_txt` attribute of
    # `compile_pip_requirements`, above.)
    # This argument is optional. If provided, the `.test` target is very
    # fast because it just has to check an integrity field. If not provided,
    # the integrity field is not added to the manifest which can help avoid
    # merge conflicts in large repos.
    requirements = "//python:requirements.lock",
)
