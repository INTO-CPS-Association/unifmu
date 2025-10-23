#!/bin/sh
## Use this script to compile a Python UniFMU as an executable - Linux

CURRENT_DIR="$PWD"
LAST_FOLDER=$(basename "$CURRENT_DIR")
COMPILATION_RESOURCES_PATH_NAME="compilation_resources"

if [ "$LAST_FOLDER" = "$COMPILATION_RESOURCES_PATH_NAME" ]; then
    echo "Moving current directory to resources/"
    cd .. # Moving script to resource folder
fi

BASE_FOLDER_FULL_PATH=$(dirname "$PWD") # Script executed from resources/
BASE_FOLDER_NAME=$(basename "$BASE_FOLDER_FULL_PATH")

TWO_UP_FULL_PATH=$(dirname "$(dirname "$PWD")")


TMP_FOLDER_PATH=../../${BASE_FOLDER_NAME}_tmp

PACKAGE_NAME="pyinstaller"
EXECUTABLE_NAME="pyinstaller" 
## Install pyinstaller if required
command -v "$EXECUTABLE_NAME" >/dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ Executable '$EXECUTABLE_NAME' (from package '$PACKAGE_NAME') is already available. Skipping installation."
else
    echo "⚙️ Executable '$EXECUTABLE_NAME' not found in PATH. Installing package '$PACKAGE_NAME' now..."
    pip3 install "$PACKAGE_NAME"
    if [ $? -eq 0 ]; then
        echo "🎉 Successfully installed '$PACKAGE_NAME'."
    else
        echo "❌ ERROR: Failed to install '$PACKAGE_NAME'. Install '$PACKAGE_NAME' manually instead."
        exit 1
    fi
fi

## Compile python app
"$EXECUTABLE_NAME" backend.py

## Create placeholders for zipping new fmu
mkdir ${TMP_FOLDER_PATH}
cp -r ../. ${TMP_FOLDER_PATH}

## Overwrite launch.toml with that for executing with pyinstaller
cp ${COMPILATION_RESOURCES_PATH_NAME}/launch_with_pyinstaller.toml ${TMP_FOLDER_PATH}/resources/launch.toml

## Remove Python-related files from tmp folder (compiled version) and build folders from original one to avoid confusion
rm ${TMP_FOLDER_PATH}/resources/*.py
rm -r ${TMP_FOLDER_PATH}/resources/schemas/
rm -r ${TMP_FOLDER_PATH}/resources/${COMPILATION_RESOURCES_PATH_NAME}
rm backend.spec
rm -r build
rm -r dist 

## Wrap the folder with fmu extension
rm -f ../../${BASE_FOLDER_NAME}_compiled.fmu
(cd ${TMP_FOLDER_PATH} && zip -r ${BASE_FOLDER_NAME}.fmu .)
cp ${TMP_FOLDER_PATH}/${BASE_FOLDER_NAME}.fmu ../../${BASE_FOLDER_NAME}_compiled.fmu
rm -r ${TMP_FOLDER_PATH}

echo "🎉 Successfully created compiled FMU in '${TWO_UP_FULL_PATH}/${BASE_FOLDER_NAME}_compiled.fmu'."
