
##################### Utils ####################

.PHONY: clean-py
clean-py:
		find . | grep -E "(/__pycache__$$|\.pyc$$|\.pyo$$)" | xargs rm -rf
