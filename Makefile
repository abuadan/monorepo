
##################### Utils ####################

.PHONY: clean-py
clean-py:
		find . | grep -E "(/__pycache__$$|\.pyc$$|\.pyo$$)" | xargs rm -rf


.PHONY: pytest
pytest:
	poetry -C python/pyproject.toml run pytest  python/tests

.PHONY: pip-compile
pip-compile:
	pip-compile --all-build-deps --all-extras --output-file=python/requirements.in pyproject.toml

.PHONY: gazelle-update-python-dep
gazelle-update-python-dep:
	# bazel run //python:requirements.update
	bazel run //:generate_requirements_txt

.PHONY: gazelle-update-manifest
gazelle-update-manifest:
	bazel run //gazelle_python_manifest.update

.PHONY: update-python-dependencies
update-python-dependencies: pip-compile gazelle-update-python-dep gazelle-update-manifest
