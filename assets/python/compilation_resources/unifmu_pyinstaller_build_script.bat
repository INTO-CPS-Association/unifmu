@echo off
REM Use this script to build a Python UniFMU as a binary - Windows
REM Note: This script uses PowerShell commands for installing pip packages and checking command existence.

SET "CURRENT_DIR=%CD%"
CALL :GET_BASENAME "%CURRENT_DIR%" LAST_FOLDER

:GET_BASENAME
    SET "IN_PATH=%~1"
    IF "%IN_PATH:~-1%"=="\" SET "IN_PATH=%IN_PATH:~0,-1%"
    SET "%2=%%~nI"
    FOR %%I IN ("%IN_PATH%") DO SET "%2=%%~nI"

set "COMPILATION_RESOURCES_PATH_NAME=compilation_resources"

if /I "%LAST_FOLDER%" EQU "%COMPILATION_RESOURCES_PATH_NAME%" (
    echo Moving current directory to resources/
    cd ..
)

for %%i in ("%CD%") do set "BASE_FOLDER_FULL_PATH=%%~dpi"
set "BASE_FOLDER_FULL_PATH=%BASE_FOLDER_FULL_PATH:~0,-1%"

for %%i in ("%BASE_FOLDER_FULL_PATH%") do set "BASE_FOLDER_NAME=%%~nxi"

set "TWO_UP_FULL_PATH=%CD%\..\.."

set "TMP_FOLDER_PATH=..\..\%BASE_FOLDER_NAME%_tmp"

:: --- Check and Install pyinstaller if required ---
SET "EXECUTABLE_NAME=pyinstaller"
SET "PACKAGE_NAME=PyInstaller"

python -m pip show %PACKAGE_NAME% >NUL 2>&1
IF %ERRORLEVEL% EQU 0 GOTO :Install_Found

echo ⚙️ Executable '%PACKAGE_NAME%' not found in PATH. Installing package '%PACKAGE_NAME%' now...

pip install "%PACKAGE_NAME%"

IF %ERRORLEVEL% EQU 0 GOTO :Install_Success

GOTO :Install_Failure

:Install_Found
echo ✅ Executable '%PACKAGE_NAME%' (from package '%PACKAGE_NAME%') is already available. Skipping installation.
GOTO :Continue_Build

:Install_Success
echo 🎉 Successfully installed '%PACKAGE_NAME%'.
GOTO :Continue_Build

:Install_Failure
echo ❌ ERROR: Failed to install '%PACKAGE_NAME%'. Install '%PACKAGE_NAME%' manually instead.
exit /b 1

:Continue_Build
:: --- Compile python app ---
python -m %PACKAGE_NAME% main.py

:: Create placeholders for zipping new fmu
mkdir "%TMP_FOLDER_PATH%"
xcopy "..\." "%TMP_FOLDER_PATH%\" /E /I /Q

:: --- Overwrite launch.toml with that for executing with pyinstaller ---
copy /Y "%COMPILATION_RESOURCES_PATH_NAME%\launch_with_pyinstaller.toml" "%TMP_FOLDER_PATH%\resources\launch.toml"

:: Remove Python-related files from tmp folder (compiled version) and build folders from original one to avoid confusion
del /Q "%TMP_FOLDER_PATH%\resources\*.py"
rmdir /S /Q "%TMP_FOLDER_PATH%\resources\schemas\"
rmdir /S /Q "%TMP_FOLDER_PATH%\resources\%COMPILATION_RESOURCES_PATH_NAME%"
del /Q "%TMP_FOLDER_PATH%\resources\requirements.txt"
del /Q "%TMP_FOLDER_PATH%\resources\README.md"
del /Q "main.spec"
rmdir /S /Q "build"
rmdir /S /Q "dist"


:: --- Wrap the folder with fmu extension (Create the .fmu file) ---
del /F /Q "..\..\%BASE_FOLDER_NAME%_compiled.fmu"

powershell -Command "Add-Type -A 'System.IO.Compression.FileSystem'; [IO.Compression.ZipFile]::CreateFromDirectory('%TMP_FOLDER_PATH%', '..\..\%BASE_FOLDER_NAME%_compiled.fmu', 'Optimal', $false)"
IF %ERRORLEVEL% NEQ 0 (
    echo ❌ ERROR: Failed to create .fmu archive.
    rmdir /S /Q "%TMP_FOLDER_PATH%"
    exit /b 1
)

rmdir /S /Q "%TMP_FOLDER_PATH%"

echo 🎉 Successfully created compiled FMU in: %TWO_UP_FULL_PATH%\%BASE_FOLDER_NAME%_compiled.fmu