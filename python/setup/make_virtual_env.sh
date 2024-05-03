#!/bin/bash

# Determine the directory of the script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Set the name of your virtual environment
VENV_NAME=".venv"

# Specify the Python version
PYTHON_VERSION="3.11"
#
#
poetry install -vvv --no-root
# # Create a virtual environment in the specified directory
# python$PYTHON_VERSION -m venv "$SCRIPT_DIR/$VENV_NAME"
#
# # Activate the virtual environment
# source "$SCRIPT_DIR/$VENV_NAME/bin/activate"
#
# # Install pip-tools if not already installed
# pip install --upgrade pip-tools
#
# # Compile requirements.in to requirements.txt
# pip-compile --output-file="$SCRIPT_DIR/requirements.txt" "$SCRIPT_DIR/requirements.in"
#
# # Install dependencies from requirements.txt
# pip install -r "$SCRIPT_DIR/requirements.txt"
#
# # Display a message indicating that the virtual environment is activated
# echo "Virtual environment '$VENV_NAME' created and activated in '$SCRIPT_DIR' with Python $PYTHON_VERSION."
