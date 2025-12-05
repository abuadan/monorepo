
##################### Utils ####################

.PHONY: clean-py
clean-py:
		find . | grep -E "(/__pycache__$$|\.pyc$$|\.pyo$$)" | xargs rm -rf


.PHONY: pytest
pytest:
	poetry -C pyproject.toml run pytest  tests

.PHONY: pip-compile
pip-compile:
	pipx run --spec pip-tools pip-compile --color --verbose --all-build-deps --all-extras --allow-unsafe --output-file=requirements.in ../pyproject.toml

.PHONY: gazelle-update-python-dep
gazelle-update-python-dep:
	# bazel run //python:requirements.update
	bazel run //:generate_requirements_txt

.PHONY: gazelle-update-manifest
gazelle-update-manifest:
	bazel run //:gazelle_python_manifest.update

.PHONY: update-python-dependencies
update-python-dependencies: pip-compile gazelle-update-python-dep gazelle-update-manifest

