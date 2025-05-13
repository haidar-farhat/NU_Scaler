@echo off
echo Building NuScaler executable...

:: Activate virtual environment if it exists
if exist .venv\Scripts\activate.bat (
    echo Activating virtual environment...
    call .venv\Scripts\activate.bat
) else (
    echo No virtual environment found, using system Python...
)

:: Run the build script
python build_executable.py

:: Check if build was successful
if errorlevel 1 (
    echo Build failed!
    pause
    exit /b 1
)

echo.
echo Build completed successfully!
echo Executable is located in the 'dist' folder
echo.

:: Show file size
for %%F in (dist\NuScaler.exe) do (
    echo File size: %%~zF bytes
)

pause 